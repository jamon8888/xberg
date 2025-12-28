# Kreuzberg for Go

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

High-performance document intelligence engine for Go built on the Rust core that powers every Kreuzberg binding. Extract text, tables, images, and metadata from 56 formats with native Go API and context-aware async support.


> **Version  Release Candidate**
> This binding targets the  APIs. Report issues at [github.com/kreuzberg-dev/kreuzberg](https://github.com/kreuzberg-dev/kreuzberg).

## Install

```bash
go get github.com/kreuzberg-dev/kreuzberg/packages/go/v4@latest
```

The Go binding uses cgo to link against the `kreuzberg-ffi` library.

## Quick Start

### Basic Usage

```go
package main

import (
	"fmt"
	"github.com/kreuzberg-dev/kreuzberg/packages/go/pkg/kreuzberg"
)

func main() {
	result, err := kreuzberg.ExtractFile("document.pdf")
	if err != nil {
		panic(err)
	}

	fmt.Println(result.Content)
}
```

### Async Extraction

```go
package main

import (
	"context"
	"fmt"
	"github.com/kreuzberg-dev/kreuzberg/packages/go/pkg/kreuzberg"
)

func main() {
	ctx := context.Background()
	result, err := kreuzberg.ExtractFileContext(ctx, "document.pdf")
	if err != nil {
		panic(err)
	}

	fmt.Println(result.Content)
}
```

### Batch Processing

```go
package main

import (
	"fmt"
	"github.com/kreuzberg-dev/kreuzberg/packages/go/pkg/kreuzberg"
)

func main() {
	files := []string{"doc1.pdf", "doc2.docx", "doc3.xlsx"}

	for _, file := range files {
		result, err := kreuzberg.ExtractFile(file)
		if err != nil {
			fmt.Printf("Error processing %s: %v\n", file, err)
			continue
		}

		fmt.Printf("%s: %d characters\n", file, len(result.Content))
	}
}
```

## Configuration

```go
package main

import (
	"context"
	"fmt"
	"github.com/kreuzberg-dev/kreuzberg/packages/go/pkg/kreuzberg"
)

func main() {
	ctx := context.Background()

	config := &kreuzberg.ExtractionConfig{
		UseCache: true,
		EnableQualityProcessing: true,
		OCR: &kreuzberg.OcrConfig{
			Backend: "tesseract",
			Language: "eng",
		},
	}

	result, err := kreuzberg.ExtractFileWithConfig(ctx, "document.pdf", config)
	if err != nil {
		panic(err)
	}

	fmt.Println(result.Content)
}
```

## OCR Support

### Tesseract Configuration

```go
package main

import (
	"fmt"
	"github.com/kreuzberg-dev/kreuzberg/packages/go/pkg/kreuzberg"
)

func main() {
	config := &kreuzberg.ExtractionConfig{
		OCR: &kreuzberg.OcrConfig{
			Backend: "tesseract",
			Language: "eng",
			TesseractConfig: &kreuzberg.TesseractConfig{
				PSM: 6,
				EnableTableDetection: true,
			},
		},
	}

	result, err := kreuzberg.ExtractFileWithConfig(nil, "scanned.pdf", config)
	if err != nil {
		panic(err)
	}

	fmt.Println(result.Content)
}
```

## Metadata Extraction

```go
package main

import (
	"fmt"
	"github.com/kreuzberg-dev/kreuzberg/packages/go/pkg/kreuzberg"
)

func main() {
	result, err := kreuzberg.ExtractFile("document.pdf")
	if err != nil {
		panic(err)
	}

	// Access metadata
	if metadata, ok := result.Metadata["pdf"].(map[string]interface{}); ok {
		if title, ok := metadata["title"].(string); ok {
			fmt.Printf("Title: %s\n", title)
		}
	}

	// Access tables
	for i, table := range result.Tables {
		fmt.Printf("Table %d:\n%s\n", i, table.Markdown)
	}
}
```

## API Reference

### Main Functions

- `ExtractFile(filePath string) (*ExtractionResult, error)` – Synchronous extraction
- `ExtractFileContext(ctx context.Context, filePath string) (*ExtractionResult, error)` – Context-aware extraction
- `ExtractFileWithConfig(ctx context.Context, filePath string, config *ExtractionConfig) (*ExtractionResult, error)` – With configuration
- `ExtractBytes(ctx context.Context, data []byte, mimeType string) (*ExtractionResult, error)` – From bytes
- `BatchExtractFiles(ctx context.Context, filePaths []string) ([]*ExtractionResult, error)` – Batch processing

### Configuration

- `ExtractionConfig` – Main configuration struct
- `OcrConfig` – OCR settings
- `TesseractConfig` – Tesseract-specific options
- `ChunkingConfig` – Text chunking settings

### Result Types

- `ExtractionResult` – Result with `Content`, `Metadata`, `Tables`, `DetectedLanguages`
- `ExtractedTable` – Table with `Cells`, `Markdown`, `PageNumber`

## Platform-Specific Build Instructions

### Linux/macOS

1. Build the Rust FFI crate:
   ```bash
   cargo build -p kreuzberg-ffi --release
   ```

2. Ensure the library is discoverable:
   ```bash
   export LD_LIBRARY_PATH=$PWD/target/release
   ```

### Windows (MinGW)

1. Build with core feature:
   ```bash
   cargo build -p kreuzberg-ffi --release --target x86_64-pc-windows-gnu --no-default-features --features core
   ```

2. Add to PATH:
   ```bash
   set PATH=%PATH%;target\x86_64-pc-windows-gnu\release
   ```

## Troubleshooting

### cgo compilation errors

Ensure you have the necessary build tools:

```bash
# macOS
xcode-select --install

# Ubuntu/Debian
sudo apt-get install build-essential

# Windows
# Install GCC via mingw-w64 or use MSVC toolchain
```

### Library not found at runtime

Set the appropriate library path:

```bash
# Linux
export LD_LIBRARY_PATH=$PWD/target/release

# macOS
export DYLD_FALLBACK_LIBRARY_PATH=$PWD/target/release

# Windows
set PATH=%PATH%;target\release
```

## Documentation

For comprehensive documentation, visit [https://kreuzberg.dev](https://kreuzberg.dev)

## License

 License - see [LICENSE](../../LICENSE) for details.
