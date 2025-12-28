# Kreuzberg.dev

[![Rust](https://img.shields.io/crates/v/kreuzberg?label=Rust)](https://crates.io/crates/kreuzberg)
[![Elixir/Hex](https://img.shields.io/hexpm/v/kreuzberg?label=Elixir)](https://hex.pm/packages/kreuzberg)
[![Python](https://img.shields.io/pypi/v/kreuzberg?label=Python)](https://pypi.org/project/kreuzberg/)
[![TypeScript/Node.js](https://img.shields.io/npm/v/@kreuzberg/node?label=TypeScript%2FNode.js&color=3178c6)](https://www.npmjs.com/package/@kreuzberg/node)
[![Browser/WASM](https://img.shields.io/npm/v/@kreuzberg/wasm?label=Browser%2FWASM&color=654ff0)](https://www.npmjs.com/package/@kreuzberg/wasm)
[![Ruby](https://img.shields.io/gem/v/kreuzberg?label=Ruby)](https://rubygems.org/gems/kreuzberg)
[![Java](https://img.shields.io/maven-central/v/dev.kreuzberg/kreuzberg?label=Java)](https://central.sonatype.com/artifact/dev.kreuzberg/kreuzberg)
[![Go](https://img.shields.io/github/v/tag/kreuzberg-dev/kreuzberg?label=Go)](https://pkg.go.dev/github.com/kreuzberg-dev/kreuzberg)
[![C#](https://img.shields.io/nuget/v/Goldziher.Kreuzberg?label=C%23)](https://www.nuget.org/packages/Goldziher.Kreuzberg/)

<img width="1128" height="191" alt="Banner2" src="https://github.com/user-attachments/assets/419fc06c-8313-4324-b159-4b4d3cfce5c0" />



[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Documentation](https://img.shields.io/badge/docs-kreuzberg.dev-blue)](https://kreuzberg.dev/)
[![Discord](https://img.shields.io/badge/Discord-Join%20our%20community-7289da)](https://discord.gg/pXxagNK2zN)

A polyglot document intelligence framework with a Rust core. Extract text, metadata, and structured information from PDFs, Office documents, images, and 56 formats. Available for Rust, Python, TypeScript/Node.js, Ruby, Go, Java, and C#—or use via CLI, REST API, or MCP server.

> **Version 4.0.0 Release Candidate**
> This is a pre-release version. Please test the library and [report any issues](https://github.com/kreuzberg-dev/kreuzberg/issues) you encounter.

> **⚠️ BREAKING CHANGE in RC.11**
>
> Embeddings now require separate ONNX Runtime installation. This reduces package sizes and enables Windows MSVC support.
>
> - **Action required**: Install ONNX Runtime if you use embeddings
> - **No action needed**: If you don't use embeddings
> - [Installation Guide](#embeddings-support-optional)

## Key Features

- **Polyglot** – Native bindings for Rust, Python, TypeScript/Node.js, Ruby, Go, Java, C#
- **56 file formats** – PDF, Office documents, images, HTML, XML, emails, archives, and more
- **OCR support** – Multiple backends (Tesseract, EasyOCR, PaddleOCR) with table extraction
- **Flexible deployment** – Use as library, CLI tool, REST API server, or MCP server
- **Memory efficient** – Streaming parsers for multi-GB files

**[Complete Documentation](https://kreuzberg.dev/)** | **[Installation Guides](#installation)**

## Kreuzberg Cloud (Coming Soon)

Don't want to manage Rust infrastructure? **Kreuzberg Cloud** is a managed document extraction API launching at the beginning of 2026.

- Hosted REST API with async jobs and webhooks
- Built-in chunking and embeddings for RAG pipelines
- Premium OCR backends for 95%+ accuracy
- No infrastructure to maintain

## Installation

Each language binding provides comprehensive documentation with examples and best practices. Choose your platform to get started:

### JavaScript/TypeScript

- **[@kreuzberg/node](https://github.com/kreuzberg-dev/kreuzberg/tree/main/crates/kreuzberg-node)** (Recommended for Node.js/Bun) – Native NAPI-RS bindings, fastest performance, direct system calls
- **[@kreuzberg/wasm](https://github.com/kreuzberg-dev/kreuzberg/tree/main/packages/typescript)** (Browser/Workers/Deno) – Pure WebAssembly, no native dependencies, cross-platform consistency

**TypeScript Decision Matrix:**

| Platform | Package | Performance | Setup | Use Case |
|----------|---------|-------------|-------|----------|
| Node.js | `@kreuzberg/node` | Fastest (100%) | Native build toolchain | Production servers, backends |
| Bun | `@kreuzberg/node` | Fastest (100%) | Native build toolchain | High-performance backends |
| Browser | `@kreuzberg/wasm` | Good (60-80% of native) | Zero dependencies | Web apps, no build complexity |
| Cloudflare Workers | `@kreuzberg/wasm` | Good (60-80% of native) | Zero dependencies | Serverless edge computing |
| Deno | `@kreuzberg/wasm` | Good (60-80% of native) | Zero dependencies | Deno runtime |

### Other Languages

- **[Python](https://github.com/kreuzberg-dev/kreuzberg/tree/main/packages/python)** – Installation, basic usage, async/sync APIs
- **[Ruby](https://github.com/kreuzberg-dev/kreuzberg/tree/main/packages/ruby)** – Installation, basic usage, configuration
- **[Go](https://github.com/kreuzberg-dev/kreuzberg/tree/main/packages/go)** – Installation, native library setup, sync/async extraction + batch APIs
  _Note: Windows builds use MinGW and don't support embeddings (ONNX Runtime requires MSVC)_
- **[Java](https://github.com/kreuzberg-dev/kreuzberg/tree/main/packages/java)** – Installation, FFM API usage, Maven/Gradle setup
- **[C#](https://github.com/kreuzberg-dev/kreuzberg/tree/main/packages/csharp)** – Installation, P/Invoke usage, NuGet package
- **[Rust](https://github.com/kreuzberg-dev/kreuzberg/tree/main/crates/kreuzberg)** – Crate usage, features, async/sync APIs
- **[CLI](https://kreuzberg.dev/cli/usage/)** – Command-line usage, batch processing, options

### Embeddings Support (Optional)

To use embeddings functionality:

1. **Install ONNX Runtime**:
   - Linux: `apt install libonnxruntime`
   - macOS: `brew install onnxruntime`
   - Windows: `scoop install onnxruntime` or `winget install onnxruntime`

2. Use embeddings in your code - see [Embeddings Guide](https://kreuzberg.dev/features/#embeddings)

Note: All other Kreuzberg features work without ONNX Runtime.

## PDFium Linking Options (Rust Crate Only)

The Rust crate offers flexible PDFium linking strategies for different deployment scenarios. Language bindings (Python, TypeScript, Ruby, Java, Go, C#) always bundle PDFium automatically—no configuration needed.

| Strategy | Feature Flag | Use Case |
|----------|--------------|----------|
| **Default (Dynamic)** | None | Download at build time, link dynamically. Simplest option. |
| **Static** | `pdf-static` | Download at build time, link statically. Useful for isolated deployments. |
| **Bundled** | `pdf-bundled` | Embed PDFium in binary. Largest binary, but zero external dependencies. |
| **System** | `pdf-system` | Use system-installed PDFium via `pkg-config`. Best for package managers. |

**Example Cargo.toml configurations:**

```toml
# Default (dynamic linking)
[dependencies]
kreuzberg = "4.0"

# Static linking
[dependencies]
kreuzberg = { version = "4.0", features = ["pdf-static"] }

# Bundled in binary
[dependencies]
kreuzberg = { version = "4.0", features = ["pdf-bundled"] }

# System library
[dependencies]
kreuzberg = { version = "4.0", features = ["pdf-system"] }
```

## Supported Formats

### Documents & Productivity

| Format | Extensions | Metadata | Tables | Images |
|--------|-----------|----------|--------|--------|
| PDF | `.pdf` | ✅ | ✅ | ✅ |
| Word | `.docx`, `.doc` | ✅ | ✅ | ✅ |
| Excel | `.xlsx`, `.xls`, `.ods` | ✅ | ✅ | ❌ |
| PowerPoint | `.pptx`, `.ppt` | ✅ | ✅ | ✅ |
| Rich Text | `.rtf` | ✅ | ❌ | ❌ |
| EPUB | `.epub` | ✅ | ❌ | ❌ |

### Images

All image formats support OCR: `.jpg`, `.jpeg`, `.png`, `.tiff`, `.tif`, `.bmp`, `.gif`, `.webp`, `.jp2`

### Web & Structured Data

| Format | Extensions | Features |
|--------|-----------|----------|
| HTML | `.html`, `.htm` | Metadata extraction, link preservation |
| XML | `.xml` | Streaming parser for multi-GB files |
| JSON | `.json` | Intelligent field detection |
| YAML | `.yaml` | Structure preservation |
| TOML | `.toml` | Configuration parsing |

### Email & Archives

| Format | Extensions | Features |
|--------|-----------|----------|
| Email | `.eml`, `.msg` | Full metadata, attachment extraction |
| Archives | `.zip`, `.tar`, `.gz`, `.7z` | File listing, metadata |

### Academic & Technical

LaTeX (`.tex`), BibTeX (`.bib`), Jupyter (`.ipynb`), reStructuredText (`.rst`), Org Mode (`.org`), Markdown (`.md`)

**[Complete Format Documentation](https://kreuzberg.dev/reference/formats/)**

## Key Features

### OCR with Table Extraction

Multiple OCR backends (Tesseract, EasyOCR, PaddleOCR) with intelligent table detection and reconstruction. Extract structured data from scanned documents and images with configurable accuracy thresholds.

**[OCR Backend Documentation →](https://kreuzberg.dev/guides/ocr/)**

### Batch Processing

Process multiple documents concurrently with configurable parallelism. Optimize throughput for large-scale document processing workloads with automatic resource management.

**[Batch Processing Guide →](https://kreuzberg.dev/features/#batch-processing)**

### Password-Protected PDFs

Handle encrypted PDFs with single or multiple password attempts. Supports both RC4 and AES encryption with automatic fallback strategies.

**[PDF Configuration →](https://kreuzberg.dev/migration/v3-to-v4/#password-protected-pdfs)**

### Language Detection

Automatic language detection in extracted text using fast-langdetect. Configure confidence thresholds and access per-language statistics.

**[Language Detection Guide →](https://kreuzberg.dev/features/#language-detection)**

### Metadata Extraction

Extract comprehensive metadata from all supported formats: authors, titles, creation dates, page counts, EXIF data, and format-specific properties.

**[Metadata Guide →](https://kreuzberg.dev/reference/types/#metadata)**

## Performance: Native vs WASM Bindings

Kreuzberg offers two JavaScript/TypeScript options with different performance characteristics:

| Metric | @kreuzberg/node (Native) | @kreuzberg/wasm (WASM) |
|--------|--------------------------|------------------------|
| **Single document extraction** | ~150ms (PDF, 10 pages) | ~240-250ms (60-80% of native) |
| **Batch processing (10 docs)** | ~850ms | ~1400-1800ms |
| **Memory usage** | Direct system calls | Browser/WASM runtime overhead |
| **Native dependencies** | Required (OS libraries) | None |
| **Browser support** | Node.js, Bun only | Browser, Workers, Deno |
| **Setup complexity** | Native build toolchain | Zero dependencies |

**Use @kreuzberg/node when:**
- Running on Node.js or Bun backend servers
- Performance is critical (2-3x faster than WASM)
- You have or can install native build toolchain
- Processing high document volumes

**Use @kreuzberg/wasm when:**
- Running in browser or web workers
- Deploying to serverless edge (Cloudflare, Vercel)
- Using Deno or similar runtimes
- Need absolute zero native dependencies
- Cross-platform consistency is important
- Trade-off: ~20-40% slower but still good performance

## Deployment Options

### REST API Server

Production-ready API server with OpenAPI documentation, health checks, and telemetry support. Deploy standalone or in containers with automatic format detection and streaming support.

**[API Server Documentation →](https://kreuzberg.dev/guides/api-server/)**

### MCP Server (AI Integration)

Model Context Protocol server for Claude and other AI assistants. Enables AI agents to extract and process documents directly with full configuration support.

**[MCP Server Documentation →](https://kreuzberg.dev/guides/api-server/#mcp-server_1)**

### Docker

Official Docker images available in multiple variants:

- **Core** (~1.0-1.3GB): Tesseract OCR, modern Office formats
- **Full** (~1.5-2.1GB): Adds LibreOffice for legacy Office formats (.doc, .ppt)

All images support API server, CLI, and MCP server modes with automatic platform detection for linux/amd64 and linux/arm64.

**[Docker Deployment Guide →](https://kreuzberg.dev/guides/docker/)**

## Comparison with Alternatives

| Feature | Kreuzberg | docling | unstructured | LlamaParse |
|---------|-----------|---------|--------------|------------|
| **Formats** | 56 | PDF, DOCX | 30+ | PDF only |
| **Self-hosted** | ✅ Yes (MIT) | ✅ Yes | ✅ Yes | ❌ API only |
| **Programming Languages** | Rust, Python, Ruby, TS, Java, Go, C# | Python | Python | API (any) |
| **Table Extraction** | ✅ Good | ✅ Good | ✅ Basic | ✅ Excellent |
| **OCR** | ✅ Multiple backends | ✅ Yes | ✅ Yes | ✅ Yes |
| **Embeddings** | ✅ Built-in | ❌ No | ❌ No | ❌ No |
| **Chunking** | ✅ Built-in | ❌ No | ✅ Yes | ❌ No |
| **Cost** | Free (MIT) | Free (MIT) | Free (Apache 2.0) | $0.003/page |
| **Air-gap deployments** | ✅ Yes | ✅ Yes | ✅ Yes | ❌ No |

**When to use Kreuzberg:**
- ✅ Need high throughput (thousands of documents)
- ✅ Memory-constrained environments
- ✅ Non-Python ecosystems (Ruby, TypeScript, Java, Go)
- ✅ RAG pipelines (built-in chunking + embeddings)
- ✅ Self-hosted or air-gapped deployments
- ✅ Multi-GB files requiring streaming

**When to consider alternatives:**
- **LlamaParse**: If you need best-in-class table extraction and only process PDFs (requires internet, paid)
- **docling**: If you're Python-only and don't need extreme performance
- **unstructured**: If you need extensive pre-built integrations with vector databases

## Architecture

Kreuzberg is built with a Rust core for efficient document extraction and processing.

### Design Principles

- **Rust core** – Native code for text extraction and processing
- **Async throughout** – Asynchronous processing with Tokio runtime
- **Memory efficient** – Streaming parsers for large files
- **Parallel batch processing** – Configurable concurrency for multiple documents
- **Zero-copy operations** – Efficient data handling where possible

## Documentation

- **[Installation Guide](https://kreuzberg.dev/getting-started/installation/)** – Setup and dependencies
- **[User Guide](https://kreuzberg.dev/guides/extraction/)** – Comprehensive usage guide
- **[API Reference](https://kreuzberg.dev/reference/api-python/)** – Complete API documentation
- **[Format Support](https://kreuzberg.dev/reference/formats/)** – Supported file formats
- **[OCR Backends](https://kreuzberg.dev/guides/ocr/)** – OCR engine setup
- **[CLI Guide](https://kreuzberg.dev/cli/usage/)** – Command-line usage
- **[Migration Guide](https://kreuzberg.dev/migration/v3-to-v4/)** – Upgrading from v3

## Contributing

Contributions are welcome! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

MIT License - see [LICENSE](LICENSE) for details. You can use Kreuzberg freely in both commercial and closed-source products with no obligations, no viral effects, and no licensing restrictions.
