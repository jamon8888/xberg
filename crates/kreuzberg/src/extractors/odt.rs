//! ODT (OpenDocument Text) extractor using native Rust parsing.
//!
//! Supports: OpenDocument Text (.odt)

use crate::Result;
use crate::core::config::ExtractionConfig;
use crate::extraction::{cells_to_markdown, office_metadata};
use crate::plugins::{DocumentExtractor, Plugin};
use crate::types::{ExtractionResult, Metadata, Table};
use ahash::AHashMap;
use async_trait::async_trait;
use memchr::memmem;
use roxmltree::Document;
use std::borrow::Cow;
use std::io::Cursor;

/// High-performance ODT extractor using native Rust XML parsing.
///
/// This extractor provides:
/// - Fast text extraction via roxmltree XML parsing
/// - Comprehensive metadata extraction from meta.xml
/// - Table extraction with row and cell support
/// - Formatting preservation (bold, italic, strikeout)
/// - Support for headings, paragraphs, and special elements
pub struct OdtExtractor;

impl OdtExtractor {
    /// Create a new ODT extractor.
    pub fn new() -> Self {
        Self
    }
}

impl Default for OdtExtractor {
    fn default() -> Self {
        Self::new()
    }
}

impl Plugin for OdtExtractor {
    fn name(&self) -> &str {
        "odt-extractor"
    }

    fn version(&self) -> String {
        env!("CARGO_PKG_VERSION").to_string()
    }

    fn initialize(&self) -> Result<()> {
        Ok(())
    }

    fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    fn description(&self) -> &str {
        "Native Rust ODT (OpenDocument Text) extractor with metadata and table support"
    }

    fn author(&self) -> &str {
        "Kreuzberg Team"
    }
}

/// Replace a word in a string only if it appears as a whole word
/// (not as a substring of a larger word).
///
/// Uses `memmem::Finder` for a single-pass search across all occurrences,
/// avoiding repeated `find()` calls that restart from the current position.
fn replace_whole_word(input: &str, word: &str, replacement: &str) -> String {
    let finder = memmem::Finder::new(word.as_bytes());
    let mut result = String::with_capacity(input.len());
    // Byte offset of the last character emitted into `result`.
    let mut last_end = 0;

    for start in finder.find_iter(input.as_bytes()) {
        let after_pos = start + word.len();

        // Check the Unicode character immediately before the match (whole-word boundary).
        let before_ok = if start == 0 {
            true
        } else {
            // SAFETY: start > 0 guarantees input[..start] is non-empty and contains
            // at least one UTF-8 character (memchr matched ASCII bytes, so the slice
            // boundary is always at a valid UTF-8 char boundary).
            let prev_char = input[..start].chars().next_back().unwrap_or(' ');
            !prev_char.is_alphanumeric()
        };

        // Check the Unicode character immediately after the match (whole-word boundary).
        let after_ok = if after_pos >= input.len() {
            true
        } else {
            // SAFETY: after_pos < input.len() guarantees input[after_pos..] is non-empty
            // and starts on a valid UTF-8 char boundary (word ends on ASCII bytes).
            let next_char = input[after_pos..].chars().next().unwrap_or(' ');
            !next_char.is_alphanumeric()
        };

        if before_ok && after_ok {
            // Emit everything from the last replacement end up to this match, then the replacement.
            result.push_str(&input[last_end..start]);
            result.push_str(replacement);
            last_end = after_pos;
        }
        // Non-whole-word matches: leave `last_end` unchanged; the bytes will be
        // included in the next emit or in the final trailing push below.
    }

    result.push_str(&input[last_end..]);
    result
}

/// Convert StarMath notation to Unicode text.
///
/// Handles common StarMath operators and superscript/subscript notation,
/// converting them to their Unicode equivalents.
fn starmath_to_unicode(formula: &str) -> String {
    let mut result = formula.to_string();

    // Replace StarMath operators with Unicode equivalents
    let replacements = [
        ("cdot", "\u{22C5}"),    // ⋅
        ("times", "\u{00D7}"),   // ×
        ("div", "\u{00F7}"),     // ÷
        ("pm", "\u{00B1}"),      // ±
        ("mp", "\u{2213}"),      // ∓
        ("le", "\u{2264}"),      // ≤
        ("ge", "\u{2265}"),      // ≥
        ("ne", "\u{2260}"),      // ≠
        ("approx", "\u{2248}"),  // ≈
        ("equiv", "\u{2261}"),   // ≡
        ("inf", "\u{221E}"),     // ∞
        ("partial", "\u{2202}"), // ∂
        ("nabla", "\u{2207}"),   // ∇
        ("sum", "\u{2211}"),     // ∑
        ("prod", "\u{220F}"),    // ∏
        ("int", "\u{222B}"),     // ∫
        ("sqrt", "\u{221A}"),    // √
        ("alpha", "\u{03B1}"),   // α
        ("beta", "\u{03B2}"),    // β
        ("gamma", "\u{03B3}"),   // γ
        ("delta", "\u{03B4}"),   // δ
        ("pi", "\u{03C0}"),      // π
        ("sigma", "\u{03C3}"),   // σ
        ("theta", "\u{03B8}"),   // θ
        ("lambda", "\u{03BB}"),  // λ
        ("mu", "\u{03BC}"),      // μ
        ("omega", "\u{03C9}"),   // ω
    ];

    for (from, to) in &replacements {
        // Replace only whole words (not substrings within words)
        result = replace_whole_word(&result, from, to);
    }

    // Handle superscripts: ^{...} or ^N (single digit)
    result = convert_superscripts(&result);
    // Handle subscripts: _{...} or _N (single digit)
    result = convert_subscripts(&result);

    result
}

