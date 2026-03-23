//! Integration tests verifying DocumentStructure output for all migrated extractors.

mod helpers;

use kreuzberg::core::config::ExtractionConfig;
use kreuzberg::core::extractor::extract_file;
use kreuzberg::rendering::render_to_markdown;
use kreuzberg::types::document_structure::NodeContent;

/// Helper: check whether a document contains at least one node matching a predicate.
fn has_node_type(
    doc: &kreuzberg::types::document_structure::DocumentStructure,
    predicate: fn(&NodeContent) -> bool,
) -> bool {
    doc.nodes.iter().any(|n| predicate(&n.content))
}

/// Build an `ExtractionConfig` with document structure enabled.
fn config_with_structure() -> ExtractionConfig {
    ExtractionConfig {
        include_document_structure: true,
        ..Default::default()
    }
}

// ============================================================================
// 1. DOCX
// ============================================================================

#[cfg(feature = "office")]
#[tokio::test]
async fn test_document_structure_docx() {
    let path = helpers::get_test_file_path("docx/unit_test_headers.docx");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("DOCX extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert!(!doc.nodes.is_empty(), "document nodes should be non-empty for DOCX");
    assert_eq!(
        doc.source_format.as_deref(),
        Some("docx"),
        "source_format should be 'docx'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Heading { .. })),
        "DOCX with headers should contain Heading nodes"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// 2. PPTX
// ============================================================================

#[cfg(feature = "office")]
#[tokio::test]
async fn test_document_structure_pptx() {
    let path = helpers::get_test_file_path("pptx/powerpoint_sample.ppsx");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("PPTX extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("pptx"),
        "source_format should be 'pptx'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Slide { .. })),
        "PPTX should contain Slide nodes"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// 3. HTML
// ============================================================================

#[cfg(feature = "html")]
#[tokio::test]
async fn test_document_structure_html() {
    let path = helpers::get_test_file_path("html/html.htm");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("HTML extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("html"),
        "source_format should be 'html'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Heading { .. })),
        "HTML should contain Heading nodes"
    );
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Paragraph { .. })),
        "HTML should contain Paragraph nodes"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// 4. LaTeX
// ============================================================================

#[cfg(feature = "office")]
#[tokio::test]
async fn test_document_structure_latex() {
    let path = helpers::get_test_file_path("latex/basic_sections.tex");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("LaTeX extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("latex"),
        "source_format should be 'latex'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Heading { .. })),
        "LaTeX with \\section commands should contain Heading nodes"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// 5. RST
// ============================================================================

#[cfg(feature = "office")]
#[tokio::test]
async fn test_document_structure_rst() {
    let path = helpers::get_test_file_path("rst/restructured_text.rst");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("RST extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("rst"),
        "source_format should be 'rst'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Heading { .. })),
        "RST should contain Heading nodes"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// 6. Org Mode
// ============================================================================

#[cfg(feature = "office")]
#[tokio::test]
async fn test_document_structure_orgmode() {
    let path = helpers::get_test_file_path("org/comprehensive.org");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("OrgMode extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("orgmode"),
        "source_format should be 'orgmode'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Heading { .. })),
        "OrgMode with * headings should contain Heading nodes"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// 7. EPUB
// ============================================================================

#[cfg(feature = "office")]
#[tokio::test]
async fn test_document_structure_epub() {
    let path = helpers::get_test_file_path("epub/features.epub");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("EPUB extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("epub"),
        "source_format should be 'epub'"
    );
    assert!(!doc.nodes.is_empty(), "document nodes should be non-empty for EPUB");
    assert!(doc.validate().is_ok(), "document structure validation should pass");

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// 8. Excel
// ============================================================================

#[cfg(any(feature = "excel", feature = "excel-wasm"))]
#[tokio::test]
async fn test_document_structure_excel() {
    let path = helpers::get_test_file_path("xlsx/excel_multi_sheet.xlsx");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("Excel extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("excel"),
        "source_format should be 'excel'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Table { .. })),
        "Excel should contain Table nodes from sheet data"
    );
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Heading { .. })),
        "Excel should contain Heading nodes from sheet names"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// 9. CSV
// ============================================================================

#[tokio::test]
async fn test_document_structure_csv() {
    let path = helpers::get_test_file_path("csv/data_table.csv");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("CSV extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("csv"),
        "source_format should be 'csv'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Table { .. })),
        "CSV should contain Table nodes"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// 10. Email
// ============================================================================

#[cfg(feature = "email")]
#[tokio::test]
async fn test_document_structure_email() {
    let path = helpers::get_test_file_path("email/fake_email.msg");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("Email extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("email"),
        "source_format should be 'email'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::MetadataBlock { .. })),
        "Email should contain MetadataBlock nodes from headers"
    );
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Paragraph { .. })),
        "Email should contain Paragraph nodes from body"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// 11. BibTeX
// ============================================================================

#[cfg(feature = "office")]
#[tokio::test]
async fn test_document_structure_bibtex() {
    let path = helpers::get_test_file_path("bibtex/comprehensive.bib");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("BibTeX extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("bibtex"),
        "source_format should be 'bibtex'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Citation { .. })),
        "BibTeX should contain Citation nodes"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// 12. Jupyter
// ============================================================================

#[cfg(feature = "office")]
#[tokio::test]
async fn test_document_structure_jupyter() {
    let path = helpers::get_test_file_path("jupyter/mime.ipynb");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("Jupyter extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("jupyter"),
        "source_format should be 'jupyter'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Code { .. })),
        "Jupyter should contain Code nodes from code cells"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// 13. PlainText
// ============================================================================

#[tokio::test]
async fn test_document_structure_plaintext() {
    let path = helpers::get_test_file_path("text/contract.txt");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("PlainText extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("text"),
        "source_format should be 'text'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Paragraph { .. })),
        "PlainText should contain Paragraph nodes"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// 14. Markdown
// ============================================================================

#[tokio::test]
async fn test_document_structure_markdown() {
    let path = helpers::get_test_file_path("markdown/comprehensive.md");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("Markdown extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    // When the `office` feature is enabled, the EnhancedMarkdownExtractor takes
    // priority and delegates document structure to the pipeline fallback, which
    // does not set source_format. The basic MarkdownExtractor (always registered)
    // sets source_format = "markdown" natively.
    if doc.source_format.is_some() {
        assert_eq!(
            doc.source_format.as_deref(),
            Some("markdown"),
            "source_format should be 'markdown' when set"
        );
    }
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Heading { .. }))
            || has_node_type(doc, |c| matches!(c, NodeContent::Paragraph { .. })),
        "Markdown should contain Heading or Paragraph nodes"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// 15. ODT
// ============================================================================

#[cfg(feature = "office")]
#[tokio::test]
async fn test_document_structure_odt() {
    let path = helpers::get_test_file_path("odt/headers.odt");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("ODT extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("odt"),
        "source_format should be 'odt'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(!doc.nodes.is_empty(), "document nodes should be non-empty for ODT");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Heading { .. })),
        "ODT with headers should contain Heading nodes"
    );
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Paragraph { .. })),
        "ODT should contain Paragraph nodes"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

#[cfg(feature = "office")]
#[tokio::test]
async fn test_document_structure_odt_table() {
    let path = helpers::get_test_file_path("odt/simpleTable.odt");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("ODT table extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(doc.source_format.as_deref(), Some("odt"));
    assert!(doc.validate().is_ok());
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Table { .. })),
        "ODT with table should contain Table nodes"
    );
}

#[cfg(feature = "office")]
#[tokio::test]
async fn test_document_structure_odt_list() {
    let path = helpers::get_test_file_path("odt/unorderedList.odt");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("ODT list extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(doc.source_format.as_deref(), Some("odt"));
    assert!(doc.validate().is_ok());
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::List { .. })),
        "ODT with list should contain List nodes"
    );
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::ListItem { .. })),
        "ODT with list should contain ListItem nodes"
    );
}

// ============================================================================
// 16. DOC
// ============================================================================

#[cfg(feature = "office")]
#[tokio::test]
async fn test_document_structure_doc() {
    let path = helpers::get_test_file_path("doc/unit_test_lists.doc");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("DOC extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("doc"),
        "source_format should be 'doc'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(!doc.nodes.is_empty(), "document nodes should be non-empty for DOC");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Paragraph { .. })),
        "DOC should contain Paragraph nodes"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// 17. PPT
// ============================================================================

#[cfg(feature = "office")]
#[tokio::test]
async fn test_document_structure_ppt() {
    let path = helpers::get_test_file_path("ppt/simple.ppt");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("PPT extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("ppt"),
        "source_format should be 'ppt'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(!doc.nodes.is_empty(), "document nodes should be non-empty for PPT");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Slide { .. })),
        "PPT should contain Slide nodes"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// 18. RTF
// ============================================================================

#[cfg(feature = "office")]
#[tokio::test]
async fn test_document_structure_rtf() {
    let path = helpers::get_test_file_path("rtf/heading.rtf");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("RTF extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("rtf"),
        "source_format should be 'rtf'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(!doc.nodes.is_empty(), "document nodes should be non-empty for RTF");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Paragraph { .. })),
        "RTF should contain Paragraph nodes"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

#[cfg(feature = "office")]
#[tokio::test]
async fn test_document_structure_rtf_table() {
    let path = helpers::get_test_file_path("rtf/table_simple.rtf");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("RTF table extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(doc.source_format.as_deref(), Some("rtf"));
    assert!(doc.validate().is_ok());
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Table { .. }))
            || has_node_type(doc, |c| matches!(c, NodeContent::Paragraph { .. })),
        "RTF with tables should contain Table or Paragraph nodes"
    );
}

// ============================================================================
// 19. DocBook
// ============================================================================

#[cfg(feature = "xml")]
#[tokio::test]
async fn test_document_structure_docbook() {
    let path = helpers::get_test_file_path("docbook/docbook-chapter.docbook");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("DocBook extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert!(!doc.nodes.is_empty(), "document nodes should be non-empty for DocBook");
    assert_eq!(
        doc.source_format.as_deref(),
        Some("docbook"),
        "source_format should be 'docbook'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Heading { .. })),
        "DocBook with chapters/sections should contain Heading nodes"
    );
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Paragraph { .. })),
        "DocBook should contain Paragraph nodes"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// 20. JATS
// ============================================================================

#[cfg(feature = "xml")]
#[tokio::test]
async fn test_document_structure_jats() {
    let path = helpers::get_test_file_path("jats/sample_article.jats");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("JATS extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert!(!doc.nodes.is_empty(), "document nodes should be non-empty for JATS");
    assert_eq!(
        doc.source_format.as_deref(),
        Some("jats"),
        "source_format should be 'jats'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Heading { .. })),
        "JATS article should contain Heading nodes"
    );
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Paragraph { .. })),
        "JATS article should contain Paragraph nodes"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// 21. FictionBook
// ============================================================================

#[cfg(feature = "office")]
#[tokio::test]
async fn test_document_structure_fictionbook() {
    let path = helpers::get_test_file_path("fictionbook/basic.fb2");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("FictionBook extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert!(
        !doc.nodes.is_empty(),
        "document nodes should be non-empty for FictionBook"
    );
    assert_eq!(
        doc.source_format.as_deref(),
        Some("fictionbook"),
        "source_format should be 'fictionbook'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Heading { .. })),
        "FictionBook with sections should contain Heading nodes"
    );
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Paragraph { .. })),
        "FictionBook should contain Paragraph nodes"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// DBF
// ============================================================================

#[tokio::test]
async fn test_document_structure_dbf() {
    let path = helpers::get_test_file_path("dbf/stations.dbf");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("DBF extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("dbf"),
        "source_format should be 'dbf'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Table { .. })),
        "DBF should contain Table nodes from records"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// Citation (RIS)
// ============================================================================

#[cfg(feature = "office")]
#[tokio::test]
async fn test_document_structure_citation() {
    let path = helpers::get_test_file_path("data_formats/sample.ris");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("Citation extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("citation"),
        "source_format should be 'citation'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Citation { .. })),
        "Citation file should contain Citation nodes"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// XML
// ============================================================================

#[tokio::test]
async fn test_document_structure_xml() {
    let path = helpers::get_test_file_path("xml/simple_note.xml");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("XML extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("xml"),
        "source_format should be 'xml'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Paragraph { .. })),
        "XML should contain Paragraph nodes from text content"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// Structured (JSON)
// ============================================================================

#[tokio::test]
async fn test_document_structure_json() {
    let path = helpers::get_test_file_path("json/simple.json");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("JSON extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("json"),
        "source_format should be 'json'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Code { .. })),
        "JSON should contain Code nodes for structured data"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// Structured (YAML)
// ============================================================================

#[tokio::test]
async fn test_document_structure_yaml() {
    let path = helpers::get_test_file_path("yaml/simple.yaml");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("YAML extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("yaml"),
        "source_format should be 'yaml'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Code { .. })),
        "YAML should contain Code nodes for structured data"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// Image (no OCR -- structure only)
// ============================================================================

#[cfg(any(feature = "ocr", feature = "ocr-wasm"))]
#[tokio::test]
async fn test_document_structure_image() {
    let path = helpers::get_test_file_path("images/example.jpg");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config).await;

    // Image extraction may fail if OCR backend is not configured at runtime;
    // verify it does not crash and produces a document when it succeeds.
    let result = match result {
        Ok(r) => r,
        Err(_) => return,
    };

    assert!(result.document.is_some(), "document should be populated for image");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("image"),
        "source_format should be 'image'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Image { .. })),
        "Image should contain Image nodes"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// OPML
// ============================================================================

#[cfg(feature = "office")]
#[tokio::test]
async fn test_document_structure_opml() {
    let path = helpers::get_test_file_path("opml/outline.opml");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("OPML extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("opml"),
        "source_format should be 'opml'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(!doc.nodes.is_empty(), "document nodes should be non-empty for OPML");

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// HWP
// ============================================================================

#[cfg(feature = "hwp")]
#[tokio::test]
async fn test_document_structure_hwp() {
    let path = helpers::get_test_file_path("hwp/converted_output.hwp");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("HWP extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("hwp"),
        "source_format should be 'hwp'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Paragraph { .. })),
        "HWP should contain Paragraph nodes"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// iWork Keynote
// ============================================================================

#[cfg(feature = "iwork")]
#[tokio::test]
async fn test_document_structure_keynote() {
    let path = helpers::get_test_file_path("iwork/test.key");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("Keynote extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("keynote"),
        "source_format should be 'keynote'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(!doc.nodes.is_empty(), "document nodes should be non-empty for Keynote");

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// iWork Numbers
// ============================================================================

#[cfg(feature = "iwork")]
#[tokio::test]
async fn test_document_structure_numbers() {
    let path = helpers::get_test_file_path("iwork/test.numbers");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("Numbers extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("numbers"),
        "source_format should be 'numbers'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(!doc.nodes.is_empty(), "document nodes should be non-empty for Numbers");

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// iWork Pages
// ============================================================================

#[cfg(feature = "iwork")]
#[tokio::test]
async fn test_document_structure_pages() {
    let path = helpers::get_test_file_path("iwork/test.pages");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("Pages extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("pages"),
        "source_format should be 'pages'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Paragraph { .. })),
        "Pages should contain Paragraph nodes"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// Enhanced Markdown (office feature — pulldown-cmark AST)
// ============================================================================

#[cfg(feature = "office")]
#[tokio::test]
async fn test_document_structure_enhanced_markdown() {
    let path = helpers::get_test_file_path("markdown/comprehensive.md");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("Enhanced Markdown extraction should succeed");

    assert!(
        result.document.is_some(),
        "document should be populated for enhanced markdown"
    );
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("markdown"),
        "source_format should be 'markdown'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Heading { .. })),
        "Enhanced Markdown should contain Heading nodes from pulldown-cmark AST"
    );
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Paragraph { .. })),
        "Enhanced Markdown should contain Paragraph nodes"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

#[cfg(feature = "office")]
#[tokio::test]
async fn test_document_structure_enhanced_markdown_with_code() {
    let path = helpers::get_test_file_path("markdown/extraction_test.md");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("Enhanced Markdown extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert!(doc.validate().is_ok(), "document structure validation should pass");

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// MDX
// ============================================================================

#[cfg(feature = "mdx")]
#[tokio::test]
async fn test_document_structure_mdx() {
    let path = helpers::get_test_file_path("markdown/sample.mdx");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("MDX extraction should succeed");

    assert!(result.document.is_some(), "document should be populated for MDX");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("mdx"),
        "source_format should be 'mdx'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Heading { .. })),
        "MDX should contain Heading nodes"
    );
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Paragraph { .. })),
        "MDX should contain Paragraph nodes"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

#[cfg(feature = "mdx")]
#[tokio::test]
async fn test_document_structure_mdx_with_frontmatter() {
    let path = helpers::get_test_file_path("markdown/sample.mdx");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("MDX extraction should succeed");

    let doc = result.document.as_ref().unwrap();

    // sample.mdx has YAML frontmatter which should produce a MetadataBlock
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::MetadataBlock { .. })),
        "MDX with frontmatter should contain MetadataBlock nodes"
    );
}

// ============================================================================
// Djot
// ============================================================================

#[tokio::test]
async fn test_document_structure_djot() {
    let path = helpers::get_test_file_path("markdown/tables.djot");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("Djot extraction should succeed");

    assert!(result.document.is_some(), "document should be populated for Djot");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("djot"),
        "source_format should be 'djot'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(!doc.nodes.is_empty(), "document nodes should be non-empty for Djot");

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

// ============================================================================
// Typst
// ============================================================================

#[cfg(feature = "office")]
#[tokio::test]
async fn test_document_structure_typst() {
    let path = helpers::get_test_file_path("typst/headings.typ");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("Typst extraction should succeed");

    assert!(result.document.is_some(), "document should be populated for Typst");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(
        doc.source_format.as_deref(),
        Some("typst"),
        "source_format should be 'typst'"
    );
    assert!(doc.validate().is_ok(), "document structure validation should pass");
    assert!(
        has_node_type(doc, |c| matches!(c, NodeContent::Heading { .. })),
        "Typst with headings should contain Heading nodes"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

#[cfg(feature = "office")]
#[tokio::test]
async fn test_document_structure_typst_metadata() {
    let path = helpers::get_test_file_path("typst/metadata.typ");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("Typst extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(doc.source_format.as_deref(), Some("typst"));
    assert!(doc.validate().is_ok());
    assert!(
        !doc.nodes.is_empty(),
        "document nodes should be non-empty for Typst with metadata"
    );

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}

#[cfg(feature = "office")]
#[tokio::test]
async fn test_document_structure_typst_code_blocks() {
    let path = helpers::get_test_file_path("typst/advanced.typ");
    if !path.exists() {
        return;
    }

    let config = config_with_structure();
    let result = extract_file(&path, None, &config)
        .await
        .expect("Typst extraction should succeed");

    assert!(result.document.is_some(), "document should be populated");
    let doc = result.document.as_ref().unwrap();

    assert_eq!(doc.source_format.as_deref(), Some("typst"));
    assert!(doc.validate().is_ok());

    let md = render_to_markdown(doc);
    assert!(
        !md.trim().is_empty(),
        "render_to_markdown should produce non-empty output"
    );
}
