//! Native Rust reStructuredText (RST) extractor.
//!
//! This extractor provides comprehensive RST document parsing.
//! It extracts:
//! - Document title and headings
//! - Field list metadata (:Author:, :Date:, :Version:, etc.)
//! - Paragraphs and text content
//! - Code blocks with language specifications
//! - Lists (bullet, numbered, definition lists)
//! - Tables (both simple and grid tables)
//! - Directives (image, code-block, note, math, etc.)
//! - Inline markup (emphasis, strong, code, links)
//! - Images and references

#[cfg(feature = "office")]
use crate::Result;
#[cfg(feature = "office")]
use crate::core::config::ExtractionConfig;
#[cfg(feature = "office")]
use crate::plugins::{DocumentExtractor, Plugin};
#[cfg(feature = "office")]
use crate::types::builder::DocumentStructureBuilder;
#[cfg(feature = "office")]
use crate::types::document_structure::{AnnotationKind, DocumentStructure, TextAnnotation};
#[cfg(feature = "office")]
use crate::types::{ExtractionResult, Metadata, Table};
#[cfg(feature = "office")]
use ahash::AHashMap;
#[cfg(feature = "office")]
use async_trait::async_trait;
#[cfg(feature = "office")]
use std::borrow::Cow;

/// Native Rust reStructuredText extractor.
///
/// Parses RST documents using document tree parsing and extracts:
/// - Metadata from field lists
/// - Document structure (headings, sections)
/// - Text content and inline formatting
/// - Code blocks and directives
/// - Tables and lists
#[cfg(feature = "office")]
pub struct RstExtractor;

#[cfg(feature = "office")]
impl RstExtractor {
    /// Create a new RST extractor.
    pub fn new() -> Self {
        Self
    }

    /// Extract text content and metadata from RST document.
    ///
    /// Uses document tree parsing and fallback text extraction.
    fn extract_text_and_metadata(content: &str) -> (String, Metadata) {
        let mut metadata = Metadata::default();
        let mut additional: AHashMap<Cow<'static, str>, serde_json::Value> = AHashMap::new();

        let text = Self::extract_text_from_rst(content, &mut additional);

        metadata.additional = additional;
        (text, metadata)
    }

    /// Extract text and metadata from RST content.
    ///
    /// This is the main extraction engine that processes RST line-by-line
    /// and extracts all document content including headings, code blocks, lists, etc.
    fn extract_text_from_rst(content: &str, metadata: &mut AHashMap<Cow<'static, str>, serde_json::Value>) -> String {
        let mut output = String::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];

            if line.trim().starts_with(':')
                && line.contains(':')
                && let Some((key, value)) = Self::parse_field_list_line(line)
            {
                // Collect continuation lines (indented lines that follow)
                let mut full_value = value.clone();
                while i + 1 < lines.len() {
                    let next = lines[i + 1];
                    if !next.is_empty() && (next.starts_with("   ") || next.starts_with("\t")) {
                        full_value.push('\n');
                        full_value.push_str(next);
                        i += 1;
                    } else {
                        break;
                    }
                }
                Self::add_metadata_field(&key, &full_value, metadata);
                // Output the field list in preserved format
                output.push_str(&format!(":{}: {}\n", key, full_value));
                i += 1;
                continue;
            }

            if i + 1 < lines.len() {
                let next_line = lines[i + 1];
                if Self::is_section_underline(next_line) && !line.trim().is_empty() {
                    output.push_str(line.trim());
                    output.push('\n');
                    i += 2;
                    continue;
                }
            }

            if line.trim().starts_with(".. code-block::") || line.trim().starts_with(".. code::") {
                // Preserve the directive line
                output.push_str(line.trim());
                output.push('\n');
                i += 1;
                // Preserve empty line after directive
                while i < lines.len() && lines[i].trim().is_empty() {
                    output.push('\n');
                    i += 1;
                }
                // Preserve indented content
                while i < lines.len() && (lines[i].starts_with("   ") || lines[i].is_empty()) {
                    output.push_str(lines[i]);
                    output.push('\n');
                    i += 1;
                }
                continue;
            }

