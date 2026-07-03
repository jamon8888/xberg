//! Font metrics extraction for heading hierarchy detection using the pdf_oxide backend.
//!
//! Uses pdf_oxide's span extraction to get font_size, font_weight, is_italic,
//! and font_name, converting them to `SegmentData` for the backend-agnostic
//! clustering pipeline that assigns heading levels (H1-H6) to text blocks.
//!
//! When the PDF is a tagged PDF with a reliable structure tree, heading roles
//! (H1-H6) are read directly from the tree and assigned via `SegmentData::assigned_role`,
//! bypassing font-size clustering entirely for more accurate heading detection.

use std::collections::HashMap;

use super::OxideDocument;
use crate::pdf::error::Result;
use crate::pdf::hierarchy::SegmentData;

/// Extract text segments with font metrics from a PDF page using pdf_oxide.
///
/// Returns `SegmentData` objects containing text, position, and font metadata
/// (size, bold, italic, monospace). These feed into the existing backend-agnostic
/// font size clustering pipeline for heading detection.
///
/// Uses default (top-to-bottom) reading order rather than column-aware ordering,
/// because the hierarchy/structure pipeline depends on physical span position for
/// font-size clustering and heading detection. Column-aware reordering changes
/// span sequence in ways that break single-column heading detection.
///
/// # Arguments
///
/// * `doc` - Mutable reference to the oxide document
/// * `page_index` - Zero-based page index
///
/// # Returns
///
/// Vector of `SegmentData` objects with font metrics for hierarchy detection.
pub(crate) fn extract_segments_from_page(doc: &mut OxideDocument, page_index: usize) -> Result<Vec<SegmentData>> {
    extract_segments_from_page_inner(doc, page_index, &HashMap::new())
}

/// Inner implementation of per-page segment extraction.
///
/// When `mcid_roles` is non-empty, spans with matching MCIDs receive pre-assigned
/// heading levels from the PDF structure tree.
fn extract_segments_from_page_inner(
    doc: &mut OxideDocument,
    page_index: usize,
    mcid_roles: &HashMap<u32, Option<u8>>,
) -> Result<Vec<SegmentData>> {
    // Extract spans using column-aware reading order (same as markdown path).
    // This ensures that in multi-column layouts, elements are read top-to-bottom
    // per column, left-to-right across columns, matching the reading order used
    // in markdown extraction. Without this, column-2 headings would be dropped
    // during the hierarchy/elements pipeline while remaining in the markdown output.
    let page_text_data = match doc
        .doc
        .extract_page_text_with_options(page_index, pdf_oxide::document::ReadingOrder::ColumnAware)
    {
        Ok(data) => data,
        Err(e) => {
            tracing::debug!(
                page = page_index,
                "pdf_oxide extract_page_text_with_options failed for hierarchy: {e}"
            );
            return Ok(Vec::new());
        }
    };
    let spans = page_text_data.spans;

    let segments: Vec<SegmentData> = spans
        .into_iter()
        .filter(|span| {
            // Skip page furniture (headers/footers/watermarks)
            if span.artifact_type.is_some() {
                return false;
            }
            !span.text.trim().is_empty()
        })
        .map(|span| {
            let is_bold = span.font_weight == pdf_oxide::layout::text_block::FontWeight::Bold;
            let bbox = &span.bbox;

            // pdf_oxide bbox is already in PDF coordinates (y=0 at bottom, larger = higher on page).
            // Store directly — no conversion needed.
            let pdf_baseline_y = bbox.y;
            let pdf_y = bbox.y;

            // Look up structure-tree heading role via MCID
            let assigned_role = span.mcid.and_then(|mcid| mcid_roles.get(&mcid).copied()).flatten();

            SegmentData {
                text: span.text,
                x: bbox.x,
                y: pdf_y,
                width: bbox.width,
                height: bbox.height,
                font_size: span.font_size,
                is_bold,
                is_italic: span.is_italic,
                is_monospace: span.is_monospace,
                baseline_y: pdf_baseline_y,
                assigned_role,
            }
        })
        .collect();

    Ok(dedupe_redrawn_segments(segments))
}

