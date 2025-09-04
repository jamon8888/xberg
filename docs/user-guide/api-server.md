# API Server

Kreuzberg includes a built-in REST API server powered by [Litestar](https://litestar.dev/) for document extraction over HTTP.

## Installation

Install Kreuzberg with the API extra:

```bash
pip install "kreuzberg[api]"
```

## Running the API Server

### Using Python

```python
from kreuzberg._api.main import app
import uvicorn

uvicorn.run(app, host="0.0.0.0", port=8000)
```

### Using Litestar CLI

```bash
litestar --app kreuzberg._api.main:app run
```

### With Custom Settings

```bash
litestar --app kreuzberg._api.main:app run --host 0.0.0.0 --port 8080
```

## API Endpoints

### Health Check

```bash
GET /health
```

Returns the server status:

```json
{
  "status": "ok"
}
```

### Extract Files

```bash
POST /extract
```

Extract text from one or more files.

**Request:**

- Method: `POST`
- Content-Type: `multipart/form-data`
- Body: One or more files with field name `data`

**Response:**

- Status: 201 Created
- Body: Array of extraction results

**Example:**

```bash
# Single file
curl -X POST http://localhost:8000/extract \
  -F "data=@document.pdf"

# Multiple files
curl -X POST http://localhost:8000/extract \
  -F "data=@document1.pdf" \
  -F "data=@document2.docx" \
  -F "data=@image.jpg"
```

**Response Format:**

```json
[
  {
    "content": "Extracted text content...",
    "mime_type": "text/plain",
    "metadata": {
      "pages": 5,
      "title": "Document Title"
    },
    "chunks": [],
    "entities": null,
    "keywords": null,
    "detected_languages": null
  }
]
```

### Runtime Configuration

The `/extract` endpoint supports runtime configuration via query parameters and HTTP headers, allowing you to customize extraction behavior without requiring static configuration files.

#### Query Parameters

Configure extraction options directly via URL query parameters:

Enable chunking with custom settings:

```bash
curl -X POST "http://localhost:8000/extract?chunk_content=true&max_chars=500&max_overlap=50" \
  -F "data=@document.pdf"
```

Extract entities and keywords:

```bash
curl -X POST "http://localhost:8000/extract?extract_entities=true&extract_keywords=true&keyword_count=5" \
  -F "data=@document.pdf"
```

Force OCR with specific backend:

```bash
curl -X POST "http://localhost:8000/extract?force_ocr=true&ocr_backend=tesseract" \
  -F "data=@image.jpg"
```

Enable language detection:

```bash
curl -X POST "http://localhost:8000/extract?auto_detect_language=true" \
  -F "data=@multilingual_document.pdf"
```

**Supported Query Parameters:**

- `chunk_content` (boolean): Enable content chunking
- `max_chars` (integer): Maximum characters per chunk
- `max_overlap` (integer): Overlap between chunks in characters
- `extract_tables` (boolean): Enable table extraction
- `extract_entities` (boolean): Enable named entity extraction
- `extract_keywords` (boolean): Enable keyword extraction
- `keyword_count` (integer): Number of keywords to extract
- `force_ocr` (boolean): Force OCR processing
- `ocr_backend` (string): OCR engine (`tesseract`, `easyocr`, `paddleocr`)
- `auto_detect_language` (boolean): Enable automatic language detection
- `pdf_password` (string): Password for encrypted PDFs

**Boolean Parameter Formats:**

Query parameters accept flexible boolean values:

- `true`, `false`
- `1`, `0`
- `yes`, `no`
- `on`, `off`

#### Header Configuration

For complex nested configurations, use the `X-Extraction-Config` header with JSON format:

Basic header configuration:

```bash
curl -X POST http://localhost:8000/extract \
  -H "X-Extraction-Config: {\"chunk_content\": true, \"max_chars\": 300, \"extract_keywords\": true}" \
  -F "data=@document.pdf"
```

Advanced OCR configuration:

```bash
curl -X POST http://localhost:8000/extract \
  -H "X-Extraction-Config: {
    \"force_ocr\": true,
    \"ocr_backend\": \"tesseract\",
    \"ocr_config\": {
      \"language\": \"eng+deu\",
      \"psm\": 6,
      \"output_format\": \"text\"
    }
  }" \
  -F "data=@multilingual_document.pdf"
```

Table extraction with GMFT configuration:

```bash
curl -X POST http://localhost:8000/extract \
  -H "X-Extraction-Config: {
    \"extract_tables\": true,
    \"gmft_config\": {
      \"detector_base_threshold\": 0.85,
      \"remove_null_rows\": true,
      \"enable_multi_header\": true
    }
  }" \
  -F "data=@document_with_tables.pdf"
```

#### Configuration Precedence

When multiple configuration sources are present, they are merged with the following precedence:

1. **Header config** (highest priority) - `X-Extraction-Config` header
1. **Query params** - URL query parameters
1. **Static config** - `kreuzberg.toml` or `pyproject.toml` files
1. **Defaults** (lowest priority) - Built-in default values

Header overrides query parameters:

```bash
curl -X POST "http://localhost:8000/extract?max_chars=1000" \
  -H "X-Extraction-Config: {\"max_chars\": 500}" \
  -F "data=@document.pdf"
```

Result: max_chars will be 500 (from header)

#### Error Handling

Invalid configuration returns appropriate error responses:

```bash
# Invalid JSON in header
curl -X POST http://localhost:8000/extract \
  -H "X-Extraction-Config: {invalid-json}" \
  -F "data=@document.pdf"

# Response: 400 Bad Request
{
  "message": "Invalid JSON in X-Extraction-Config header: ...",
  "details": "{\"error\": \"...\"}"
}
```

## Error Handling

The API uses standard HTTP status codes:

- `200 OK`: Successful health check
- `201 Created`: Successful extraction
- `400 Bad Request`: Validation error (e.g., invalid file format)
- `422 Unprocessable Entity`: Parsing error (e.g., corrupted file)
- `500 Internal Server Error`: Unexpected error

Error responses include:

```json
{
  "message": "Error description",
  "details": "{\"additional\": \"context\"}"
}
```

## Features

- **Runtime Configuration**: Configure extraction via query parameters and HTTP headers
- **Batch Processing**: Extract from multiple files in a single request
- **Automatic Format Detection**: Detects file types from MIME types
- **OCR Support**: Automatically applies OCR to images and scanned PDFs
- **Configuration Precedence**: Flexible configuration merging with clear precedence
- **Structured Logging**: Uses structlog for detailed logging
- **OpenTelemetry**: Built-in observability support
- **Async Processing**: High-performance async request handling

## Configuration

The API server uses the default Kreuzberg extraction configuration:

- Tesseract OCR is included by default
- PDF, image, and document extraction is supported
- Table extraction with GMFT (if installed)

To use custom configuration, modify the extraction call in your own API wrapper:

```python
from kreuzberg import ExtractionConfig, batch_extract_bytes
from litestar import Litestar, post

@post("/extract-custom")
async def custom_extract(data: list[UploadFile]) -> list[ExtractionResult]:
    config = ExtractionConfig(force_ocr=True, ocr_backend="easyocr", extract_tables=True)
    return await batch_extract_bytes([(await file.read(), file.content_type) for file in data], config=config)

app = Litestar(route_handlers=[custom_extract])
```

## Production Deployment

For production use, consider:

1. **Reverse Proxy**: Use nginx or similar for SSL termination
1. **Process Manager**: Use systemd, supervisor, or similar
1. **Workers**: Run multiple workers with uvicorn or gunicorn
1. **Monitoring**: Enable OpenTelemetry exporters
1. **Rate Limiting**: Add rate limiting middleware
1. **Authentication**: Add authentication middleware if needed

Example production command:

```bash
uvicorn kreuzberg._api.main:app \
  --host 0.0.0.0 \
  --port 8000 \
  --workers 4 \
  --log-level info
```