            if line.trim().starts_with(".. highlight::") {
                let lang = line.trim_start_matches(".. highlight::").trim().to_string();
                if !lang.is_empty() {
                    output.push_str("highlight: ");
                    output.push_str(&lang);
                    output.push('\n');
                }
                i += 1;
                continue;
            }

            if line.trim().ends_with("::") && !line.trim().starts_with(".. ") {
                if let Some(display_text) = line.strip_suffix("::")
                    && !display_text.trim().is_empty()
                {
                    output.push_str(display_text.trim());
                    output.push('\n');
                }
                i += 1;
                while i < lines.len() && (lines[i].starts_with("    ") || lines[i].is_empty()) {
                    if !lines[i].is_empty() {
                        output.push_str(lines[i].trim_start());
                        output.push('\n');
                    }
                    i += 1;
                }
                continue;
            }

            if Self::is_list_item(line) {
                output.push_str(line.trim());
                output.push('\n');
                i += 1;
                continue;
            }

            if line.trim().starts_with(".. ") || line.trim() == ".." {
                let trimmed = line.trim();
                let directive = if trimmed == ".." { "" } else { &trimmed[3..] };

                if directive.starts_with("image::") {
                    let uri = directive.strip_prefix("image::").unwrap_or("").trim();
                    output.push_str("image: ");
                    output.push_str(uri);
                    output.push('\n');
                    i += 1;
                    continue;
                }

                if directive.starts_with("note::")
                    || directive.starts_with("warning::")
                    || directive.starts_with("important::")
                    || directive.starts_with("caution::")
                    || directive.starts_with("hint::")
                    || directive.starts_with("tip::")
                {
                    // Preserve the directive marker
                    output.push_str(trimmed);
                    output.push('\n');
                    i += 1;
                    while i < lines.len() && (lines[i].starts_with("   ") || lines[i].is_empty()) {
                        if !lines[i].is_empty() {
                            output.push_str(lines[i]);
                            output.push('\n');
                        }
                        i += 1;
                    }
                    continue;
                }

                if directive.starts_with("math::") {
                    let math = directive.strip_prefix("math::").unwrap_or("").trim();
                    if !math.is_empty() {
                        output.push_str("math: ");
                        output.push_str(math);
                        output.push('\n');
                    }
                    i += 1;
                    while i < lines.len() && (lines[i].starts_with("   ") || lines[i].is_empty()) {
                        if !lines[i].is_empty() {
                            output.push_str(lines[i].trim());
                            output.push('\n');
                        }
                        i += 1;
                    }
                    continue;
                }

                i += 1;
                while i < lines.len() && (lines[i].starts_with("   ") || lines[i].is_empty()) {
                    i += 1;
                }
                continue;
            }

            if !line.trim().is_empty() && !Self::is_markup_line(line) {
                output.push_str(line);
                output.push('\n');
            }