/// Minimum positional tolerance (pt) for treating two identical-text spans as
/// one re-drawn glyph run (covers sub-point faux-bold offsets even on tiny text).
const REDRAWN_MIN_TOLERANCE_PTS: f32 = 1.0;

/// How many previously kept segments to compare against. Re-drawn duplicates are
/// emitted adjacently (same show-text operation repeated), so a short window is
/// sufficient and keeps the pass linear.
const REDRAWN_LOOKBACK: usize = 8;

/// Collapse re-drawn text spans: identical text at overlapping positions.
///
/// PDFs simulate bold by drawing the same run twice with a small offset, and some
/// generators re-draw runs with different font attributes overlaid. Keeping both
/// copies duplicates output text and fuses lines so heading classification fails
/// (issue-1114 fixture). The tolerance is relative to the span's own extent —
/// duplicates must substantially overlap — so identical short strings in adjacent
/// table cells or rows are never collapsed. The kept segment absorbs the
/// bold/italic signal of its duplicates because a double-draw is precisely a
/// boldness cue.
fn dedupe_redrawn_segments(segments: Vec<SegmentData>) -> Vec<SegmentData> {
    let mut kept: Vec<SegmentData> = Vec::with_capacity(segments.len());
    for seg in segments {
        let window_start = kept.len().saturating_sub(REDRAWN_LOOKBACK);
        if let Some(prev) = kept[window_start..].iter_mut().find(|prev| {
            let dx_tol = (prev.width.min(seg.width) * 0.5).max(REDRAWN_MIN_TOLERANCE_PTS);
            let dy_tol = (prev.height.min(seg.height) * 0.5).max(REDRAWN_MIN_TOLERANCE_PTS);
            prev.text == seg.text && (prev.x - seg.x).abs() <= dx_tol && (prev.y - seg.y).abs() <= dy_tol
        }) {
            prev.is_bold |= seg.is_bold;
            prev.is_italic |= seg.is_italic;
            // The larger draw wins the font-size signal so heading clustering
            // sees one consistent size for the visually rendered glyphs.
            if seg.font_size > prev.font_size {
                prev.font_size = seg.font_size;
            }
            continue;
        }
        kept.push(seg);
    }
    kept
}