/// Convert superscript notation (^2, ^{10}, etc.) to Unicode superscript characters.
fn convert_superscripts(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '^' {
            if chars.peek() == Some(&'{') {
                chars.next(); // consume '{'
                let mut content = String::new();
                for c in chars.by_ref() {
                    if c == '}' {
                        break;
                    }
                    content.push(c);
                }
                for c in content.chars() {
                    result.push(char_to_superscript(c));
                }
            } else if let Some(&next) = chars.peek() {
                chars.next();
                result.push(char_to_superscript(next));
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// Convert subscript notation (_2, _{10}, etc.) to Unicode subscript characters.
fn convert_subscripts(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '_' {
            if chars.peek() == Some(&'{') {
                chars.next(); // consume '{'
                let mut content = String::new();
                for c in chars.by_ref() {
                    if c == '}' {
                        break;
                    }
                    content.push(c);
                }
                for c in content.chars() {
                    result.push(char_to_subscript(c));
                }
            } else if let Some(&next) = chars.peek() {
                chars.next();
                result.push(char_to_subscript(next));
            } else {
                result.push(ch);
            }
        } else {
            result.push(ch);
        }
    }

    result
}

/// Convert a single character to its Unicode superscript equivalent.
fn char_to_superscript(c: char) -> char {
    match c {
        '0' => '\u{2070}',
        '1' => '\u{00B9}',
        '2' => '\u{00B2}',
        '3' => '\u{00B3}',
        '4' => '\u{2074}',
        '5' => '\u{2075}',
        '6' => '\u{2076}',
        '7' => '\u{2077}',
        '8' => '\u{2078}',
        '9' => '\u{2079}',
        '+' => '\u{207A}',
        '-' => '\u{207B}',
        '=' => '\u{207C}',
        '(' => '\u{207D}',
        ')' => '\u{207E}',
        'n' => '\u{207F}',
        'i' => '\u{2071}',
        _ => c,
    }
}

/// Convert a single character to its Unicode subscript equivalent.
fn char_to_subscript(c: char) -> char {
    match c {
        '0' => '\u{2080}',
        '1' => '\u{2081}',
        '2' => '\u{2082}',
        '3' => '\u{2083}',
        '4' => '\u{2084}',
        '5' => '\u{2085}',
        '6' => '\u{2086}',
        '7' => '\u{2087}',
        '8' => '\u{2088}',
        '9' => '\u{2089}',
        '+' => '\u{208A}',
        '-' => '\u{208B}',
        '=' => '\u{208C}',
        '(' => '\u{208D}',
        ')' => '\u{208E}',
        _ => c,
    }
}

/// Extract text from MathML formula element
///
/// # Arguments
/// * `math_node` - The math XML node
///
/// # Returns
/// * `Option<String>` - The extracted formula text
fn extract_mathml_text(math_node: roxmltree::Node) -> Option<String> {
    for node in math_node.descendants() {
        if node.tag_name().name() == "annotation"
            && let Some(encoding) = node.attribute("encoding")
            && encoding.contains("StarMath")
            && let Some(text) = node.text()
        {
            return Some(starmath_to_unicode(text));
        }
    }

    let mut formula_parts = Vec::new();
    for node in math_node.descendants() {
        match node.tag_name().name() {
            "mi" | "mo" | "mn" | "ms" | "mtext" => {
                if let Some(text) = node.text() {
                    formula_parts.push(text.to_string());
                }
            }
            _ => {}
        }
    }

    if !formula_parts.is_empty() {
        Some(formula_parts.join(" "))
    } else {
        None
    }
}

/// Resolved formatting properties for a text style.
#[derive(Default, Clone)]
struct OdtStyleProps {
    bold: bool,
    italic: bool,
    underline: bool,
    strikethrough: bool,
    color: Option<String>,
    font_size: Option<String>,
}

/// Build a map from style-name to resolved formatting properties.
///
/// Parses `<style:style>` elements from the `office:automatic-styles` section
/// of content.xml and resolves `style:text-properties` attributes.
fn build_style_map(root: roxmltree::Node) -> AHashMap<String, OdtStyleProps> {
    let mut styles = AHashMap::new();
    for child in root.children() {
        if child.tag_name().name() == "automatic-styles" || child.tag_name().name() == "styles" {
            for style_node in child.children() {
                if style_node.tag_name().name() == "style" {
                    if let Some(name) = style_node
                        .attribute(("urn:oasis:names:tc:opendocument:xmlns:style:1.0", "name"))
                        .or_else(|| style_node.attribute("style:name"))
                    {
                        let mut props = OdtStyleProps::default();
                        for prop_child in style_node.children() {
                            if prop_child.tag_name().name() == "text-properties" {
                                // Bold: fo:font-weight="bold"
                                if let Some(fw) = prop_child
                                    .attribute((
                                        "urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0",
                                        "font-weight",
                                    ))
                                    .or_else(|| prop_child.attribute("fo:font-weight"))
                                {
                                    props.bold = fw == "bold";
                                }
                                // Italic: fo:font-style="italic"
                                if let Some(fs) = prop_child
                                    .attribute((
                                        "urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0",
                                        "font-style",
                                    ))
                                    .or_else(|| prop_child.attribute("fo:font-style"))
                                {
                                    props.italic = fs == "italic";
                                }
                                // Underline: style:text-underline-style != "none"
                                if let Some(ul) = prop_child
                                    .attribute((
                                        "urn:oasis:names:tc:opendocument:xmlns:style:1.0",
                                        "text-underline-style",
                                    ))
                                    .or_else(|| prop_child.attribute("style:text-underline-style"))
                                {
                                    props.underline = ul != "none";
                                }
                                // Strikethrough: style:text-line-through-style != "none"
                                if let Some(st) = prop_child
                                    .attribute((
                                        "urn:oasis:names:tc:opendocument:xmlns:style:1.0",
                                        "text-line-through-style",
                                    ))
                                    .or_else(|| prop_child.attribute("style:text-line-through-style"))
                                {
                                    props.strikethrough = st != "none";
                                }
                                // Color: fo:color="#rrggbb"
                                if let Some(color) = prop_child
                                    .attribute(("urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0", "color"))
                                    .or_else(|| prop_child.attribute("fo:color"))
                                {
                                    if color != "#000000" {
                                        props.color = Some(color.to_string());
                                    }
                                }
                                // Font size: fo:font-size="12pt"
                                if let Some(size) = prop_child
                                    .attribute((
                                        "urn:oasis:names:tc:opendocument:xmlns:xsl-fo-compatible:1.0",
                                        "font-size",
                                    ))
                                    .or_else(|| prop_child.attribute("fo:font-size"))
                                {
                                    props.font_size = Some(size.to_string());
                                }
                            }
                        }
                        styles.insert(name.to_string(), props);
                    }
                }
            }
        }
    }
    styles
}

/// Build a `DocumentStructure` from ODT content.xml.
///
/// Walks the same XML tree as `process_document_elements` but emits structured
/// nodes through the `DocumentStructureBuilder`. Resolves styles to produce
/// text annotations (bold, italic, underline, color, font-size).
fn build_odt_document_structure(
    archive: &mut zip::ZipArchive<Cursor<Vec<u8>>>,
) -> crate::error::Result<crate::types::document_structure::DocumentStructure> {
    use crate::types::builder::DocumentStructureBuilder;

    let mut xml_content = String::new();

    match archive.by_name("content.xml") {
        Ok(mut file) => {
            use std::io::Read;
            file.read_to_string(&mut xml_content)
                .map_err(|e| crate::error::KreuzbergError::parsing(format!("Failed to read content.xml: {}", e)))?;
        }
        Err(_) => {
            return Ok(DocumentStructureBuilder::new().source_format("odt").build());
        }
    }

    let doc = Document::parse(&xml_content)
        .map_err(|e| crate::error::KreuzbergError::parsing(format!("Failed to parse content.xml: {}", e)))?;

    let root = doc.root_element();
    let style_map = build_style_map(root);
    let mut builder = DocumentStructureBuilder::new().source_format("odt");

    for body_child in root.children() {
        if body_child.tag_name().name() == "body" {
            for text_elem in body_child.children() {
                if text_elem.tag_name().name() == "text" {
                    build_structure_from_elements(text_elem, &mut builder, &style_map);
                }
            }
        }
    }

    // Extract headers/footers from styles.xml (master pages)
    extract_odt_headers_footers(archive, &mut builder);

    Ok(builder.build())
}

/// Extract headers and footers from styles.xml master-page elements.
fn extract_odt_headers_footers(
    archive: &mut zip::ZipArchive<Cursor<Vec<u8>>>,
    builder: &mut crate::types::builder::DocumentStructureBuilder,
) {
    let mut styles_xml = String::new();
    if let Ok(mut file) = archive.by_name("styles.xml") {
        use std::io::Read;
        if file.read_to_string(&mut styles_xml).is_err() {
            return;
        }
    } else {
        return;
    }

    let Ok(doc) = Document::parse(&styles_xml) else {
        return;
    };

    for node in doc.root_element().descendants() {
        match node.tag_name().name() {
            "header" => {
                if let Some(text) = extract_node_text(node) {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        builder.push_header(trimmed, None);
                    }
                }
            }
            "footer" => {
                if let Some(text) = extract_node_text(node) {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        builder.push_footer(trimmed, None);
                    }
                }
            }
            _ => {}
        }
    }
}