            i += 1;
        }

        output
    }

    /// Parse a field list line (e.g., ":Author: John Doe")
    fn parse_field_list_line(line: &str) -> Option<(String, String)> {
        let trimmed = line.trim();
        if !trimmed.starts_with(':') {
            return None;
        }

        let rest = &trimmed[1..];
        if let Some(end_pos) = rest.find(':') {
            let key = rest[..end_pos].to_string();
            let value = rest[end_pos + 1..].trim().to_string();
            return Some((key, value));
        }

        None
    }

    /// Add a metadata field from RST field list.
    fn add_metadata_field(key: &str, value: &str, metadata: &mut AHashMap<Cow<'static, str>, serde_json::Value>) {
        let key_lower = key.to_lowercase();
        match key_lower.as_str() {
            "author" | "authors" => {
                metadata.insert(Cow::Borrowed("author"), serde_json::Value::String(value.to_string()));
            }
            "date" => {
                metadata.insert(Cow::Borrowed("date"), serde_json::Value::String(value.to_string()));
            }
            "version" | "revision" => {
                metadata.insert(Cow::Borrowed("version"), serde_json::Value::String(value.to_string()));
            }
            "title" => {
                metadata.insert(Cow::Borrowed("title"), serde_json::Value::String(value.to_string()));
            }
            _ => {
                metadata.insert(
                    Cow::Owned(format!("field_{}", key_lower)),
                    serde_json::Value::String(value.to_string()),
                );
            }
        }
    }

    /// Check if a line is a section underline.
    fn is_section_underline(line: &str) -> bool {
        let trimmed = line.trim();
        if trimmed.len() < 3 {
            return false;
        }
        let chars: Vec<char> = trimmed.chars().collect();
        let first = chars[0];
        matches!(first, '=' | '-' | '~' | '+' | '^' | '"' | '`' | '#' | '*') && chars.iter().all(|c| *c == first)
    }

    /// Check if a line is a list item.
    fn is_list_item(line: &str) -> bool {
        let trimmed = line.trim_start();
        if trimmed.starts_with("* ") || trimmed.starts_with("+ ") || trimmed.starts_with("- ") {
            return true;
        }
        if let Some(space_pos) = trimmed.find(' ')
            && space_pos > 0
            && space_pos < 4
        {
            let prefix = &trimmed[..space_pos];
            if prefix.ends_with('.') || prefix.ends_with(')') {
                return prefix[..prefix.len() - 1].chars().all(|c| c.is_numeric());
            }
        }
        false
    }

    /// Check if a line is just markup (underlines, etc.)
    fn is_markup_line(line: &str) -> bool {
        let trimmed = line.trim();
        if trimmed.len() < 3 {
            return false;
        }
        let first = trimmed.chars().next().unwrap();
        trimmed.chars().all(|c| c == first)
            && matches!(first, '=' | '-' | '~' | '+' | '^' | '"' | '`' | '#' | '*' | '/')
    }

    /// Extract tables from RST content.
    ///
    /// Identifies and extracts both simple and grid tables.
    fn extract_tables(content: &str) -> Vec<Table> {
        let mut tables = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];

            if line.contains("|")
                && (line.contains("=") || line.contains("-"))
                && let Some(table) = Self::parse_grid_table(&lines, &mut i)
            {
                tables.push(table);
                continue;
            }

            i += 1;
        }

        tables
    }

    /// Parse a grid table from lines.
    fn parse_grid_table(lines: &[&str], i: &mut usize) -> Option<Table> {
        let mut cells = Vec::new();
        let mut row = Vec::new();

        while *i < lines.len() && lines[*i].contains("|") {
            let line = lines[*i].trim_matches(|c| c == '|');
            if !line.is_empty() {
                let cell_content = line.split('|').map(|s| s.trim().to_string()).collect::<Vec<_>>();
                row.extend(cell_content);

                if !row.is_empty() {
                    cells.push(row.clone());
                    row.clear();
                }
            }
            *i += 1;
        }

        if cells.is_empty() {
            return None;
        }

        let markdown = Self::cells_to_markdown(&cells);
        Some(Table {
            cells,
            markdown,
            page_number: 1,
            bounding_box: None,
        })
    }

    /// Strip RST inline markup from text and produce annotations with byte offsets
    /// into the stripped text.
    ///
    /// Handles: `**strong**` (bold), `*emphasis*` (italic), ``` ``literal`` ``` (code),
    /// and `` `interpreted` `` (code).
    fn parse_inline_markup(raw: &str) -> (String, Vec<TextAnnotation>) {
        let mut out = String::with_capacity(raw.len());
        let mut annotations = Vec::new();
        let bytes = raw.as_bytes();
        let len = bytes.len();
        let mut i = 0;

        while i < len {
            // **strong emphasis**
            if i + 1 < len && bytes[i] == b'*' && bytes[i + 1] == b'*' {
                if let Some(end) = Self::find_closing_marker(raw, i + 2, "**") {
                    let inner = &raw[i + 2..end];
                    let start = out.len() as u32;
                    out.push_str(inner);
                    let end_off = out.len() as u32;
                    if start < end_off {
                        annotations.push(TextAnnotation {
                            start,
                            end: end_off,
                            kind: AnnotationKind::Bold,
                        });
                    }
                    i = end + 2;
                    continue;
                }
            }
            // *emphasis*  (single star, not followed by another star)
            if bytes[i] == b'*' && (i + 1 >= len || bytes[i + 1] != b'*') {
                if let Some(end) = Self::find_closing_marker(raw, i + 1, "*") {
                    // Make sure this isn't inside a ** pair
                    if end + 1 >= len || bytes[end + 1] != b'*' {
                        let inner = &raw[i + 1..end];
                        let start = out.len() as u32;
                        out.push_str(inner);
                        let end_off = out.len() as u32;
                        if start < end_off {
                            annotations.push(TextAnnotation {
                                start,
                                end: end_off,
                                kind: AnnotationKind::Italic,
                            });
                        }
                        i = end + 1;
                        continue;
                    }
                }
            }
            // ``literal``
            if i + 1 < len && bytes[i] == b'`' && bytes[i + 1] == b'`' {
                if let Some(end) = Self::find_closing_marker(raw, i + 2, "``") {
                    let inner = &raw[i + 2..end];
                    let start = out.len() as u32;
                    out.push_str(inner);
                    let end_off = out.len() as u32;
                    if start < end_off {
                        annotations.push(TextAnnotation {
                            start,
                            end: end_off,
                            kind: AnnotationKind::Code,
                        });
                    }
                    i = end + 2;
                    continue;
                }
            }
            // `interpreted text`
            if bytes[i] == b'`' && (i + 1 >= len || bytes[i + 1] != b'`') {
                if let Some(end) = Self::find_closing_single_backtick(raw, i + 1) {
                    let inner = &raw[i + 1..end];
                    let start = out.len() as u32;
                    out.push_str(inner);
                    let end_off = out.len() as u32;
                    if start < end_off {
                        annotations.push(TextAnnotation {
                            start,
                            end: end_off,
                            kind: AnnotationKind::Code,
                        });
                    }
                    i = end + 1;
                    continue;
                }
            }
            out.push(bytes[i] as char);
            i += 1;
        }

        (out, annotations)
    }

    /// Find the position of a closing marker substring starting from `from`.
    fn find_closing_marker(text: &str, from: usize, marker: &str) -> Option<usize> {
        text[from..].find(marker).map(|pos| from + pos)
    }

    /// Find closing single backtick that is NOT part of a double backtick.
    fn find_closing_single_backtick(text: &str, from: usize) -> Option<usize> {
        let bytes = text.as_bytes();
        let mut j = from;
        while j < bytes.len() {
            if bytes[j] == b'`' {
                // Make sure it's not ``
                if j + 1 < bytes.len() && bytes[j + 1] == b'`' {
                    j += 2;
                    continue;
                }
                return Some(j);
            }
            j += 1;
        }
        None
    }

    /// Parse RST footnote references from a line.
    /// Returns footnote labels found (e.g. "1" from `[1]_` or "#" from `[#]_`).
    fn find_footnote_references(line: &str) -> Vec<String> {
        let mut refs = Vec::new();
        let bytes = line.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'[' {
                if let Some(close) = line[i + 1..].find(']') {
                    let label_end = i + 1 + close;
                    let label = &line[i + 1..label_end];
                    // Check for trailing _
                    if label_end + 1 < bytes.len() && bytes[label_end + 1] == b'_' {
                        // Valid footnote ref: numeric or #-prefixed
                        if label.chars().all(|c| c.is_ascii_digit()) || label.starts_with('#') {
                            refs.push(label.to_string());
                        }
                    }
                }
            }
            i += 1;
        }
        refs
    }

    /// Parse image directive options (`:alt:`, `:width:`, `:height:`) from indented lines.
    fn parse_image_options<'a>(lines: &[&'a str], start: &mut usize) -> AHashMap<String, String> {
        let mut opts = AHashMap::new();
        while *start < lines.len() {
            let line = lines[*start];
            if !line.starts_with("   ") && !line.starts_with("\t") {
                break;
            }
            let trimmed = line.trim();
            if trimmed.is_empty() {
                *start += 1;
                break;
            }
            if trimmed.starts_with(':') {
                if let Some(colon2) = trimmed[1..].find(':') {
                    let key = trimmed[1..1 + colon2].to_string();
                    let value = trimmed[2 + colon2..].trim().to_string();
                    opts.insert(key, value);
                }
            }
            *start += 1;
        }
        opts
    }

    /// Build a `DocumentStructure` from RST content.
    fn build_document_structure(content: &str) -> DocumentStructure {
        let mut builder = DocumentStructureBuilder::new().source_format("rst");
        let lines: Vec<&str> = content.lines().collect();
        // Determine heading level order from the underline characters used.
        // RST heading levels are defined by the order of appearance of underline characters.
        let mut heading_char_order: Vec<char> = Vec::new();
        let mut i = 0;

        while i < lines.len() {
            let line = lines[i];
            let trimmed = line.trim();

            // Field list metadata
            if trimmed.starts_with(':')
                && trimmed.len() > 1
                && let Some((key, value)) = Self::parse_field_list_line(trimmed)
            {
                // Collect continuation lines
                let mut full_value = value;
                while i + 1 < lines.len() {
                    let next = lines[i + 1];
                    if !next.is_empty() && (next.starts_with("   ") || next.starts_with("\t")) {
                        full_value.push('\n');
                        full_value.push_str(next.trim());
                        i += 1;
                    } else {
                        break;
                    }
                }
                builder.push_metadata_block(vec![(key, full_value)], None);
                i += 1;
                continue;
            }

            // Heading: text line followed by underline
            if i + 1 < lines.len() && !trimmed.is_empty() && Self::is_section_underline(lines[i + 1]) {
                let underline_char = lines[i + 1].trim().chars().next().unwrap_or('=');
                if !heading_char_order.contains(&underline_char) {
                    heading_char_order.push(underline_char);
                }
                let level = heading_char_order
                    .iter()
                    .position(|&c| c == underline_char)
                    .map(|p| (p + 1) as u8)
                    .unwrap_or(1);
                builder.push_heading(level, trimmed, None, None);
                i += 2;
                continue;
            }

            // Code block directive
            if trimmed.starts_with(".. code-block::") || trimmed.starts_with(".. code::") {
                let language = if let Some(rest) = trimmed.strip_prefix(".. code-block::") {
                    let lang = rest.trim();
                    if lang.is_empty() { None } else { Some(lang.to_string()) }
                } else if let Some(rest) = trimmed.strip_prefix(".. code::") {
                    let lang = rest.trim();
                    if lang.is_empty() { None } else { Some(lang.to_string()) }
                } else {
                    None
                };
                i += 1;
                // Skip blank lines after directive
                while i < lines.len() && lines[i].trim().is_empty() {
                    i += 1;
                }
                // Collect indented content
                let mut code_content = String::new();
                while i < lines.len() && (lines[i].starts_with("   ") || lines[i].is_empty()) {
                    if !code_content.is_empty() {
                        code_content.push('\n');
                    }
                    // Strip 3-space indent
                    if lines[i].starts_with("   ") {
                        code_content.push_str(&lines[i][3..]);
                    }
                    i += 1;
                }
                builder.push_code(code_content.trim_end(), language.as_deref(), None);
                continue;
            }

            // Admonition directives
            if trimmed.starts_with(".. note::")
                || trimmed.starts_with(".. warning::")
                || trimmed.starts_with(".. important::")
                || trimmed.starts_with(".. caution::")
                || trimmed.starts_with(".. hint::")
                || trimmed.starts_with(".. tip::")
            {
                let kind = trimmed.strip_prefix(".. ").unwrap_or("").trim_end_matches("::").trim();
                builder.push_admonition(kind, None, None);
                i += 1;
                // Collect indented content as paragraphs
                let mut admonition_text = String::new();
                while i < lines.len() && (lines[i].starts_with("   ") || lines[i].is_empty()) {
                    if !lines[i].is_empty() {
                        if !admonition_text.is_empty() {
                            admonition_text.push(' ');
                        }
                        admonition_text.push_str(lines[i].trim());
                    }
                    i += 1;
                }
                if !admonition_text.is_empty() {
                    builder.push_paragraph(&admonition_text, vec![], None, None);
                }
                builder.exit_container();
                continue;
            }

            // Image directive with options
            if trimmed.starts_with(".. image::") {
                let uri = trimmed.strip_prefix(".. image::").unwrap_or("").trim();
                i += 1;
                let opts = Self::parse_image_options(&lines, &mut i);
                let alt = opts.get("alt").cloned();
                let description = alt.as_deref().or(if uri.is_empty() { None } else { Some(uri) });
                let img_idx = builder.push_image(description, None, None, None);
                // Store width/height and uri as attributes
                let mut attrs = AHashMap::new();
                if !uri.is_empty() {
                    attrs.insert("src".to_string(), uri.to_string());
                }
                if let Some(w) = opts.get("width") {
                    attrs.insert("width".to_string(), w.clone());
                }
                if let Some(h) = opts.get("height") {
                    attrs.insert("height".to_string(), h.clone());
                }
                if !attrs.is_empty() {
                    builder.set_attributes(img_idx, attrs);
                }
                continue;
            }

            // Math directive
            if trimmed.starts_with(".. math::") {
                let inline_math = trimmed.strip_prefix(".. math::").unwrap_or("").trim();
                i += 1;
                let mut math_content = if inline_math.is_empty() {
                    String::new()
                } else {
                    inline_math.to_string()
                };
                // Collect indented math content
                while i < lines.len() && (lines[i].starts_with("   ") || lines[i].is_empty()) {
                    if !lines[i].is_empty() {
                        if !math_content.is_empty() {
                            math_content.push('\n');
                        }
                        math_content.push_str(lines[i].trim());
                    }
                    i += 1;
                }
                if !math_content.is_empty() {
                    builder.push_formula(&math_content, None);
                }
                continue;
            }

            // Other directives - skip
            if trimmed.starts_with(".. ") || trimmed == ".." {
                i += 1;
                while i < lines.len() && (lines[i].starts_with("   ") || lines[i].is_empty()) {
                    i += 1;
                }
                continue;
            }

            // Definition list: term followed by indented definition
            if !trimmed.is_empty()
                && !Self::is_list_item(line)
                && i + 1 < lines.len()
                && !lines[i + 1].trim().is_empty()
                && (lines[i + 1].starts_with("   ") || lines[i + 1].starts_with("\t"))
                && !Self::is_section_underline(lines[i + 1])
            {
                // Check if this really is a definition list (term + indented definition)
                let term = trimmed.to_string();
                i += 1;
                let mut definition = String::new();
                while i < lines.len() && (lines[i].starts_with("   ") || lines[i].starts_with("\t")) {
                    if !definition.is_empty() {
                        definition.push(' ');
                    }
                    definition.push_str(lines[i].trim());
                    i += 1;
                }
                let dl = builder.push_definition_list(None);
                builder.push_definition_item(dl, &term, &definition, None);
                continue;
            }

            // List items
            if Self::is_list_item(line) {
                let is_ordered = {
                    let t = trimmed.trim_start();
                    if let Some(space_pos) = t.find(' ') {
                        let prefix = &t[..space_pos];
                        prefix.ends_with('.') || prefix.ends_with(')')
                    } else {
                        false
                    }
                };
                let list_idx = builder.push_list(is_ordered, None);
                // Collect consecutive list items
                while i < lines.len() && Self::is_list_item(lines[i]) {
                    let item_trimmed = lines[i].trim();
                    // Strip bullet/number prefix
                    let text = if let Some(rest) = item_trimmed
                        .strip_prefix("* ")
                        .or_else(|| item_trimmed.strip_prefix("+ "))
                        .or_else(|| item_trimmed.strip_prefix("- "))
                    {
                        rest
                    } else if let Some(space_pos) = item_trimmed.find(' ') {
                        &item_trimmed[space_pos + 1..]
                    } else {
                        item_trimmed
                    };
                    builder.push_list_item(list_idx, text, None);
                    i += 1;
                }
                continue;
            }

            // Grid table
            if trimmed.contains('|') && (trimmed.contains('=') || trimmed.contains('-')) {
                let mut table_lines = Vec::new();
                while i < lines.len() && lines[i].contains('|') {
                    table_lines.push(lines[i]);
                    i += 1;
                }
                let cells = Self::parse_grid_table_cells(&table_lines);
                if !cells.is_empty() {
                    builder.push_table_simple(&cells, None);
                }
                continue;
            }

            // Footnote definitions: .. [1] text  or  .. [#label] text
            if trimmed.starts_with(".. [")
                && let Some(close) = trimmed.find(']')
                && close > 4
            {
                let label = &trimmed[4..close];
                let footnote_text = trimmed[close + 1..].trim();
                // Collect continuation lines
                let mut full_text = footnote_text.to_string();
                i += 1;
                while i < lines.len() && (lines[i].starts_with("   ") || lines[i].starts_with("\t")) {
                    if !full_text.is_empty() {
                        full_text.push(' ');
                    }
                    full_text.push_str(lines[i].trim());
                    i += 1;
                }
                let display = if full_text.is_empty() {
                    format!("[{}]", label)
                } else {
                    format!("[{}] {}", label, full_text)
                };
                builder.push_footnote(&display, None);
                continue;
            }

            // Regular paragraph with inline markup scanning
            if !trimmed.is_empty() && !Self::is_markup_line(line) {
                // Check for footnote references first
                let footnote_refs = Self::find_footnote_references(trimmed);
                let (stripped, annotations) = Self::parse_inline_markup(trimmed);
                builder.push_paragraph(&stripped, annotations, None, None);
                for fref in footnote_refs {
                    builder.push_footnote(&format!("[{}]", fref), None);
                }
            }

            i += 1;
        }

        builder.build()
    }

    /// Parse cells from grid table lines (for DocumentStructure).
    fn parse_grid_table_cells(lines: &[&str]) -> Vec<Vec<String>> {
        let mut cells = Vec::new();
        for line in lines {
            let content = line.trim().trim_matches('|');
            if content.is_empty() {
                continue;
            }
            // Skip separator lines (all dashes/equals)
            if content
                .chars()
                .all(|c| c == '-' || c == '=' || c == '+' || c == '|' || c == ' ')
            {
                continue;
            }
            let row: Vec<String> = content
                .split('|')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if !row.is_empty() {
                cells.push(row);
            }
        }
        cells
    }

    /// Convert table cells to markdown format.
    fn cells_to_markdown(cells: &[Vec<String>]) -> String {
        if cells.is_empty() {
            return String::new();
        }

        let mut md = String::new();

        md.push('|');
        for cell in &cells[0] {
            md.push(' ');
            md.push_str(cell);
            md.push_str(" |");
        }
        md.push('\n');

        md.push('|');
        for _ in &cells[0] {
            md.push_str(" --- |");
        }
        md.push('\n');

        for row in &cells[1..] {
            md.push('|');
            for cell in row {
                md.push(' ');
                md.push_str(cell);
                md.push_str(" |");
            }
            md.push('\n');
        }

        md
    }
}