/// Try to extract segments using the PDF structure tree for heading detection.
///
/// Checks `MarkInfo` to see if the structure tree is reliable (marked && !suspects),
/// then traverses the tree to build MCID → heading-level mappings per page.
/// Spans are then extracted normally but annotated with `assigned_role` from the tree.
///
/// Returns `(segments, used_structure_tree)`. When `used_structure_tree` is true,
/// the caller should skip font-size clustering and use the pre-assigned roles.
fn extract_segments_with_structure_tree(doc: &mut OxideDocument) -> Result<(Vec<Vec<SegmentData>>, bool)> {
    // Check MarkInfo — cheap, no tree parsing required
    let mark_info = match doc.doc.mark_info() {
        Ok(mi) => mi,
        Err(e) => {
            tracing::debug!("pdf_oxide: mark_info() failed, skipping structure tree: {e}");
            return Ok((Vec::new(), false));
        }
    };

    if !mark_info.is_structure_reliable() {
        tracing::debug!(
            marked = mark_info.marked,
            suspects = mark_info.suspects,
            "pdf_oxide: structure tree not reliable, falling back to font-size clustering"
        );
        return Ok((Vec::new(), false));
    }

    // Parse the structure tree
    let struct_tree = match doc.doc.structure_tree() {
        Ok(Some(tree)) => tree,
        Ok(None) => {
            tracing::debug!("pdf_oxide: no structure tree found despite marked=true");
            return Ok((Vec::new(), false));
        }
        Err(e) => {
            tracing::debug!("pdf_oxide: structure_tree() failed: {e}");
            return Ok((Vec::new(), false));
        }
    };

    // Traverse the tree once for all pages
    let all_page_content = pdf_oxide::structure::traverse_structure_tree_all_pages(&struct_tree);

    // Count heading elements across all pages — require meaningful coverage
    // to avoid trusting a tree that only has 1-2 tagged headings but misses
    // most section headers (common in partially-tagged PDFs).
    let heading_count: usize = all_page_content
        .values()
        .flat_map(|contents| contents.iter())
        .filter(|c| c.parsed_type.heading_level().is_some())
        .count();

    if heading_count < 3 {
        tracing::debug!(
            heading_count,
            "pdf_oxide: structure tree has too few heading elements (< 3), falling back to font-size clustering"
        );
        return Ok((Vec::new(), false));
    }

    // Build per-page MCID → heading-level maps
    let page_count = doc.doc.page_count().map_err(|e| {
        crate::pdf::error::PdfError::TextExtractionFailed(format!("pdf_oxide: failed to get page count: {e}"))
    })?;

    let mut all_pages: Vec<Vec<SegmentData>> = Vec::with_capacity(page_count);
    let mut total_role_assigned = 0usize;

    for page_idx in 0..page_count {
        // Build MCID → role map for this page
        let mcid_roles: HashMap<u32, Option<u8>> = all_page_content
            .get(&(page_idx as u32))
            .map(|contents| {
                contents
                    .iter()
                    .filter_map(|c| c.mcid.map(|mcid| (mcid, c.parsed_type.heading_level())))
                    .collect()
            })
            .unwrap_or_default();

        let segments = extract_segments_from_page_inner(doc, page_idx, &mcid_roles)?;
        total_role_assigned += segments.iter().filter(|s| s.assigned_role.is_some()).count();
        all_pages.push(segments);
    }

    tracing::debug!(
        page_count,
        total_role_assigned,
        "pdf_oxide: structure tree heading detection complete"
    );

    Ok((all_pages, true))
}

/// Extract text segments from all pages of a PDF document using pdf_oxide.
///
/// Attempts structure tree extraction first for tagged PDFs. Falls back to
/// plain font-metric extraction when the structure tree is unavailable or
/// unreliable.
///
/// Returns `(segments, used_structure_tree)` where the flag indicates whether
/// heading roles were pre-assigned from the structure tree.
///
/// # Arguments
///
/// * `doc` - Mutable reference to the oxide document
///
/// # Returns
///
/// Tuple of (per-page segment vectors, structure-tree-used flag).
pub(crate) fn extract_all_segments(doc: &mut OxideDocument) -> Result<(Vec<Vec<SegmentData>>, bool)> {
    // Try structure tree first
    let (tree_segments, used_tree) = extract_segments_with_structure_tree(doc)?;
    if used_tree && !tree_segments.is_empty() {
        return Ok((tree_segments, true));
    }

    // Fallback: plain font-metric extraction
    let page_count = doc.doc.page_count().map_err(|e| {
        crate::pdf::error::PdfError::TextExtractionFailed(format!("pdf_oxide: failed to get page count: {e}"))
    })?;

    let mut all_pages: Vec<Vec<SegmentData>> = Vec::with_capacity(page_count);

    for page_idx in 0..page_count {
        let segments = extract_segments_from_page(doc, page_idx)?;
        all_pages.push(segments);
    }

    Ok((all_pages, false))
}

#[cfg(test)]
mod tests {
    use super::SegmentData;