/// Collect text and annotations from an ODT paragraph/heading node's children.
///
/// Walks `<text:span>` children, resolves their `text:style-name` against the
/// style map, and produces byte-offset `TextAnnotation`s.
fn collect_odt_annotations(
    node: roxmltree::Node,
    style_map: &AHashMap<String, OdtStyleProps>,
) -> (String, Vec<crate::types::TextAnnotation>) {
    use crate::types::builder;
    use crate::types::document_structure::{AnnotationKind, TextAnnotation};

    let mut text = String::new();
    let mut annotations = Vec::new();

    for child in node.children() {
        match child.tag_name().name() {
            "span" => {
                let span_text = child.text().unwrap_or("");
                if span_text.is_empty() {
                    continue;
                }
                let start = text.len() as u32;
                text.push_str(span_text);
                let end = text.len() as u32;

                // Resolve style
                let style_name = child
                    .attribute(("urn:oasis:names:tc:opendocument:xmlns:text:1.0", "style-name"))
                    .or_else(|| child.attribute("text:style-name"));
                if let Some(name) = style_name {
                    if let Some(props) = style_map.get(name) {
                        if props.bold {
                            annotations.push(builder::bold(start, end));
                        }
                        if props.italic {
                            annotations.push(builder::italic(start, end));
                        }
                        if props.underline {
                            annotations.push(builder::underline(start, end));
                        }
                        if props.strikethrough {
                            annotations.push(builder::strikethrough(start, end));
                        }
                        if let Some(ref color) = props.color {
                            annotations.push(TextAnnotation {
                                start,
                                end,
                                kind: AnnotationKind::Color { value: color.clone() },
                            });
                        }
                        if let Some(ref size) = props.font_size {
                            annotations.push(TextAnnotation {
                                start,
                                end,
                                kind: AnnotationKind::FontSize { value: size.clone() },
                            });
                        }
                    }
                }
            }
            "tab" => {
                text.push('\t');
            }
            "line-break" => {
                text.push('\n');
            }
            "note" => {
                // Footnotes/endnotes: skip inline (handled separately)
            }
            "a" => {
                // Hyperlinks inside paragraphs
                let link_text = child.text().unwrap_or("");
                if !link_text.is_empty() {
                    let start = text.len() as u32;
                    text.push_str(link_text);
                    let end = text.len() as u32;
                    let url = child
                        .attribute(("http://www.w3.org/1999/xlink", "href"))
                        .or_else(|| child.attribute("xlink:href"))
                        .unwrap_or("");
                    if !url.is_empty() {
                        annotations.push(builder::link(start, end, url, None));
                    }
                }
            }
            _ => {
                if let Some(t) = child.text() {
                    text.push_str(t);
                }
            }
        }
    }

    // Fallback: if no children produced text, try the node's own text
    if text.is_empty() {
        if let Some(t) = node.text() {
            text = t.to_string();
        }
    }

    (text, annotations)
}

