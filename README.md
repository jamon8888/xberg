# Kreuzberg

Kreuzberg is a modern Python library for text extraction from documents, designed for simplicity and efficiency. It provides a unified async interface for extracting text from a wide range of file formats including PDFs, images, office documents, and more.

## Why Kreuzberg?

- **Simple and Hassle-Free**: Clean API that just works, without complex configuration
- **Local Processing**: No external API calls or cloud dependencies required
- **Resource Efficient**: Lightweight processing without GPU requirements
- **Format Support**: Comprehensive support for documents, images, and text formats
- **Modern Python**: Built with async/await, type hints, and current best practices

Kreuzberg was created to solve text extraction needs in RAG (Retrieval Augmented Generation) applications.
Unlike many commercial solutions that require API calls or complex setups, Kreuzberg focuses on local processing with minimal dependencies.

## Features

- **Universal Text Extraction**: Extract text from PDFs (both searchable and scanned), images, office documents, and more
- **Smart Processing**: Automatic OCR for scanned documents, encoding detection for text files
- **Modern Python Design**:
  - Async-first API using `anyio` for multi-loop compatibility
  - Comprehensive type hints and full API documentation
  - Detailed error handling with context information
- **Production Ready**:
  - Robust error handling
  - Detailed debugging information
  - Memory efficient processing
  - Extensive test coverage

## Installation

### 1. Install the Python Package

```shell
pip install kreuzberg
```

### 2. Install System Dependencies

Kreuzberg requires two system level dependencies:

