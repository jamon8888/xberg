# Kreuzberg

[![Rust](https://img.shields.io/crates/v/kreuzberg?label=Rust&color=007ec6)](https://crates.io/crates/kreuzberg)
[![Python](https://img.shields.io/pypi/v/kreuzberg?label=Python&color=007ec6)](https://pypi.org/project/kreuzberg/)
[![TypeScript](https://img.shields.io/npm/v/@kreuzberg/node?label=TypeScript&color=007ec6)](https://www.npmjs.com/package/@kreuzberg/node)
[![WASM](https://img.shields.io/npm/v/@kreuzberg/wasm?label=WASM&color=007ec6)](https://www.npmjs.com/package/@kreuzberg/wasm)
[![Ruby](https://img.shields.io/gem/v/kreuzberg?label=Ruby&color=007ec6)](https://rubygems.org/gems/kreuzberg)
[![Java](https://img.shields.io/maven-central/v/dev.kreuzberg/kreuzberg?label=Java&color=007ec6)](https://central.sonatype.com/artifact/dev.kreuzberg/kreuzberg)
[![Go](https://img.shields.io/github/v/tag/kreuzberg-dev/kreuzberg?label=Go&color=007ec6)](https://pkg.go.dev/github.com/kreuzberg-dev/kreuzberg)
[![C#](https://img.shields.io/nuget/v/Goldziher.Kreuzberg?label=C%23&color=007ec6)](https://www.nuget.org/packages/Goldziher.Kreuzberg/)

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Documentation](https://img.shields.io/badge/docs-kreuzberg.dev-007ec6)](https://kreuzberg.dev/)
[![Discord](https://img.shields.io/badge/Discord-Join%20our%20community-007ec6)](https://discord.gg/pXxagNK2zN)

High-performance document intelligence engine for Python built on a Rust core. Extract text, metadata, and structured information from PDFs, Office documents, images, and 56 formats with extensible OCR backends and processing pipelines.


**Powered by a Rust core** – Native performance for document extraction.

> **Version  Release Candidate**
> This is a pre-release version. Please test the library and [report any issues](https://github.com/kreuzberg-dev/kreuzberg/issues) you encounter.

## Installation

```bash
pip install kreuzberg
```

### With OCR Support

```bash
pip install "kreuzberg[easyocr]"
pip install "kreuzberg[paddleocr]"
```

### All Features

```bash
pip install "kreuzberg[all]"
```

## Quick Start

### Basic Usage

```python
import asyncio
from kreuzberg import extract_file, ExtractionConfig

async def main() -> None:
    config = ExtractionConfig(
        use_cache=True,
        enable_quality_processing=True
    )
    result = await extract_file("document.pdf", config=config)
    print(result.content)

asyncio.run(main())
```


### Simple Extraction

```python
import asyncio
from pathlib import Path
from kreuzberg import extract_file

async def main() -> None:
    file_path: Path = Path("document.pdf")

    result = await extract_file(file_path)

    print(f"Content: {result.content}")
    print(f"MIME Type: {result.metadata.format_type}")
    print(f"Tables: {len(result.tables)}")

asyncio.run(main())
```


### Reading Content

```python
import asyncio
from kreuzberg import extract_file

async def main() -> None:
    result = await extract_file("document.pdf")

    content: str = result.content
    tables: int = len(result.tables)
    format_type: str | None = result.metadata.format_type

    print(f"Content length: {len(content)} characters")
    print(f"Tables found: {tables}")
    print(f"Format: {format_type}")

asyncio.run(main())
```


## OCR Support

### Using OCR

```python
import asyncio
from kreuzberg import extract_file, ExtractionConfig, OcrConfig, TesseractConfig

async def main() -> None:
    config = ExtractionConfig(
        force_ocr=True,
        ocr=OcrConfig(
            backend="tesseract",
            language="eng",
            tesseract_config=TesseractConfig(psm=3)
        )
    )
    result = await extract_file("scanned.pdf", config=config)
    print(result.content)
    print(f"Detected Languages: {result.detected_languages}")

asyncio.run(main())
```


### EasyOCR (GPU-Accelerated)

```python
from kreuzberg import extract_file_sync, ExtractionConfig, OcrConfig

config = ExtractionConfig(
    ocr=OcrConfig(backend="easyocr", language="en")
)

result = extract_file_sync(
    "photo.jpg",
    config=config,
    easyocr_kwargs={"use_gpu": True}
)
```

### PaddleOCR (Complex Layouts)

```python
from kreuzberg import extract_file_sync, ExtractionConfig, OcrConfig

config = ExtractionConfig(
    ocr=OcrConfig(backend="paddleocr", language="ch")
)

result = extract_file_sync(
    "invoice.pdf",
    config=config,
    paddleocr_kwargs={"use_angle_cls": True}
)
```

## Table Extraction

```python
from kreuzberg import extract_file_sync, ExtractionConfig, OcrConfig, TesseractConfig

config = ExtractionConfig(
    ocr=OcrConfig(
        backend="tesseract",
        tesseract_config=TesseractConfig(
            enable_table_detection=True
        )
    )
)

result = extract_file_sync("invoice.pdf", config=config)

for table in result.tables:
    print(table.markdown)
    print(table.cells)
```

## Configuration

### Complete Configuration Example

```python
from kreuzberg import (
    extract_file_sync,
    ExtractionConfig,
    OcrConfig,
    TesseractConfig,
    ChunkingConfig,
    ImageExtractionConfig,
    PdfConfig,
    TokenReductionConfig,
    LanguageDetectionConfig,
)

config = ExtractionConfig(
    use_cache=True,
    enable_quality_processing=True,
    ocr=OcrConfig(
        backend="tesseract",
        language="eng",
        tesseract_config=TesseractConfig(
            psm=6,
            enable_table_detection=True,
            min_confidence=50.0,
        ),
    ),
    force_ocr=False,
    chunking=ChunkingConfig(
        max_chars=1000,
        max_overlap=200,
    ),
    images=ImageExtractionConfig(
        extract_images=True,
        target_dpi=300,
        max_image_dimension=4096,
        auto_adjust_dpi=True,
    ),
    pdf_options=PdfConfig(
        extract_images=True,
        passwords=["password1", "password2"],
        extract_metadata=True,
    ),
    token_reduction=TokenReductionConfig(
        mode="moderate",
        preserve_important_words=True,
    ),
    language_detection=LanguageDetectionConfig(
        enabled=True,
        min_confidence=0.8,
        detect_multiple=False,
    ),
)

result = extract_file_sync("document.pdf", config=config)
```

### HTML Conversion Options & Batch Concurrency

```python
from kreuzberg import ExtractionConfig

config = ExtractionConfig(
    max_concurrent_extractions=8,
    html_options={
        "extract_metadata": True,
        "wrap": True,
        "wrap_width": 100,
        "strip_tags": ["script", "style"],
        "preprocessing": {"enabled": True, "preset": "standard"},
    },
)
```

## Metadata Extraction

```python
from kreuzberg import extract_file_sync

result = extract_file_sync("document.pdf")

if result.images:
    print(f"Extracted {len(result.images)} inline images")

if result.chunks:
    print(f"First chunk tokens: {result.chunks[0]['metadata']['token_count']}")

print(result.metadata.get("pdf", {}))
print(result.metadata.get("language"))
print(result.metadata.get("format"))

if "pdf" in result.metadata:
    pdf_meta = result.metadata["pdf"]
    print(f"Title: {pdf_meta.get('title')}")
    print(f"Author: {pdf_meta.get('author')}")
    print(f"Pages: {pdf_meta.get('page_count')}")
    print(f"Created: {pdf_meta.get('creation_date')}")
```

## Password-Protected PDFs

```python
from kreuzberg import extract_file_sync, ExtractionConfig, PdfConfig

config = ExtractionConfig(
    pdf_options=PdfConfig(
        passwords=["password1", "password2", "password3"]
    )
)

result = extract_file_sync("protected.pdf", config=config)
```

## Language Detection

```python
from kreuzberg import extract_file_sync, ExtractionConfig, LanguageDetectionConfig

config = ExtractionConfig(
    language_detection=LanguageDetectionConfig(enabled=True)
)

result = extract_file_sync("multilingual.pdf", config=config)
print(result.detected_languages)
```

## Text Chunking

```python
from kreuzberg import extract_file_sync, ExtractionConfig, ChunkingConfig

config = ExtractionConfig(
    chunking=ChunkingConfig(
        max_chars=1000,
        max_overlap=200,
    )
)

result = extract_file_sync("long_document.pdf", config=config)

for chunk in result.chunks:
    print(chunk)
```

## Extract from Bytes

```python
from kreuzberg import extract_bytes_sync

with open("document.pdf", "rb") as f:
    data = f.read()

result = extract_bytes_sync(data, "application/pdf")
print(result.content)
```

## API Reference

### Extraction Functions

- `extract_file(file_path, mime_type=None, config=None, **kwargs)` – Async extraction
- `extract_file_sync(file_path, mime_type=None, config=None, **kwargs)` – Sync extraction
- `extract_bytes(data, mime_type, config=None, **kwargs)` – Async extraction from bytes
- `extract_bytes_sync(data, mime_type, config=None, **kwargs)` – Sync extraction from bytes
- `batch_extract_files(paths, config=None, **kwargs)` – Async batch extraction
- `batch_extract_files_sync(paths, config=None, **kwargs)` – Sync batch extraction
- `batch_extract_bytes(data_list, mime_types, config=None, **kwargs)` – Async batch from bytes
- `batch_extract_bytes_sync(data_list, mime_types, config=None, **kwargs)` – Sync batch from bytes

### Configuration Classes

- `ExtractionConfig` – Main configuration
- `OcrConfig` – OCR settings
- `TesseractConfig` – Tesseract-specific options
- `ChunkingConfig` – Text chunking settings
- `ImageExtractionConfig` – Image extraction settings
- `PdfConfig` – PDF-specific options
- `TokenReductionConfig` – Token reduction settings
- `LanguageDetectionConfig` – Language detection settings

### Result Types

- `ExtractionResult` – Main result object with `content`, `metadata`, `tables`, `detected_languages`, `chunks`
- `ExtractedTable` – Table with `cells`, `markdown`, `page_number`
- `Metadata` – Typed metadata dictionary

### Exceptions

- `KreuzbergError` – Base exception
- `ValidationError` – Invalid configuration or input
- `ParsingError` – Document parsing failure
- `OCRError` – OCR processing failure
- `MissingDependencyError` – Missing optional dependency

## Examples

### Custom Processing

```python
from kreuzberg import extract_file_sync

result = extract_file_sync("document.pdf")

text = result.content
text = text.lower()
text = text.replace("old", "new")

print(text)
```

### Multiple Files with Progress

```python
from kreuzberg import extract_file_sync
from pathlib import Path

files = list(Path("documents").glob("*.pdf"))
results = []

for i, file in enumerate(files, 1):
    print(f"Processing {i}/{len(files)}: {file.name}")
    result = extract_file_sync(str(file))
    results.append((file.name, result))

for name, result in results:
    print(f"{name}: {len(result.content)} characters")
```

### Filter by Language

```python
from kreuzberg import extract_file_sync, ExtractionConfig, LanguageDetectionConfig

config = ExtractionConfig(
    language_detection=LanguageDetectionConfig(enabled=True)
)

result = extract_file_sync("document.pdf", config=config)

if result.detected_languages and "en" in result.detected_languages:
    print("English document detected")
    print(result.content)
```

## System Requirements

### ONNX Runtime (for embeddings)

If using embeddings functionality, ONNX Runtime version 1.21 or lower must be installed:

```bash
# macOS (pinned to version 1.21)
brew install onnxruntime@1.21

# Ubuntu/Debian (ensure version ≤ 1.21)
sudo apt install libonnxruntime libonnxruntime-dev

# Windows (download version 1.21 or lower)
# Download from https://github.com/microsoft/onnxruntime/releases
```

**Important:** Kreuzberg requires ONNX Runtime version 1.21 or lower. Newer versions may not be compatible.

Without ONNX Runtime, embeddings will raise `MissingDependencyError` with installation instructions.

### Tesseract OCR (Required for OCR)

```bash
brew install tesseract
```

```bash
sudo apt-get install tesseract-ocr
```

### LibreOffice (Optional, for .doc and .ppt)

```bash
brew install libreoffice
```

```bash
sudo apt-get install libreoffice
```

### Pandoc (Optional, for some formats)

```bash
brew install pandoc
```

```bash
sudo apt-get install pandoc
```

## Troubleshooting

### Import Error: No module named '_kreuzberg'

This usually means the Rust extension wasn't built correctly. Try:

```bash
pip install --force-reinstall --no-cache-dir kreuzberg
```

### OCR Not Working

Make sure Tesseract is installed:

```bash
tesseract --version
```

### Memory Issues with Large PDFs

Use streaming or enable chunking:

```python
config = ExtractionConfig(
    chunking=ChunkingConfig(max_chars=1000)
)
```

## PDFium Integration

PDF extraction is powered by PDFium, which is automatically bundled with this package. No system installation required.

### Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| Linux x86_64 | ✅ | Bundled |
| macOS ARM64 | ✅ | Bundled |
| macOS x86_64 | ✅ | Bundled |
| Windows x86_64 | ✅ | Bundled |

### Binary Size Impact

PDFium adds approximately 8-15 MB to the package size depending on platform. This ensures consistent PDF extraction across all environments without external dependencies.

## Documentation

For comprehensive documentation, visit [https://kreuzberg.dev](https://kreuzberg.dev)

## License

 License - see [LICENSE](../../LICENSE) for details.