#[cfg(feature = "office")]
impl Default for RstExtractor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "office")]
impl Plugin for RstExtractor {
    fn name(&self) -> &str {
        "rst-extractor"
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
        "Native Rust extractor for reStructuredText (RST) documents"
    }

    fn author(&self) -> &str {
        "Kreuzberg Team"
    }
}

#[cfg(feature = "office")]
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
impl DocumentExtractor for RstExtractor {
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
        let text = String::from_utf8_lossy(content).into_owned();

        let (extracted_text, metadata) = Self::extract_text_and_metadata(&text);

        let tables = Self::extract_tables(&text);

        let document = if config.include_document_structure {
            Some(Self::build_document_structure(&text))
        } else {
            None
        };

        Ok(ExtractionResult {
            content: extracted_text,
            mime_type: mime_type.to_string().into(),
            metadata,
            tables,
            detected_languages: None,
            chunks: None,
            images: None,
            djot_content: None,
            pages: None,
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
        &["text/x-rst", "text/prs.fallenstein.rst"]
    }

    fn priority(&self) -> i32 {
        50
    }
}

#[cfg(all(test, feature = "office"))]
mod tests {
    use super::*;

    #[test]
    fn test_rst_extractor_plugin_interface() {
        let extractor = RstExtractor::new();
        assert_eq!(extractor.name(), "rst-extractor");
        assert_eq!(extractor.version(), env!("CARGO_PKG_VERSION"));
        assert_eq!(extractor.priority(), 50);
        assert!(!extractor.supported_mime_types().is_empty());
    }