/// Recursively walk ODT XML elements and populate the `DocumentStructureBuilder`.
fn build_structure_from_elements(
    parent: roxmltree::Node,
    builder: &mut crate::types::builder::DocumentStructureBuilder,
    style_map: &AHashMap<String, OdtStyleProps>,
) {
    for node in parent.children() {
        match node.tag_name().name() {
            "h" => {
                let (text, _annotations) = collect_odt_annotations(node, style_map);
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    let level = node
                        .attribute(("urn:oasis:names:tc:opendocument:xmlns:text:1.0", "outline-level"))
                        .and_then(|v| v.parse::<u8>().ok())
                        .unwrap_or(1);
                    builder.push_heading(level, trimmed, None, None);
                }
            }
            "p" => {
                // Check for draw:frame images inside paragraphs
                let mut has_image = false;
                for child in node.children() {
                    if child.tag_name().name() == "frame" {
                        for frame_child in child.children() {
                            if frame_child.tag_name().name() == "image" {
                                let href = frame_child
                                    .attribute(("http://www.w3.org/1999/xlink", "href"))
                                    .or_else(|| frame_child.attribute("xlink:href"));
                                let description = child
                                    .attribute(("urn:oasis:names:tc:opendocument:xmlns:svg-compatible:1.0", "title"))
                                    .or_else(|| child.attribute("svg:title"));
                                builder.push_image(description.or(href), None, None, None);
                                has_image = true;
                            }
                        }
                    }
                }

                // Extract footnotes from this paragraph
                for child in node.descendants() {
                    if child.tag_name().name() == "note" {
                        // Find the note-body
                        for note_child in child.children() {
                            if note_child.tag_name().name() == "note-body" {
                                if let Some(note_text) = extract_node_text(note_child) {
                                    let trimmed = note_text.trim();
                                    if !trimmed.is_empty() {
                                        builder.push_footnote(trimmed, None);
                                    }
                                }
                            }
                        }
                    }
                }

                // Extract office:annotation comments
                for child in node.descendants() {
                    if child.tag_name().name() == "annotation" {
                        let mut annotation_text = String::new();
                        let mut annotation_author = None;
                        let mut annotation_date = None;
                        for ann_child in child.children() {
                            match ann_child.tag_name().name() {
                                "p" => {
                                    if let Some(t) = ann_child.text() {
                                        if !annotation_text.is_empty() {
                                            annotation_text.push(' ');
                                        }
                                        annotation_text.push_str(t);
                                    }
                                }
                                "creator" => {
                                    annotation_author = ann_child.text().map(|s| s.to_string());
                                }
                                "date" => {
                                    annotation_date = ann_child.text().map(|s| s.to_string());
                                }
                                _ => {}
                            }
                        }
                        if !annotation_text.is_empty() {
                            let mut attrs = AHashMap::new();
                            attrs.insert("type".to_string(), "comment".to_string());
                            attrs.insert("text".to_string(), annotation_text);
                            if let Some(author) = annotation_author {
                                attrs.insert("author".to_string(), author);
                            }
                            if let Some(date) = annotation_date {
                                attrs.insert("date".to_string(), date);
                            }
                            let idx = builder.push_paragraph("", vec![], None, None);
                            builder.set_attributes(idx, attrs);
                        }
                    }
                }

                let (text, annotations) = collect_odt_annotations(node, style_map);
                let trimmed = text.trim();
                if !trimmed.is_empty() {
                    builder.push_paragraph(trimmed, annotations, None, None);
                } else if !has_image {
                    // Only skip empty paragraphs if no image was emitted
                }
            }
            "table" => {
                let cells = extract_table_cells(node);
                if !cells.is_empty() {
                    builder.push_table_simple(&cells, None);
                }
            }
            "list" => {
                build_list_structure(node, builder);
            }
            "section" => {
                build_structure_from_elements(node, builder, style_map);
            }
            _ => {}
        }
    }
}

/// Extract table cells as `Vec<Vec<String>>` from an ODT table element.
fn extract_table_cells(table_node: roxmltree::Node) -> Vec<Vec<String>> {
    let mut rows = Vec::new();
    for row_node in table_node.children() {
        if row_node.tag_name().name() == "table-row" {
            let mut row_cells = Vec::new();
            for cell_node in row_node.children() {
                if cell_node.tag_name().name() == "table-cell" {
                    let cell_text = extract_node_text(cell_node).unwrap_or_default();
                    row_cells.push(cell_text.trim().to_string());
                }
            }
            if !row_cells.is_empty() {
                rows.push(row_cells);
            }
        }
    }
    rows
}