- [Pandoc](https://pandoc.org/installing.html) - For document format conversion
- [Tesseract OCR](https://tesseract-ocr.github.io/) - For image and PDF OCR

Please install these using their respective installation guides.

## Architecture

Kreuzberg is designed as a high-level async abstraction over established open-source tools. It integrates:

- **PDF Processing**:
  - `pdfium2` for searchable PDFs
  - Tesseract OCR for scanned content
- **Document Conversion**:
  - Pandoc for many document and markup formats
  - `python-pptx` for PowerPoint files
  - `html-to-markdown` for HTML content
  - `calamine` for Excel spreadsheets (with multi-sheet support)
- **Text Processing**:
  - Smart encoding detection
  - Markdown and plain text handling

### Supported Formats

#### Document Formats

- PDF (`.pdf`, both searchable and scanned documents)
- Microsoft Word (`.docx`)
- PowerPoint presentations (`.pptx`)
- OpenDocument Text (`.odt`)
- Rich Text Format (`.rtf`)
- EPUB (`.epub`)
- DocBook XML (`.dbk`, `.xml`)
- FictionBook (`.fb2`)
- LaTeX (`.tex`, `.latex`)
- Typst (`.typ`)

#### Markup and Text Formats

- HTML (`.html`, `.htm`)
- Plain text (`.txt`) and Markdown (`.md`, `.markdown`)
- reStructuredText (`.rst`)
- Org-mode (`.org`)
- DokuWiki (`.txt`)
- Pod (`.pod`)
- Troff/Man (`.1`, `.2`, etc.)

#### Data and Research Formats

- Excel spreadsheets (`.xlsx`)
- CSV (`.csv`) and TSV (`.tsv`) files
- OPML files (`.opml`)
- Jupyter Notebooks (`.ipynb`)
- BibTeX (`.bib`) and BibLaTeX (`.bib`)
- CSL-JSON (`.json`)
- EndNote XML (`.xml`)
- RIS (`.ris`)
- JATS XML (`.xml`)

#### Image Formats

- JPEG (`.jpg`, `.jpeg`, `.pjpeg`)
- PNG (`.png`)
- TIFF (`.tiff`, `.tif`)
- BMP (`.bmp`)
- GIF (`.gif`)
- JPEG 2000 family (`.jp2`, `.jpm`, `.jpx`, `.mj2`)
- WebP (`.webp`)
- Portable anymap formats:
  - Portable Anymap (`.pnm`)
  - Portable Bitmap (`.pbm`)
  - Portable Graymap (`.pgm`)
  - Portable Pixmap (`.ppm`)

## Usage

Kreuzberg provides both async and sync APIs for text extraction. The library exports four main functions:

- `extract_file()`: Async function to extract text from a file (accepts string path or `pathlib.Path`)
- `extract_bytes()`: Async function to extract text from bytes (accepts a byte string)
- `extract_file_sync()`: Synchronous version of `extract_file()`
- `extract_bytes_sync()`: Synchronous version of `extract_bytes()`

### Why Async?

Kreuzberg is designed with an async-first approach for several reasons:

1. **I/O Operations**: Text extraction often involves heavy I/O operations (reading files, OCR processing, etc.). Async allows other tasks to run while waiting for these operations.
2. **Scalability**: In web applications or batch processing scenarios, async enables handling multiple extractions concurrently without blocking.
3. **Resource Efficiency**: Async operations make better use of system resources by avoiding thread blocking during I/O-bound operations.

However, we also provide sync methods for simpler use cases or when working in synchronous contexts.

### Quick Start

```python
from pathlib import Path
from kreuzberg import extract_file
from kreuzberg.extraction import ExtractionResult


# Basic file extraction
async def extract_document():
    # Extract from a PDF file
    pdf_result: ExtractionResult = await extract_file("document.pdf")
    print(f"Content: {pdf_result.content}")


    # Extract from an image
    img_result = await extract_file("scan.png")
    print(f"Image text: {img_result.content}")

    # Extract from Word document with metadata
    docx_result = await extract_file(Path("document.docx"))
    if docx_result.metadata:
        print(f"Title: {docx_result.metadata.get('title')}")
        print(f"Author: {docx_result.metadata.get('author')}")
```

### Extracting Bytes

```python
from kreuzberg import extract_bytes
from kreuzberg.extraction import ExtractionResult


async def process_upload(file_content: bytes, mime_type: str) -> ExtractionResult:
    """Process uploaded file content with known MIME type."""
    return await extract_bytes(
        file_content,
        mime_type=mime_type,
    )


# Example usage with different file types
async def handle_uploads(docx_bytes: bytes, pdf_bytes: bytes, image_bytes: bytes):
    # Process PDF upload
    pdf_result = await process_upload(pdf_bytes, mime_type="application/pdf")
    print(f"PDF content: {pdf_result.content}")
    print(f"PDF metadata: {pdf_result.metadata}")

    # Process image upload (will use OCR)
    img_result = await process_upload(image_bytes, mime_type="image/jpeg")
    print(f"Image text: {img_result.content}")

    # Process Word document upload
    docx_result = await process_upload(
        docx_bytes,
        mime_type="application/vnd.openxmlformats-officedocument.wordprocessingml.document"
    )
    print(f"Word content: {docx_result.content}")
```

### Advanced Features

#### Smart PDF Processing

Kreuzberg employs a smart approach to PDF text extraction:

1. **Searchable Text Detection**: First attempts to extract text directly from searchable PDFs using `pdfium2`.

2. **Text Validation**: Extracted text is validated for corruption by checking for:

   - Control and non-printable characters
   - Unicode replacement characters (�)
   - Zero-width spaces and other invisible characters
   - Empty or whitespace-only content

3. **Automatic OCR Fallback**: If the extracted text appears corrupted or if the PDF is image-based, automatically falls back to OCR using Tesseract.

This approach ensures optimal text quality while minimizing unnecessary OCR processing.

#### PDF Processing Options

You can control PDF processing behavior using optional parameters:

```python
from kreuzberg import extract_file


async def process_pdf():
    # Default behavior: auto-detect and use OCR if needed
    # By default, max_tesseract_concurrency=1 for safe operation
    result = await extract_file("document.pdf")
    print(result.content)

    # Force OCR even for searchable PDFs
    result = await extract_file("document.pdf", force_ocr=True)
    print(result.content)

    # Control OCR concurrency for large documents
    # Warning: High concurrency values can cause system resource exhaustion
    # Start with a low value and increase based on your system's capabilities
    result = await extract_file(
        "large_document.pdf",
        max_tesseract_concurrency=4  # Process up to 4 pages concurrently
    )
    print(result.content)

    # Process a scanned PDF (automatically uses OCR)
    result = await extract_file("scanned.pdf")
    print(result.content)
```

#### ExtractionResult Object

All extraction functions return an `ExtractionResult` containing:

- `content`: The extracted text (str)
- `mime_type`: Output format ("text/plain" or "text/markdown" for Pandoc conversions)

```python
from kreuzberg import extract_file, ExtractionResult

async def process_document(path: str) -> tuple[str, str]:
    # Access as a named tuple
    result: ExtractionResult = await extract_file(path)
    print(f"Content: {result.content}")
    print(f"Format: {result.mime_type}")

    # Or unpack as a tuple
    content, mime_type = await extract_file(path)
    return content, mime_type
```

### Error Handling

Kreuzberg provides comprehensive error handling through several exception types, all inheriting from `KreuzbergError`. Each exception includes helpful context information for debugging.

```python
from kreuzberg import extract_file
from kreuzberg.exceptions import (
    ValidationError,
    ParsingError,
    OCRError,
    MissingDependencyError
)

async def safe_extract(path: str) -> str:
    try:
        result = await extract_file(path)
        return result.content

    except ValidationError as e:
        # Input validation issues
        # - Unsupported or undetectable MIME types
        # - Missing files
        # - Invalid input parameters
        print(f"Validation failed: {e}")

    except OCRError as e:
        # OCR-specific issues
        # - Tesseract processing failures
        # - Image conversion problems
        print(f"OCR failed: {e}")

    except MissingDependencyError as e:
        # System dependency issues
        # - Missing Tesseract OCR
        # - Missing Pandoc
        # - Incompatible versions
        print(f"Dependency missing: {e}")

    except ParsingError as e:
        # General processing errors
        # - PDF parsing failures
        # - Format conversion issues
        # - Encoding problems
        print(f"Processing failed: {e}")

    return ""
```

All exceptions provide:

- A descriptive error message
- Relevant context in the `context` attribute
- String representation with both message and context
- Proper exception chaining for debugging

## Roadmap

V1:

- [x] - html file text extraction
- [ ] - better PDF table extraction
- [ ] - batch APIs
- [ ] - sync APIs

V2:

- [ ] - metadata extraction (breaking change)
- [ ] - TBD

## Contribution

This library is open to contribution. Feel free to open issues or submit PRs. Its better to discuss issues before
submitting PRs to avoid disappointment.

### Local Development

1. Clone the repo
2. Install the system dependencies
3. Install the full dependencies with `uv sync`
4. Install the pre-commit hooks with:
   ```shell
   pre-commit install && pre-commit install --hook-type commit-msg
   ```
5. Make your changes and submit a PR

## License

This library uses the MIT license.
