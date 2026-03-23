//! Core RTF parsing logic.

use crate::extractors::rtf::encoding::{decode_windows_1252, parse_hex_byte, parse_rtf_control_word};
use crate::extractors::rtf::formatting::normalize_whitespace;
use crate::extractors::rtf::images::extract_image_metadata;
use crate::extractors::rtf::tables::TableState;
use crate::types::Table;
use crate::types::TextAnnotation;
use crate::types::document_structure::AnnotationKind;

/// A formatting span tracked during RTF parsing.
#[derive(Debug, Clone)]
pub struct RtfFormattingSpan {
    /// Byte offset in the output text where this format starts.
    pub start: usize,
    /// Byte offset in the output text where this format ends.
    pub end: usize,
    /// Whether bold was active.
    pub bold: bool,
    /// Whether italic was active.
    pub italic: bool,
    /// Whether underline was active.
    pub underline: bool,
    /// Color index into the color table (0 = default/auto).
    pub color_index: u16,
}

/// RTF formatting metadata extracted alongside text.
pub struct RtfFormattingData {
    /// Formatting spans corresponding to text regions.
    pub spans: Vec<RtfFormattingSpan>,
    /// Color table entries (index 0 is auto/default).
    pub color_table: Vec<String>,
    /// Header text content (from \header groups).
    pub header_text: Option<String>,
    /// Footer text content (from \footer groups).
    pub footer_text: Option<String>,
    /// Hyperlink spans: (start_byte, end_byte, url).
    pub hyperlinks: Vec<(usize, usize, String)>,
}

/// Extract the color table from RTF content.
///
/// Looks for `{\colortbl ...}` and parses semicolon-delimited color entries.
/// Each entry is formatted as `\red{R}\green{G}\blue{B};`.
fn parse_rtf_color_table(content: &str) -> Vec<String> {
    let mut colors = Vec::new();
    // Find {\colortbl
    let Some(start) = content.find("{\\colortbl") else {
        return colors;
    };
    let rest = &content[start..];
    // Find the closing brace
    let mut depth = 0;
    let mut table_content = String::new();
    for ch in rest.chars() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            _ => {}
        }
        if depth > 0 {
            table_content.push(ch);
        }
    }
    // Remove the leading `{\colortbl` prefix
    let table_body = table_content.strip_prefix("{\\colortbl").unwrap_or(&table_content);

    // Split on semicolons
    for entry in table_body.split(';') {
        let entry = entry.trim();
        if entry.is_empty() {
            // Auto/default color entry
            colors.push(String::new());
            continue;
        }
        // Parse \red{N}\green{N}\blue{N}
        let mut r = 0u8;
        let mut g = 0u8;
        let mut b = 0u8;
        for part in entry.split('\\') {
            let part = part.trim();
            if let Some(val) = part.strip_prefix("red") {
                r = val.parse().unwrap_or(0);
            } else if let Some(val) = part.strip_prefix("green") {
                g = val.parse().unwrap_or(0);
            } else if let Some(val) = part.strip_prefix("blue") {
                b = val.parse().unwrap_or(0);
            }
        }
        colors.push(format!("#{r:02x}{g:02x}{b:02x}"));
    }
    colors
}