/// Build list structure from an ODT `text:list` element.
fn build_list_structure(list_node: roxmltree::Node, builder: &mut crate::types::builder::DocumentStructureBuilder) {
    let list_idx = builder.push_list(false, None);
    for item in list_node.children() {
        if item.tag_name().name() == "list-item" {
            for child in item.children() {
                match child.tag_name().name() {
                    "p" | "h" => {
                        if let Some(text) = extract_node_text(child) {
                            let trimmed = text.trim();
                            if !trimmed.is_empty() {
                                builder.push_list_item(list_idx, trimmed, None);
                            }
                        }
                    }
                    "list" => {
                        // Nested lists: add as separate list (builder doesn't support nested lists as children of list items)
                        build_list_structure(child, builder);
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Extract text from embedded formula objects
///
/// # Arguments
/// * `archive` - ZIP archive containing the ODT document
///
/// # Returns
/// * `String` - Extracted formula content from embedded objects
fn extract_embedded_formulas(archive: &mut zip::ZipArchive<Cursor<Vec<u8>>>) -> crate::error::Result<String> {
    use std::io::Read;
    let mut formula_parts = Vec::new();

    let file_names: Vec<String> = archive.file_names().map(|s| s.to_string()).collect();

    for file_name in file_names {
        if file_name.contains("Object")
            && file_name.ends_with("content.xml")
            && let Ok(mut file) = archive.by_name(&file_name)
        {
            let mut xml_content = String::new();
            if file.read_to_string(&mut xml_content).is_ok()
                && let Ok(doc) = Document::parse(&xml_content)
            {
                let root = doc.root_element();

                if root.tag_name().name() == "math" {
                    if let Some(formula_text) = extract_mathml_text(root) {
                        formula_parts.push(formula_text);
                    }
                } else {
                    for node in root.descendants() {
                        if node.tag_name().name() == "math"
                            && let Some(formula_text) = extract_mathml_text(node)
                        {
                            formula_parts.push(formula_text);
                        }
                    }
                }
            }
        }
    }

    Ok(formula_parts.join("\n"))
}

/// Extract text content from ODT content.xml
///
/// # Arguments
/// * `archive` - ZIP archive containing the ODT document
///
/// # Returns
/// * `String` - Extracted text content
fn extract_content_text(archive: &mut zip::ZipArchive<Cursor<Vec<u8>>>, plain: bool) -> crate::error::Result<String> {
    let mut xml_content = String::new();

    match archive.by_name("content.xml") {
        Ok(mut file) => {
            use std::io::Read;
            file.read_to_string(&mut xml_content)
                .map_err(|e| crate::error::KreuzbergError::parsing(format!("Failed to read content.xml: {}", e)))?;
        }
        Err(_) => {
            return Ok(String::new());
        }
    }

    let doc = Document::parse(&xml_content)
        .map_err(|e| crate::error::KreuzbergError::parsing(format!("Failed to parse content.xml: {}", e)))?;

    let root = doc.root_element();

    let mut text_parts: Vec<String> = Vec::new();

    for body_child in root.children() {
        if body_child.tag_name().name() == "body" {
            for text_elem in body_child.children() {
                if text_elem.tag_name().name() == "text" {
                    process_document_elements(text_elem, &mut text_parts, plain);
                }
            }
        }
    }

    Ok(text_parts.join("\n").trim().to_string())
}

/// Helper function to process document elements (paragraphs, headings, tables, lists)
/// Only processes direct children, avoiding nested content like table cells
fn process_document_elements(parent: roxmltree::Node, text_parts: &mut Vec<String>, plain: bool) {
    for node in parent.children() {
        match node.tag_name().name() {
            "h" => {
                if let Some(text) = extract_node_text(node)
                    && !text.trim().is_empty()
                {
                    if plain {
                        text_parts.push(text.trim().to_string());
                    } else {
                        text_parts.push(format!("# {}", text.trim()));
                    }
                    text_parts.push(String::new());
                }
            }
            "p" => {
                if let Some(text) = extract_node_text(node)
                    && !text.trim().is_empty()
                {
                    text_parts.push(text.trim().to_string());
                    text_parts.push(String::new());
                }
            }
            "table" => {
                if let Some(table_text) = extract_table_text(node, plain) {
                    text_parts.push(table_text);
                    text_parts.push(String::new());
                }
            }
            "list" => {
                process_list_elements(node, text_parts, 0, plain);
                text_parts.push(String::new());
            }
            "section" => {
                process_document_elements(node, text_parts, plain);
            }
            _ => {}
        }
    }
}

/// Process list elements recursively, handling nested lists with indentation
fn process_list_elements(list_node: roxmltree::Node, text_parts: &mut Vec<String>, depth: usize, plain: bool) {
    let indent = "  ".repeat(depth);
    for item in list_node.children() {
        if item.tag_name().name() == "list-item" {
            for child in item.children() {
                match child.tag_name().name() {
                    "p" => {
                        if let Some(text) = extract_node_text(child)
                            && !text.trim().is_empty()
                        {
                            if plain {
                                text_parts.push(text.trim().to_string());
                            } else {
                                text_parts.push(format!("{indent}- {}", text.trim()));
                            }
                        }
                    }
                    "h" => {
                        if let Some(text) = extract_node_text(child)
                            && !text.trim().is_empty()
                        {
                            if plain {
                                text_parts.push(text.trim().to_string());
                            } else {
                                text_parts.push(format!("{indent}- # {}", text.trim()));
                            }
                        }
                    }
                    "list" => {
                        process_list_elements(child, text_parts, depth + 1, plain);
                    }
                    _ => {}
                }
            }
        }
    }
}

/// Extract text from a single XML node, handling spans and formatting
///
/// # Arguments
/// * `node` - The XML node to extract text from
///
/// # Returns
/// * `Option<String>` - The extracted text with formatting preserved
fn extract_node_text(node: roxmltree::Node) -> Option<String> {
    let mut text_parts = Vec::new();

    for child in node.children() {
        match child.tag_name().name() {
            "span" => {
                if let Some(text) = child.text() {
                    text_parts.push(text.to_string());
                }
            }
            "tab" => {
                text_parts.push("\t".to_string());
            }
            "line-break" => {
                text_parts.push("\n".to_string());
            }
            _ => {
                if let Some(text) = child.text() {
                    text_parts.push(text.to_string());
                }
            }
        }
    }

    if text_parts.is_empty() {
        node.text().map(|s| s.to_string())
    } else {
        Some(text_parts.join(""))
    }
}

/// Extract table content as text with markdown formatting
///
/// # Arguments
/// * `table_node` - The table XML node
///
/// # Returns
/// * `Option<String>` - Markdown formatted table
fn extract_table_text(table_node: roxmltree::Node, plain: bool) -> Option<String> {
    let mut rows = Vec::new();
    let mut max_cols = 0;

    for row_node in table_node.children() {
        if row_node.tag_name().name() == "table-row" {
            let mut row_cells = Vec::new();

            for cell_node in row_node.children() {
                if cell_node.tag_name().name() == "table-cell" {
                    let cell_text = extract_node_text(cell_node).unwrap_or_default();
                    row_cells.push(cell_text.trim().to_string());
                }
            }

            if !row_cells.is_empty() {
                max_cols = max_cols.max(row_cells.len());
                rows.push(row_cells);
            }
        }
    }

    if rows.is_empty() {
        return None;
    }

    for row in &mut rows {
        while row.len() < max_cols {
            row.push(String::new());
        }
    }

    if plain {
        Some(crate::extraction::cells_to_text(&rows))
    } else {
        let mut markdown = String::new();

        if !rows.is_empty() {
            markdown.push('|');
            for cell in &rows[0] {
                markdown.push(' ');
                markdown.push_str(cell);
                markdown.push_str(" |");
            }
            markdown.push('\n');

            markdown.push('|');
            for _ in 0..rows[0].len() {
                markdown.push_str(" --- |");
            }
            markdown.push('\n');

            for row in rows.iter().skip(1) {
                markdown.push('|');
                for cell in row {
                    markdown.push(' ');
                    markdown.push_str(cell);
                    markdown.push_str(" |");
                }
                markdown.push('\n');
            }
        }

        Some(markdown)
    }
}

/// Extract tables from ODT content.xml
///
/// # Arguments
/// * `archive` - ZIP archive containing the ODT document
///
/// # Returns
/// * `Result<Vec<Table>>` - Extracted tables
fn extract_tables(archive: &mut zip::ZipArchive<Cursor<Vec<u8>>>) -> crate::error::Result<Vec<Table>> {
    let mut xml_content = String::new();

    match archive.by_name("content.xml") {
        Ok(mut file) => {
            use std::io::Read;
            file.read_to_string(&mut xml_content)
                .map_err(|e| crate::error::KreuzbergError::parsing(format!("Failed to read content.xml: {}", e)))?;
        }
        Err(_) => {
            return Ok(Vec::new());
        }
    }

    let doc = Document::parse(&xml_content)
        .map_err(|e| crate::error::KreuzbergError::parsing(format!("Failed to parse content.xml: {}", e)))?;

    let root = doc.root_element();
    let mut tables = Vec::new();
    let mut table_index = 0;

    for node in root.descendants() {
        if node.tag_name().name() == "table"
            && let Some(table) = parse_odt_table(node, table_index)
        {
            tables.push(table);
            table_index += 1;
        }
    }

    Ok(tables)
}

/// Parse a single ODT table element into a Table struct
///
/// # Arguments
/// * `table_node` - The table XML node
/// * `table_index` - Index of the table in the document
///
/// # Returns
/// * `Option<Table>` - Parsed table
fn parse_odt_table(table_node: roxmltree::Node, table_index: usize) -> Option<Table> {
    let mut cells: Vec<Vec<String>> = Vec::new();

    for row_node in table_node.children() {
        if row_node.tag_name().name() == "table-row" {
            let mut row_cells = Vec::new();

            for cell_node in row_node.children() {
                if cell_node.tag_name().name() == "table-cell" {
                    let cell_text = extract_node_text(cell_node).unwrap_or_default();
                    row_cells.push(cell_text.trim().to_string());
                }
            }

            if !row_cells.is_empty() {
                cells.push(row_cells);
            }
        }
    }

    if cells.is_empty() {
        return None;
    }

    let markdown = cells_to_markdown(&cells);

    Some(Table {
        cells,
        markdown,
        page_number: table_index + 1,
        bounding_box: None,
    })
}

#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl DocumentExtractor for OdtExtractor {
    #[cfg_attr(
        feature = "otel",
        tracing::instrument(
            skip(self, content, config),
            fields(
                extractor.name = self.name(),
                content.size_bytes = content.len(),
            )
        )
    )]
    async fn extract_bytes(
        &self,
        content: &[u8],
        mime_type: &str,
        config: &ExtractionConfig,
    ) -> Result<ExtractionResult> {
        let content_owned = content.to_vec();
        let plain = matches!(
            config.output_format,
            crate::core::config::OutputFormat::Plain | crate::core::config::OutputFormat::Structured
        );

        let (text, tables) = {
            #[cfg(feature = "tokio-runtime")]
            if crate::core::batch_mode::is_batch_mode() {
                let content_for_task = content_owned.clone();
                let span = tracing::Span::current();
                tokio::task::spawn_blocking(move || -> crate::error::Result<(String, Vec<Table>)> {
                    let _guard = span.entered();

                    let cursor = Cursor::new(content_for_task);
                    let mut archive = zip::ZipArchive::new(cursor).map_err(|e| {
                        crate::error::KreuzbergError::parsing(format!("Failed to open ZIP archive: {}", e))
                    })?;

                    let text = extract_content_text(&mut archive, plain)?;
                    let tables = extract_tables(&mut archive)?;
                    let embedded_formulas = extract_embedded_formulas(&mut archive)?;

                    let combined_text = if !embedded_formulas.is_empty() {
                        if !text.is_empty() {
                            format!("{}\n{}", text, embedded_formulas)
                        } else {
                            embedded_formulas
                        }
                    } else {
                        text
                    };

                    Ok((combined_text, tables))
                })
                .await
                .map_err(|e| crate::error::KreuzbergError::parsing(format!("ODT extraction task failed: {}", e)))??
            } else {
                let cursor = Cursor::new(content_owned.clone());
                let mut archive = zip::ZipArchive::new(cursor)
                    .map_err(|e| crate::error::KreuzbergError::parsing(format!("Failed to open ZIP archive: {}", e)))?;

                let text = extract_content_text(&mut archive, plain)?;
                let tables = extract_tables(&mut archive)?;
                let embedded_formulas = extract_embedded_formulas(&mut archive)?;

                let combined_text = if !embedded_formulas.is_empty() {
                    if !text.is_empty() {
                        format!("{}\n{}", text, embedded_formulas)
                    } else {
                        embedded_formulas
                    }
                } else {
                    text
                };

                (combined_text, tables)
            }

            #[cfg(not(feature = "tokio-runtime"))]
            {
                let cursor = Cursor::new(content_owned.clone());
                let mut archive = zip::ZipArchive::new(cursor)
                    .map_err(|e| crate::error::KreuzbergError::parsing(format!("Failed to open ZIP archive: {}", e)))?;

                let text = extract_content_text(&mut archive, plain)?;
                let tables = extract_tables(&mut archive)?;
                let embedded_formulas = extract_embedded_formulas(&mut archive)?;

                let combined_text = if !embedded_formulas.is_empty() {
                    if !text.is_empty() {
                        format!("{}\n{}", text, embedded_formulas)
                    } else {
                        embedded_formulas
                    }
                } else {
                    text
                };

                (combined_text, tables)
            }
        };

        let mut metadata_map = AHashMap::new();

        let cursor = Cursor::new(content_owned.clone());
        let mut archive = zip::ZipArchive::new(cursor).map_err(|e| {
            crate::error::KreuzbergError::parsing(format!("Failed to open ZIP archive for metadata: {}", e))
        })?;

        if let Ok(odt_props) = office_metadata::extract_odt_properties(&mut archive) {
            if let Some(title) = odt_props.title {
                metadata_map.insert(Cow::Borrowed("title"), serde_json::Value::String(title));
            }
            if let Some(creator) = odt_props.creator {
                metadata_map.insert(
                    Cow::Borrowed("authors"),
                    serde_json::Value::Array(vec![serde_json::Value::String(creator.clone())]),
                );
                metadata_map.insert(Cow::Borrowed("created_by"), serde_json::Value::String(creator));
            }
            if let Some(initial_creator) = odt_props.initial_creator {
                metadata_map.insert(
                    Cow::Borrowed("initial_creator"),
                    serde_json::Value::String(initial_creator),
                );
            }
            if let Some(subject) = odt_props.subject {
                metadata_map.insert(Cow::Borrowed("subject"), serde_json::Value::String(subject));
            }
            if let Some(keywords) = odt_props.keywords {
                metadata_map.insert(Cow::Borrowed("keywords"), serde_json::Value::String(keywords));
            }
            if let Some(description) = odt_props.description {
                metadata_map.insert(Cow::Borrowed("description"), serde_json::Value::String(description));
            }
            if let Some(creation_date) = odt_props.creation_date {
                metadata_map.insert(Cow::Borrowed("created_at"), serde_json::Value::String(creation_date));
            }
            if let Some(date) = odt_props.date {
                metadata_map.insert(Cow::Borrowed("modified_at"), serde_json::Value::String(date));
            }
            if let Some(language) = odt_props.language {
                metadata_map.insert(Cow::Borrowed("language"), serde_json::Value::String(language));
            }
            if let Some(generator) = odt_props.generator {
                metadata_map.insert(Cow::Borrowed("generator"), serde_json::Value::String(generator));
            }
            if let Some(editing_duration) = odt_props.editing_duration {
                metadata_map.insert(
                    Cow::Borrowed("editing_duration"),
                    serde_json::Value::String(editing_duration),
                );
            }
            if let Some(editing_cycles) = odt_props.editing_cycles {
                metadata_map.insert(
                    Cow::Borrowed("editing_cycles"),
                    serde_json::Value::String(editing_cycles),
                );
            }
            if let Some(page_count) = odt_props.page_count {
                metadata_map.insert(
                    Cow::Borrowed("page_count"),
                    serde_json::Value::Number(page_count.into()),
                );
            }
            if let Some(word_count) = odt_props.word_count {
                metadata_map.insert(
                    Cow::Borrowed("word_count"),
                    serde_json::Value::Number(word_count.into()),
                );
            }
            if let Some(character_count) = odt_props.character_count {
                metadata_map.insert(
                    Cow::Borrowed("character_count"),
                    serde_json::Value::Number(character_count.into()),
                );
            }
            if let Some(paragraph_count) = odt_props.paragraph_count {
                metadata_map.insert(
                    Cow::Borrowed("paragraph_count"),
                    serde_json::Value::Number(paragraph_count.into()),
                );
            }
            if let Some(table_count) = odt_props.table_count {
                metadata_map.insert(
                    Cow::Borrowed("table_count"),
                    serde_json::Value::Number(table_count.into()),
                );
            }
            if let Some(image_count) = odt_props.image_count {
                metadata_map.insert(
                    Cow::Borrowed("image_count"),
                    serde_json::Value::Number(image_count.into()),
                );
            }
        }

        let document = if config.include_document_structure {
            let cursor = Cursor::new(content_owned.clone());
            let mut doc_archive = zip::ZipArchive::new(cursor).map_err(|e| {
                crate::error::KreuzbergError::parsing(format!(
                    "Failed to open ZIP archive for document structure: {}",
                    e
                ))
            })?;
            Some(build_odt_document_structure(&mut doc_archive)?)
        } else {
            None
        };

        Ok(ExtractionResult {
            content: text,
            mime_type: mime_type.to_string().into(),
            metadata: Metadata {
                additional: metadata_map,
                ..Default::default()
            },
            pages: None,
            tables,
            detected_languages: None,
            chunks: None,
            images: None,
            djot_content: None,
            elements: None,
            ocr_elements: None,
            document,
            #[cfg(any(feature = "keywords-yake", feature = "keywords-rake"))]
            extracted_keywords: None,
            quality_score: None,
            processing_warnings: Vec::new(),
            annotations: None,
        })
    }

    fn supported_mime_types(&self) -> &[&str] {
        &["application/vnd.oasis.opendocument.text"]
    }

    fn priority(&self) -> i32 {
        60
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_odt_extractor_plugin_interface() {
        let extractor = OdtExtractor::new();
        assert_eq!(extractor.name(), "odt-extractor");
        assert_eq!(extractor.version(), env!("CARGO_PKG_VERSION"));
        assert_eq!(extractor.priority(), 60);
        assert_eq!(extractor.supported_mime_types().len(), 1);
    }

    #[tokio::test]
    async fn test_odt_extractor_supports_odt() {
        let extractor = OdtExtractor::new();
        assert!(
            extractor
                .supported_mime_types()
                .contains(&"application/vnd.oasis.opendocument.text")
        );
    }

    #[tokio::test]
    async fn test_odt_extractor_default() {
        let extractor = OdtExtractor;
        assert_eq!(extractor.name(), "odt-extractor");
    }

    #[tokio::test]
    async fn test_odt_extractor_initialize_shutdown() {
        let extractor = OdtExtractor::new();
        assert!(extractor.initialize().is_ok());
        assert!(extractor.shutdown().is_ok());
    }

    #[test]
    fn test_extract_node_text_simple() {
        let xml = r#"<p xmlns:text="urn:oasis:names:tc:opendocument:xmlns:text:1.0">Hello world</p>"#;
        let doc = roxmltree::Document::parse(xml).unwrap();
        let node = doc.root_element();

        let result = extract_node_text(node);
        assert!(result.is_some());
        assert!(!result.unwrap().is_empty());
    }

    /// Helper to load test ODT, extract with document structure, and return the structure.
    async fn extract_odt_with_structure(filename: &str) -> Option<crate::types::document_structure::DocumentStructure> {
        let test_file = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../test_documents/odt")
            .join(filename);
        if !test_file.exists() {
            return None;
        }
        let content = std::fs::read(&test_file).expect("Failed to read test ODT");
        let extractor = OdtExtractor::new();
        let mut config = ExtractionConfig::default();
        config.include_document_structure = true;
        let result = extractor
            .extract_bytes(&content, "application/vnd.oasis.opendocument.text", &config)
            .await
            .expect("ODT extraction failed");
        result.document
    }

    #[tokio::test]
    async fn test_odt_footnote_extraction() {
        let doc = extract_odt_with_structure("footnote.odt").await;
        let Some(doc) = doc else { return };
        // Should contain at least one Footnote node
        let has_footnote = doc.nodes.iter().any(|n| {
            matches!(
                n.content,
                crate::types::document_structure::NodeContent::Footnote { .. }
            )
        });
        assert!(
            has_footnote,
            "Footnote ODT should produce Footnote nodes in document structure"
        );
    }

    #[tokio::test]
    async fn test_odt_header_extraction() {
        let doc = extract_odt_with_structure("headers.odt").await;
        let Some(doc) = doc else { return };
        // Should contain at least one Header-layer node
        let has_header = doc
            .nodes
            .iter()
            .any(|n| n.content_layer == crate::types::document_structure::ContentLayer::Header);
        assert!(
            has_header,
            "Headers ODT should produce Header-layer nodes in document structure"
        );
    }

    #[tokio::test]
    async fn test_odt_image_extraction() {
        let doc = extract_odt_with_structure("imageWithCaption.odt").await;
        let Some(doc) = doc else { return };
        let has_image = doc
            .nodes
            .iter()
            .any(|n| matches!(n.content, crate::types::document_structure::NodeContent::Image { .. }));
        assert!(has_image, "Image ODT should produce Image nodes in document structure");
    }

    #[tokio::test]
    async fn test_odt_bold_annotations() {
        let doc = extract_odt_with_structure("bold.odt").await;
        let Some(doc) = doc else { return };
        let has_bold = doc.nodes.iter().any(|n| {
            n.annotations
                .iter()
                .any(|a| matches!(a.kind, crate::types::document_structure::AnnotationKind::Bold))
        });
        assert!(
            has_bold,
            "Bold ODT should produce Bold annotations in document structure"
        );
    }

    #[tokio::test]
    async fn test_odt_italic_annotations() {
        let doc = extract_odt_with_structure("italic.odt").await;
        let Some(doc) = doc else { return };
        let has_italic = doc.nodes.iter().any(|n| {
            n.annotations
                .iter()
                .any(|a| matches!(a.kind, crate::types::document_structure::AnnotationKind::Italic))
        });
        assert!(
            has_italic,
            "Italic ODT should produce Italic annotations in document structure"
        );
    }

    #[tokio::test]
    async fn test_odt_underline_annotations() {
        let doc = extract_odt_with_structure("strikeout.odt").await;
        let Some(doc) = doc else { return };
        // strikeout.odt should have strikethrough annotations
        let has_strikethrough = doc.nodes.iter().any(|n| {
            n.annotations
                .iter()
                .any(|a| matches!(a.kind, crate::types::document_structure::AnnotationKind::Strikethrough))
        });
        assert!(
            has_strikethrough,
            "Strikeout ODT should produce Strikethrough annotations"
        );
    }
}