    #[test]
    fn test_rst_extractor_supports_text_x_rst() {
        let extractor = RstExtractor::new();
        assert!(extractor.supported_mime_types().contains(&"text/x-rst"));
    }

    #[test]
    fn test_rst_extractor_supports_fallenstein_rst() {
        let extractor = RstExtractor::new();
        assert!(extractor.supported_mime_types().contains(&"text/prs.fallenstein.rst"));
    }

    #[test]
    fn test_extract_text_from_rst_simple_document() {
        let content = r#"
Title
=====

This is a paragraph.

Another paragraph.
"#;

        let mut metadata = AHashMap::new();
        let output = RstExtractor::extract_text_from_rst(content, &mut metadata);
        assert!(output.contains("Title"));
        assert!(output.contains("This is a paragraph"));
        assert!(output.contains("Another paragraph"));
    }

    #[test]
    fn test_extract_text_from_rst_with_code_block() {
        let content = r#"
.. code-block:: python

   def hello():
       print("world")

Some text after.
"#;

        let mut metadata = AHashMap::new();
        let output = RstExtractor::extract_text_from_rst(content, &mut metadata);
        assert!(output.contains("code-block"));
        assert!(output.contains("def hello"));
        assert!(output.contains("Some text after"));
    }

    #[test]
    fn test_extract_text_from_rst_with_metadata() {
        let content = r#"
:Author: John Doe
:Date: 2024-01-15

First paragraph.

Second paragraph.
"#;

        let mut metadata = AHashMap::new();
        let output = RstExtractor::extract_text_from_rst(content, &mut metadata);
        assert!(output.contains("First paragraph"));
        assert!(output.contains("Second paragraph"));
        assert!(metadata.contains_key("author"));
        assert_eq!(metadata.get("author").and_then(|v| v.as_str()), Some("John Doe"));
    }

    #[test]
    fn test_cells_to_markdown_format() {
        let cells = vec![
            vec!["Name".to_string(), "Age".to_string()],
            vec!["Alice".to_string(), "30".to_string()],
            vec!["Bob".to_string(), "25".to_string()],
        ];

        let markdown = RstExtractor::cells_to_markdown(&cells);
        assert!(markdown.contains("Name"));
        assert!(markdown.contains("Age"));
        assert!(markdown.contains("Alice"));
        assert!(markdown.contains("Bob"));
        assert!(markdown.contains("---"));
    }

    #[test]
    fn test_rst_extractor_default() {
        let extractor = RstExtractor;
        assert_eq!(extractor.name(), "rst-extractor");
    }

    #[test]
    fn test_rst_extractor_initialize_shutdown() {
        let extractor = RstExtractor::new();
        assert!(extractor.initialize().is_ok());
        assert!(extractor.shutdown().is_ok());
    }
}