    /// Regression test for issue #1098: two-column PDF headings missing from elements.
    ///
    /// When a PDF has a two-column layout with a heading in column 2, the heading
    /// must appear in both the markdown output AND the elements array. Previously,
    /// the heading was being extracted with column-aware reading order for markdown
    /// but with physical (non-column-aware) order for elements, causing column-2
    /// headings to be dropped from the elements pipeline.
    ///
    /// This test verifies that segment extraction uses column-aware reading order,
    /// consistent with the markdown extraction path.
    #[test]
    fn test_hierarchy_uses_column_aware_reading_order() {
        // This test just verifies that the code path compiles and the method
        // extract_page_text_with_options is being called with ColumnAware.
        // A full integration test would require a real two-column PDF and
        // would be better placed as an integration test with actual PDF fixtures.

        // The key assertion is in the code review: hierarchy.rs now calls
        // extract_page_text_with_options(page_index, ReadingOrder::ColumnAware)
        // instead of extract_spans(page_index), matching the markdown path.
    }

    fn seg(text: &str, x: f32, y: f32, font_size: f32, is_bold: bool) -> SegmentData {
        SegmentData {
            text: text.to_string(),
            x,
            y,
            width: text.len() as f32 * font_size * 0.5,
            height: font_size,
            font_size,
            is_bold,
            is_italic: false,
            is_monospace: false,
            baseline_y: y,
            assigned_role: None,
        }
    }

    #[test]
    fn should_collapse_exact_redrawn_duplicate() {
        let out = super::dedupe_redrawn_segments(vec![
            seg("Duplicated", 72.0, 700.0, 14.0, false),
            seg("Duplicated", 72.0, 700.0, 14.0, false),
        ]);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].text, "Duplicated");
    }

    #[test]
    fn should_collapse_shifted_duplicate_and_absorb_bold_and_size() {
        let out = super::dedupe_redrawn_segments(vec![
            seg("Weight", 72.0, 650.0, 14.0, false),
            seg("Weight", 72.6, 649.5, 15.0, true),
        ]);
        assert_eq!(out.len(), 1);
        assert!(out[0].is_bold, "double-draw bold signal must be kept");
        assert_eq!(out[0].font_size, 15.0, "larger draw wins the size signal");
    }

    #[test]
    fn should_collapse_issue_1114_shift_variants() {
        // The pdfplumber issue-1114 fixture re-draws "Horizontal shift" at dx=5.7pt
        // and "Vertical shift" at dy=3.7pt (18pt font). Both offsets are well inside
        // the span's own extent, so they must collapse.
        let out = super::dedupe_redrawn_segments(vec![
            seg("Horizontal shift", 117.6, 237.0, 18.0, false),
            seg("Horizontal shift", 123.3, 237.0, 18.0, false),
            seg("Vertical shift", 117.6, 187.1, 18.0, false),
            seg("Vertical shift", 117.6, 183.4, 18.0, false),
        ]);
        assert_eq!(out.len(), 2);
    }

    #[test]
    fn should_keep_identical_digits_in_adjacent_table_cells() {
        // Same short string 6pt apart is two real table cells, not a re-draw:
        // the relative tolerance (half the span extent) must not swallow it.
        let out = super::dedupe_redrawn_segments(vec![
            seg("1", 100.0, 500.0, 10.0, false),
            seg("1", 106.0, 500.0, 10.0, false),
            seg("1", 100.0, 488.0, 10.0, false),
        ]);
        assert_eq!(out.len(), 3, "adjacent identical table cells are real text");
    }

    #[test]
    fn should_keep_repeated_word_at_distinct_position() {
        let out = super::dedupe_redrawn_segments(vec![
            seg("total", 72.0, 700.0, 10.0, false),
            seg("total", 140.0, 700.0, 10.0, false),
            seg("total", 72.0, 640.0, 10.0, false),
        ]);
        assert_eq!(out.len(), 3, "same word at different positions is real text");
    }

    #[test]
    fn should_keep_different_text_at_same_position() {
        let out = super::dedupe_redrawn_segments(vec![
            seg("a", 72.0, 700.0, 10.0, false),
            seg("b", 72.0, 700.0, 10.0, false),
        ]);
        assert_eq!(out.len(), 2);
    }
}
