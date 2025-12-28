# Kreuzberg for Ruby

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

High-performance document intelligence engine for Ruby built on a Rust core. Extract text, tables, images, and metadata from 56 file formats including PDF, DOCX, PPTX, XLSX, images, and more.


> **Version  Release Candidate**
> This is a pre-release version. Please test the gem and [report any issues](https://github.com/kreuzberg-dev/kreuzberg/issues) you encounter.

## Installation

Add to your Gemfile:

```ruby
gem 'kreuzberg'
```

Then execute:

```bash
bundle install
```

Or install it directly:

```bash
gem install kreuzberg
```

## Quick Start

### Basic Usage

```ruby
require 'kreuzberg'

# Simple synchronous extraction
result = Kreuzberg.extract_file("document.pdf")
puts result.content
```

### Async Extraction

```ruby
require 'kreuzberg'

# Using Fiber for concurrency (Ruby 3.0+)
Fiber.new do
  result = Kreuzberg.extract_file_async("document.pdf")
  puts result.content
end.resume
```

### Batch Processing

```ruby
require 'kreuzberg'

files = ["doc1.pdf", "doc2.docx", "doc3.xlsx"]

results = files.map { |file| Kreuzberg.extract_file(file) }

results.each do |result|
  puts "Content length: #{result.content.length}"
end
```

## Configuration

```ruby
require 'kreuzberg'

config = Kreuzberg::ExtractionConfig.new(
  use_cache: true,
  enable_quality_processing: true,
  ocr: Kreuzberg::OcrConfig.new(
    backend: 'tesseract',
    language: 'eng'
  )
)

result = Kreuzberg.extract_file("document.pdf", config: config)
puts result.content
```

## OCR Support

### Tesseract Configuration

```ruby
require 'kreuzberg'

config = Kreuzberg::ExtractionConfig.new(
  ocr: Kreuzberg::OcrConfig.new(
    backend: 'tesseract',
    language: 'eng',
    tesseract_config: Kreuzberg::TesseractConfig.new(
      psm: 6,
      enable_table_detection: true
    )
  )
)

result = Kreuzberg.extract_file("scanned.pdf", config: config)
puts result.content
```

## Table Extraction

```ruby
require 'kreuzberg'

config = Kreuzberg::ExtractionConfig.new(
  ocr: Kreuzberg::OcrConfig.new(
    backend: 'tesseract',
    tesseract_config: Kreuzberg::TesseractConfig.new(
      enable_table_detection: true
    )
  )
)

result = Kreuzberg.extract_file("invoice.pdf", config: config)

result.tables.each_with_index do |table, index|
  puts "Table #{index}:"
  puts table.markdown
end
```

## Metadata Extraction

```ruby
require 'kreuzberg'

result = Kreuzberg.extract_file("document.pdf")

# PDF metadata
if result.metadata[:pdf]
  pdf_meta = result.metadata[:pdf]
  puts "Title: #{pdf_meta[:title]}"
  puts "Author: #{pdf_meta[:author]}"
  puts "Pages: #{pdf_meta[:page_count]}"
end

# Detected languages
puts "Languages: #{result.detected_languages}"

# Images
if result.images
  puts "Images found: #{result.images.count}"
end
```

## Text Chunking

```ruby
require 'kreuzberg'

config = Kreuzberg::ExtractionConfig.new(
  chunking: Kreuzberg::ChunkingConfig.new(
    max_chars: 1000,
    max_overlap: 200
  )
)

result = Kreuzberg.extract_file("long_document.pdf", config: config)

result.chunks.each_with_index do |chunk, index|
  puts "Chunk #{index}: #{chunk.length} characters"
end
```

## Password-Protected PDFs

```ruby
require 'kreuzberg'

config = Kreuzberg::ExtractionConfig.new(
  pdf_options: Kreuzberg::PdfConfig.new(
    passwords: ["password1", "password2"]
  )
)

result = Kreuzberg.extract_file("protected.pdf", config: config)
puts result.content
```

## Language Detection

```ruby
require 'kreuzberg'

config = Kreuzberg::ExtractionConfig.new(
  language_detection: Kreuzberg::LanguageDetectionConfig.new(
    enabled: true
  )
)

result = Kreuzberg.extract_file("multilingual.pdf", config: config)
puts "Detected languages: #{result.detected_languages}"
```

## API Reference

### Main Methods

- `Kreuzberg.extract_file(path, config: nil)` – Extract from file
- `Kreuzberg.extract_file_async(path, config: nil)` – Async extraction
- `Kreuzberg.extract_bytes(data, mime_type, config: nil)` – Extract from bytes
- `Kreuzberg.batch_extract_files(paths, config: nil)` – Batch processing

### Configuration Classes

- `ExtractionConfig` – Main configuration
- `OcrConfig` – OCR settings
- `TesseractConfig` – Tesseract-specific options
- `ChunkingConfig` – Text chunking settings
- `PdfConfig` – PDF-specific options
- `LanguageDetectionConfig` – Language detection settings

### Result Object

- `content` – Extracted text
- `metadata` – File metadata as Hash
- `tables` – Array of ExtractedTable objects
- `detected_languages` – Array of language codes
- `chunks` – Array of text chunks
- `images` – Array of extracted images (if enabled)

## System Requirements

### Required

- Ruby 2.7 or higher
- Rust toolchain (for native extension compilation)

### Optional

```bash
# Tesseract OCR
brew install tesseract          # macOS
sudo apt-get install tesseract-ocr  # Ubuntu/Debian

# LibreOffice (for .doc, .ppt, etc.)
brew install libreoffice        # macOS
sudo apt-get install libreoffice    # Ubuntu/Debian
```

## Development

Clone and setup:

```bash
git clone https://github.com/kreuzberg-dev/kreuzberg.git
cd kreuzberg
bundle install
```

Run tests:

```bash
rake test
```

## Troubleshooting

### Native extension compilation error

Ensure build tools are installed:

```bash
# macOS
xcode-select --install

# Ubuntu/Debian
sudo apt-get install build-essential ruby-dev

# Windows (via RubyInstaller)
ridk install
```

### "Could not find Kreuzberg"

Reinstall the gem:

```bash
gem uninstall kreuzberg
gem install kreuzberg --no-document
```

### OCR not working

Verify Tesseract is installed:

```bash
tesseract --version
```

## Examples

### Process Directory of PDFs

```ruby
require 'kreuzberg'
require 'pathname'

Dir.glob("documents/*.pdf").each do |file|
  puts "Processing: #{file}"
  result = Kreuzberg.extract_file(file)
  puts "  Content length: #{result.content.length}"
  puts "  Language: #{result.detected_languages}"
end
```

### Extract and Parse Structured Data

```ruby
require 'kreuzberg'
require 'json'

result = Kreuzberg.extract_file("data.pdf")

# Parse content as JSON (if applicable)
begin
  data = JSON.parse(result.content)
  puts "Parsed data: #{data}"
rescue JSON::ParserError
  puts "Content is not JSON"
end
```

### Save Extracted Images

```ruby
require 'kreuzberg'

config = Kreuzberg::ExtractionConfig.new(
  images: Kreuzberg::ImageExtractionConfig.new(
    extract_images: true
  )
)

result = Kreuzberg.extract_file("document.pdf", config: config)

result.images&.each_with_index do |image, index|
  File.write("image_#{index}.png", image.data)
end
```

## Documentation

For comprehensive documentation, visit [https://kreuzberg.dev](https://kreuzberg.dev)

## License

 License - see [LICENSE](../../LICENSE) for details.