/// Extract formatting metadata from RTF content.
///
/// This performs a lightweight pass over the RTF to extract:
/// - Bold/italic/underline formatting state changes
/// - Color table and color references
/// - Header/footer text
/// - Hyperlink field instructions
pub fn extract_rtf_formatting(content: &str) -> RtfFormattingData {
    let color_table = parse_rtf_color_table(content);
    let mut spans = Vec::new();
    let mut hyperlinks = Vec::new();
    // Track formatting state
    let mut bold = false;
    let mut italic = false;
    let mut underline = false;
    let mut color_idx: u16 = 0;
    let mut text_offset: usize = 0;
    let mut span_start: usize = 0;

    // Track header/footer destinations
    let mut in_header = false;
    let mut in_footer = false;
    let mut header_depth: i32 = 0;
    let mut footer_depth: i32 = 0;
    let mut header_buf = String::new();
    let mut footer_buf = String::new();

    // Track HYPERLINK fields
    let mut in_fldinst = false;
    let mut fldinst_depth: i32 = 0;
    let mut fldinst_content = String::new();
    let mut in_fldrslt = false;
    let mut fldrslt_depth: i32 = 0;
    let mut fldrslt_start: usize = 0;
    let mut pending_hyperlink_url: Option<String> = None;

    let mut group_depth: i32 = 0;
    let mut skip_depth: i32 = 0;
    let mut chars = content.chars().peekable();
    let mut expect_destination = false;
    let mut ignorable_pending = false;

    // Subset of SKIP_DESTINATIONS -- we DON'T skip "field" or "fldinst" here
    // because we want to parse hyperlinks.
    let skip_dests = [
        "fonttbl",
        "stylesheet",
        "info",
        "listtable",
        "listoverridetable",
        "generator",
        "filetbl",
        "revtbl",
        "rsidtbl",
        "xmlnstbl",
        "mmathPr",
        "themedata",
        "colorschememapping",
        "datastore",
        "latentstyles",
        "datafield",
        "objdata",
        "objclass",
        "panose",
        "bkmkstart",
        "bkmkend",
        "wgrffmtfilter",
        "fcharset",
        "pgdsctbl",
        "colortbl",
        "pict",
    ];

    while let Some(ch) = chars.next() {
        match ch {
            '{' => {
                group_depth += 1;
                expect_destination = true;
            }
            '}' => {
                group_depth -= 1;
                expect_destination = false;
                ignorable_pending = false;
                if skip_depth > 0 && group_depth < skip_depth {
                    skip_depth = 0;
                }
                if in_header && group_depth < header_depth {
                    in_header = false;
                }
                if in_footer && group_depth < footer_depth {
                    in_footer = false;
                }
                if in_fldinst && group_depth < fldinst_depth {
                    in_fldinst = false;
                    // Parse the HYPERLINK URL from fldinst content
                    let trimmed = fldinst_content.trim();
                    if let Some(rest) = trimmed.strip_prefix("HYPERLINK") {
                        let url = rest.trim().trim_matches('"').trim().to_string();
                        if !url.is_empty() {
                            pending_hyperlink_url = Some(url);
                        }
                    }
                    fldinst_content.clear();
                }
                if in_fldrslt && group_depth < fldrslt_depth {
                    in_fldrslt = false;
                    if let Some(url) = pending_hyperlink_url.take() {
                        hyperlinks.push((fldrslt_start, text_offset, url));
                    }
                }
            }
            '\\' => {
                if let Some(&next_ch) = chars.peek() {
                    match next_ch {
                        '\\' | '{' | '}' => {
                            chars.next();
                            expect_destination = false;
                            if skip_depth > 0 {
                                continue;
                            }
                            if in_fldinst {
                                fldinst_content.push(next_ch);
                                continue;
                            }
                            text_offset += next_ch.len_utf8();
                            if in_header {
                                header_buf.push(next_ch);
                            }
                            if in_footer {
                                footer_buf.push(next_ch);
                            }
                        }
                        '\'' => {
                            chars.next();
                            expect_destination = false;
                            let _h1 = chars.next();
                            let _h2 = chars.next();
                            if skip_depth > 0 {
                                continue;
                            }
                            // Count 1 byte for the decoded char
                            text_offset += 1;
                        }
                        '*' => {
                            chars.next();
                            ignorable_pending = true;
                        }
                        _ => {
                            let (word, param) = parse_rtf_control_word(&mut chars);

                            if expect_destination || ignorable_pending {
                                expect_destination = false;

                                if ignorable_pending {
                                    ignorable_pending = false;
                                    if skip_depth == 0 {
                                        skip_depth = group_depth;
                                    }
                                    continue;
                                }

                                // Handle special destinations
                                match word.as_str() {
                                    "fldinst" => {
                                        in_fldinst = true;
                                        fldinst_depth = group_depth;
                                        continue;
                                    }
                                    "fldrslt" => {
                                        in_fldrslt = true;
                                        fldrslt_depth = group_depth;
                                        fldrslt_start = text_offset;
                                        continue;
                                    }
                                    _ => {}
                                }

                                if skip_dests.contains(&word.as_str()) {
                                    if skip_depth == 0 {
                                        skip_depth = group_depth;
                                    }
                                    continue;
                                }
                            }

                            if skip_depth > 0 {
                                continue;
                            }
                            if in_fldinst {
                                fldinst_content.push_str(&word);
                                continue;
                            }

                            match word.as_str() {
                                "b" => {
                                    let new_bold = param.unwrap_or(1) != 0;
                                    if new_bold != bold {
                                        // Close previous span
                                        if text_offset > span_start {
                                            spans.push(RtfFormattingSpan {
                                                start: span_start,
                                                end: text_offset,
                                                bold,
                                                italic,
                                                underline,
                                                color_index: color_idx,
                                            });
                                        }
                                        span_start = text_offset;
                                        bold = new_bold;
                                    }
                                }
                                "i" => {
                                    let new_italic = param.unwrap_or(1) != 0;
                                    if new_italic != italic {
                                        if text_offset > span_start {
                                            spans.push(RtfFormattingSpan {
                                                start: span_start,
                                                end: text_offset,
                                                bold,
                                                italic,
                                                underline,
                                                color_index: color_idx,
                                            });
                                        }
                                        span_start = text_offset;
                                        italic = new_italic;
                                    }
                                }
                                "ul" => {
                                    let new_ul = param.unwrap_or(1) != 0;
                                    if new_ul != underline {
                                        if text_offset > span_start {
                                            spans.push(RtfFormattingSpan {
                                                start: span_start,
                                                end: text_offset,
                                                bold,
                                                italic,
                                                underline,
                                                color_index: color_idx,
                                            });
                                        }
                                        span_start = text_offset;
                                        underline = new_ul;
                                    }
                                }
                                "ulnone" => {
                                    if underline {
                                        if text_offset > span_start {
                                            spans.push(RtfFormattingSpan {
                                                start: span_start,
                                                end: text_offset,
                                                bold,
                                                italic,
                                                underline,
                                                color_index: color_idx,
                                            });
                                        }
                                        span_start = text_offset;
                                        underline = false;
                                    }
                                }
                                "cf" => {
                                    let new_idx = param.unwrap_or(0) as u16;
                                    if new_idx != color_idx {
                                        if text_offset > span_start {
                                            spans.push(RtfFormattingSpan {
                                                start: span_start,
                                                end: text_offset,
                                                bold,
                                                italic,
                                                underline,
                                                color_index: color_idx,
                                            });
                                        }
                                        span_start = text_offset;
                                        color_idx = new_idx;
                                    }
                                }
                                "plain" => {
                                    // Reset formatting
                                    if bold || italic || underline || color_idx != 0 {
                                        if text_offset > span_start {
                                            spans.push(RtfFormattingSpan {
                                                start: span_start,
                                                end: text_offset,
                                                bold,
                                                italic,
                                                underline,
                                                color_index: color_idx,
                                            });
                                        }
                                        span_start = text_offset;
                                        bold = false;
                                        italic = false;
                                        underline = false;
                                        color_idx = 0;
                                    }
                                }
                                "header" | "headerl" | "headerr" | "headerf" => {
                                    in_header = true;
                                    header_depth = group_depth;
                                }
                                "footer" | "footerl" | "footerr" | "footerf" => {
                                    in_footer = true;
                                    footer_depth = group_depth;
                                }
                                "par" | "line" => {
                                    text_offset += 1; // newline
                                    if in_header {
                                        header_buf.push('\n');
                                    }
                                    if in_footer {
                                        footer_buf.push('\n');
                                    }
                                }
                                "u" => {
                                    // Unicode char
                                    if let Some(code_num) = param {
                                        let code_u = if code_num < 0 {
                                            (code_num + 65536) as u32
                                        } else {
                                            code_num as u32
                                        };
                                        if let Some(c) = char::from_u32(code_u) {
                                            text_offset += c.len_utf8();
                                            if in_header {
                                                header_buf.push(c);
                                            }
                                            if in_footer {
                                                footer_buf.push(c);
                                            }
                                        }
                                    }
                                    // Skip replacement char
                                    if let Some(&next) = chars.peek()
                                        && next != '\\'
                                        && next != '{'
                                        && next != '}'
                                    {
                                        chars.next();
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
            '\n' | '\r' => {}
            _ => {
                if skip_depth > 0 || in_fldinst {
                    continue;
                }
                text_offset += ch.len_utf8();
                if in_header {
                    header_buf.push(ch);
                }
                if in_footer {
                    footer_buf.push(ch);
                }
            }
        }
    }

    // Close final span
    if text_offset > span_start && (bold || italic || underline || color_idx != 0) {
        spans.push(RtfFormattingSpan {
            start: span_start,
            end: text_offset,
            bold,
            italic,
            underline,
            color_index: color_idx,
        });
    }

    let header_trimmed = header_buf.trim().to_string();
    let footer_trimmed = footer_buf.trim().to_string();

    RtfFormattingData {
        spans,
        color_table,
        header_text: if header_trimmed.is_empty() {
            None
        } else {
            Some(header_trimmed)
        },
        footer_text: if footer_trimmed.is_empty() {
            None
        } else {
            Some(footer_trimmed)
        },
        hyperlinks,
    }
}

/// Convert RTF formatting spans into `TextAnnotation` vectors for a paragraph.
///
/// Given the byte range of a paragraph within the full extracted text,
/// produces annotations from the formatting spans that overlap.
pub fn spans_to_annotations(para_start: usize, para_end: usize, formatting: &RtfFormattingData) -> Vec<TextAnnotation> {
    let mut annotations = Vec::new();
    for span in &formatting.spans {
        // Check overlap
        if span.end <= para_start || span.start >= para_end {
            continue;
        }
        let ann_start = span.start.max(para_start) - para_start;
        let ann_end = span.end.min(para_end) - para_start;
        if ann_start >= ann_end {
            continue;
        }
        let s = ann_start as u32;
        let e = ann_end as u32;
        if span.bold {
            annotations.push(TextAnnotation {
                start: s,
                end: e,
                kind: AnnotationKind::Bold,
            });
        }
        if span.italic {
            annotations.push(TextAnnotation {
                start: s,
                end: e,
                kind: AnnotationKind::Italic,
            });
        }
        if span.underline {
            annotations.push(TextAnnotation {
                start: s,
                end: e,
                kind: AnnotationKind::Underline,
            });
        }
        if span.color_index > 0 {
            if let Some(color) = formatting.color_table.get(span.color_index as usize) {
                if !color.is_empty() && color != "#000000" {
                    annotations.push(TextAnnotation {
                        start: s,
                        end: e,
                        kind: AnnotationKind::Color { value: color.clone() },
                    });
                }
            }
        }
    }

    // Add hyperlink annotations
    for (link_start, link_end, url) in &formatting.hyperlinks {
        if *link_end <= para_start || *link_start >= para_end {
            continue;
        }
        let s = (link_start.max(&para_start) - para_start) as u32;
        let e = (link_end.min(&para_end) - para_start) as u32;
        if s < e {
            annotations.push(TextAnnotation {
                start: s,
                end: e,
                kind: AnnotationKind::Link {
                    url: url.clone(),
                    title: None,
                },
            });
        }
    }

    annotations
}

/// Known RTF destination groups whose content should be skipped entirely.
///
/// These are groups that start with a control word and contain metadata,
/// font tables, style sheets, or binary data — not document body text.
const SKIP_DESTINATIONS: &[&str] = &[
    "fonttbl",
    "colortbl",
    "stylesheet",
    "info",
    "listtable",
    "listoverridetable",
    "generator",
    "filetbl",
    "revtbl",
    "rsidtbl",
    "xmlnstbl",
    "mmathPr",
    "themedata",
    "colorschememapping",
    "datastore",
    "latentstyles",
    "datafield",
    "fldinst",
    "objdata",
    "objclass",
    "panose",
    "bkmkstart",
    "bkmkend",
    "field",
    "wgrffmtfilter",
    "fcharset",
    "pgdsctbl",
];

/// Extract text and image metadata from RTF document.
///
/// This function extracts plain text from an RTF document by:
/// 1. Tracking group nesting depth with a state stack
/// 2. Skipping known destination groups (fonttbl, stylesheet, info, etc.)
/// 3. Skipping `{\*\...}` ignorable destination groups
/// 4. Converting encoded characters to Unicode
/// 5. Extracting text while skipping formatting groups
/// 6. Detecting and extracting image metadata (\pict sections)
/// 7. Normalizing whitespace
pub fn extract_text_from_rtf(content: &str, plain: bool) -> (String, Vec<Table>) {
    let mut result = String::new();
    let mut chars = content.chars().peekable();
    let mut tables: Vec<Table> = Vec::new();
    let mut table_state: Option<TableState> = None;

    // Group state stack: each entry tracks whether the group should be skipped.
    // When skip_depth > 0, all content is suppressed until we return to the
    // enclosing depth.
    let mut group_depth: i32 = 0;
    let mut skip_depth: i32 = 0; // 0 = not skipping; >0 = skip until depth drops below this

    // Track whether the next group is an ignorable destination (\*)
    let mut ignorable_pending = false;
    // Track whether we just entered a new group and the first control word decides skip
    let mut expect_destination = false;

    let ensure_table = |table_state: &mut Option<TableState>| {
        if table_state.is_none() {
            *table_state = Some(TableState::new());
        }
    };

    let finalize_table = move |state_opt: &mut Option<TableState>, tables: &mut Vec<Table>| {
        if let Some(state) = state_opt.take()
            && let Some(table) = state.finalize_with_format(plain)
        {
            tables.push(table);
        }
    };

    while let Some(ch) = chars.next() {
        match ch {
            '{' => {
                group_depth += 1;
                expect_destination = true;
                // If we're already skipping, just track depth
            }
            '}' => {
                group_depth -= 1;
                expect_destination = false;
                ignorable_pending = false;
                // If we were skipping and just exited the skipped group, stop skipping
                if skip_depth > 0 && group_depth < skip_depth {
                    skip_depth = 0;
                }
                // Add space at group boundary (only when not skipping)
                if skip_depth == 0 && !result.is_empty() && !result.ends_with(' ') && !result.ends_with('\n') {
                    result.push(' ');
                }
            }
            '\\' => {
                if let Some(&next_ch) = chars.peek() {
                    match next_ch {
                        '\\' | '{' | '}' => {
                            chars.next();
                            expect_destination = false;
                            if skip_depth > 0 {
                                continue;
                            }
                            result.push(next_ch);
                        }
                        '\'' => {
                            chars.next();
                            expect_destination = false;
                            let hex1 = chars.next();
                            let hex2 = chars.next();
                            if skip_depth > 0 {
                                continue;
                            }
                            if let (Some(h1), Some(h2)) = (hex1, hex2)
                                && let Some(byte) = parse_hex_byte(h1, h2)
                            {
                                let decoded = decode_windows_1252(byte);
                                result.push(decoded);
                                if let Some(state) = table_state.as_mut()
                                    && state.in_row
                                {
                                    state.current_cell.push(decoded);
                                }
                            }
                        }
                        '*' => {
                            chars.next();
                            // \* marks an ignorable destination — skip the entire group
                            // if we don't recognize the keyword
                            ignorable_pending = true;
                        }
                        _ => {
                            let (control_word, _param) = parse_rtf_control_word(&mut chars);

                            // Check if this control word starts a destination to skip
                            if expect_destination || ignorable_pending {
                                expect_destination = false;

                                if ignorable_pending {
                                    // \* destination: skip entire group unless we specifically handle it
                                    ignorable_pending = false;
                                    if skip_depth == 0 {
                                        skip_depth = group_depth;
                                    }
                                    continue;
                                }

                                if SKIP_DESTINATIONS.contains(&control_word.as_str()) {
                                    if skip_depth == 0 {
                                        skip_depth = group_depth;
                                    }
                                    continue;
                                }
                            }

                            if skip_depth > 0 {
                                continue;
                            }

                            handle_control_word(
                                &control_word,
                                _param,
                                &mut chars,
                                &mut result,
                                &mut table_state,
                                &mut tables,
                                &ensure_table,
                                &finalize_table,
                                plain,
                            );
                        }
                    }
                }
            }
            '\n' | '\r' => {
                // RTF line breaks in the source are not significant
            }
            ' ' | '\t' => {
                if skip_depth > 0 {
                    continue;
                }
                if !result.is_empty() && !result.ends_with(' ') && !result.ends_with('\n') {
                    result.push(' ');
                }
                if let Some(state) = table_state.as_mut()
                    && state.in_row
                    && !state.current_cell.ends_with(' ')
                {
                    state.current_cell.push(' ');
                }
            }
            _ => {
                expect_destination = false;
                if skip_depth > 0 {
                    continue;
                }
                if let Some(state) = table_state.as_ref()
                    && !state.in_row
                    && !state.rows.is_empty()
                {
                    finalize_table(&mut table_state, &mut tables);
                }
                result.push(ch);
                if let Some(state) = table_state.as_mut()
                    && state.in_row
                {
                    state.current_cell.push(ch);
                }
            }
        }
    }

    if table_state.is_some() {
        finalize_table(&mut table_state, &mut tables);
    }

    (normalize_whitespace(&result), tables)
}

/// Handle an RTF control word during parsing.
#[allow(clippy::too_many_arguments)]
fn handle_control_word(
    control_word: &str,
    param: Option<i32>,
    chars: &mut std::iter::Peekable<std::str::Chars>,
    result: &mut String,
    table_state: &mut Option<TableState>,
    tables: &mut Vec<Table>,
    ensure_table: &dyn Fn(&mut Option<TableState>),
    finalize_table: &dyn Fn(&mut Option<TableState>, &mut Vec<Table>),
    plain: bool,
) {
    match control_word {
        // Unicode escape: \u1234 (signed integer)
        "u" => {
            if let Some(code_num) = param {
                let code_u = if code_num < 0 {
                    (code_num + 65536) as u32
                } else {
                    code_num as u32
                };
                if let Some(c) = char::from_u32(code_u) {
                    result.push(c);
                    if let Some(state) = table_state.as_mut()
                        && state.in_row
                    {
                        state.current_cell.push(c);
                    }
                }
                // Skip the replacement character (usually `?` or next byte)
                if let Some(&next) = chars.peek()
                    && next != '\\'
                    && next != '{'
                    && next != '}'
                {
                    chars.next();
                }
            }
        }
        "pict" => {
            let image_metadata = extract_image_metadata(chars);
            if !image_metadata.is_empty() && !plain {
                result.push('!');
                result.push('[');
                result.push_str("image");
                result.push(']');
                result.push('(');
                result.push_str(&image_metadata);
                result.push(')');
                result.push(' ');
                if let Some(state) = table_state.as_mut()
                    && state.in_row
                {
                    state.current_cell.push('!');
                    state.current_cell.push('[');
                    state.current_cell.push_str("image");
                    state.current_cell.push(']');
                    state.current_cell.push('(');
                    state.current_cell.push_str(&image_metadata);
                    state.current_cell.push(')');
                    state.current_cell.push(' ');
                }
            }
        }
        "par" | "line" => {
            if table_state.is_some() {
                finalize_table(table_state, tables);
            }
            if !result.is_empty() && !result.ends_with('\n') {
                result.push('\n');
                result.push('\n');
            }
        }
        "tab" => {
            result.push('\t');
            if let Some(state) = table_state.as_mut()
                && state.in_row
            {
                state.current_cell.push('\t');
            }
        }
        "bullet" => {
            result.push('\u{2022}');
        }
        "lquote" => {
            result.push('\u{2018}');
        }
        "rquote" => {
            result.push('\u{2019}');
        }
        "ldblquote" => {
            result.push('\u{201C}');
        }
        "rdblquote" => {
            result.push('\u{201D}');
        }
        "endash" => {
            result.push('\u{2013}');
        }
        "emdash" => {
            result.push('\u{2014}');
        }
        "trowd" => {
            ensure_table(table_state);
            if let Some(state) = table_state.as_mut() {
                state.start_row();
            }
            if !result.is_empty() && !result.ends_with('\n') {
                result.push('\n');
            }
            if !plain && !result.ends_with('|') {
                result.push('|');
                result.push(' ');
            }
        }
        "cell" => {
            if let Some(state) = table_state.as_mut()
                && state.in_row
            {
                state.push_cell();
            }
            if plain {
                // In plain mode, separate cells with pipes in the result string
                if !result.ends_with('|') && !result.ends_with('\n') && !result.is_empty() {
                    result.push('|');
                }
            } else {
                if !result.ends_with('|') {
                    if !result.ends_with(' ') && !result.is_empty() {
                        result.push(' ');
                    }
                    result.push('|');
                }
                if !result.ends_with(' ') {
                    result.push(' ');
                }
            }
        }
        "row" => {
            ensure_table(table_state);
            if let Some(state) = table_state.as_mut()
                && (state.in_row || !state.current_cell.is_empty())
            {
                state.push_row();
            }
            if !plain && !result.ends_with('|') {
                result.push('|');
            }
            if !result.ends_with('\n') {
                result.push('\n');
            }
        }
        _ => {}
    }
}
