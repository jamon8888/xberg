<?php

declare(strict_types=1);

namespace Kreuzberg\Config;

/**
 * Configuration for document extraction.
 *
 * @example
 * ```php
 * use Kreuzberg\Config\ExtractionConfig;
 * use Kreuzberg\Config\OcrConfig;
 * use Kreuzberg\Config\PdfConfig;
 * use Kreuzberg\Config\ChunkingConfig;
 *
 * $config = new ExtractionConfig(
 *     ocr: new OcrConfig(backend: 'tesseract', language: 'eng'),
 *     pdf: new PdfConfig(extractImages: true),
 *     chunking: new ChunkingConfig(maxChunkSize: 1000),
 * );
 * ```
 */
readonly class ExtractionConfig
{
    public function __construct(
        /**
         * OCR configuration.
         *
         * Configures Optical Character Recognition settings for scanned documents
         * and image-based PDFs. Includes backend selection, language, and Tesseract options.
         *
         * @var OcrConfig|null
         * @default null
         */
        public ?OcrConfig $ocr = null,

        /**
         * PDF extraction configuration.
         *
         * Configures PDF-specific extraction options like image extraction,
         * metadata extraction, OCR fallback, and page range selection.
         *
         * @var PdfConfig|null
         * @default null
         */
        public ?PdfConfig $pdfOptions = null,

        /**
         * Text chunking configuration.
         *
         * Configures how extracted text is split into chunks for processing,
         * including chunk size, overlap, and boundary preservation options.
         *
         * @var ChunkingConfig|null
         * @default null
         */
        public ?ChunkingConfig $chunking = null,

        /**
         * Embedding generation configuration.
         *
         * Configures how text chunks are converted into embeddings (vector representations)
         * for semantic search and similarity matching.
         *
         * @var EmbeddingConfig|null
         * @default null
         */
        public ?EmbeddingConfig $embedding = null,

        /**
         * Image extraction configuration.
         *
         * Configures image extraction parameters such as minimum dimensions
         * and whether to perform OCR on extracted images.
         *
         * @var ImageExtractionConfig|null
         * @default null
         */
        public ?ImageExtractionConfig $images = null,

        /**
         * Page extraction configuration.
         *
         * Configures page-level extraction options including page markers and format.
         *
         * @var PageConfig|null
         * @default null
         */
        public ?PageConfig $pages = null,

        /**
         * Language detection configuration.
         *
         * Configures automatic language detection for document content,
         * including confidence thresholds and maximum languages to detect.
         *
         * @var LanguageDetectionConfig|null
         * @default null
         */
        public ?LanguageDetectionConfig $languageDetection = null,

        /**
         * Keyword extraction configuration.
         *
         * Configures keyword extraction parameters such as maximum number of keywords
         * and minimum relevance score thresholds.
         *
         * @var KeywordConfig|null
         * @default null
         */
        public ?KeywordConfig $keywords = null,

        /**
         * Post-processor configuration.
         *
         * Configures post-processing options for extracted content.
         *
         * @var PostProcessorConfig|null
         * @default null
         */
        public ?PostProcessorConfig $postprocessor = null,

        /**
         * Token reduction configuration.
         *
         * Configures token reduction/optimization for extracted content.
         *
         * @var TokenReductionConfig|null
         * @default null
         */
        public ?TokenReductionConfig $tokenReduction = null,

        /**
         * Enable image extraction from documents.
         *
         * When enabled, images will be extracted from PDFs and other document formats
         * and included in the extraction results.
         *
         * @var bool
         * @default false
         */
        public bool $extractImages = false,

        /**
         * Enable table extraction from documents.
         *
         * When enabled, tables will be detected and extracted from documents
         * with structured formatting preserved.
         *
         * @var bool
         * @default true
         */
        public bool $extractTables = true,

        /**
         * Preserve document formatting in extracted text.
         *
         * When enabled, attempts to preserve original document formatting
         * including indentation, spacing, and structure in the extracted text.
         *
         * @var bool
         * @default false
         */
        public bool $preserveFormatting = false,

        /**
         * Output format for extracted content.
         *
         * Specifies the format for the extracted content. Common values:
         * - 'text': Plain text format
         * - 'markdown': Markdown format with basic formatting
         * - 'html': HTML format with rich formatting
         *
         * @var string|null
         * @default null
         */
        public ?string $outputFormat = null,

        /**
         * Enable caching of extraction results.
         *
         * When enabled, extraction results are cached to avoid redundant processing
         * of the same documents. Cache key is based on document content hash.
         *
         * @var bool
         * @default false
         */
        public bool $useCache = false,

        /**
         * Enable quality processing enhancements.
         *
         * When enabled, applies advanced quality improvement techniques including
         * text smoothing, error correction, and content validation to improve
         * extraction output quality.
         *
         * @var bool
         * @default false
         */
        public bool $enableQualityProcessing = false,

        /**
         * Force OCR on all documents regardless of document type.
         *
         * When enabled, OCR will be applied even to documents that typically
         * have machine-readable text. Useful for ensuring consistent text extraction
         * quality across heterogeneous document collections.
         *
         * @var bool
         * @default false
         */
        public bool $forceOcr = false,

        /**
         * Maximum number of concurrent extraction operations.
         *
         * Controls the degree of parallelism for batch extraction operations.
         * Higher values allow more documents to be processed concurrently but consume more resources.
         * Minimum recommended value: 1, typical range: 2-16.
         *
         * @var int
         * @default 4
         */
        public int $maxConcurrentExtractions = 4,

        /**
         * Result format for structured output.
         *
         * Specifies how results are formatted when structured output is requested.
         * Common values:
         * - 'unified': Single unified format combining all extraction results
         * - 'split': Separate results for text, tables, images, and metadata
         * - 'nested': Hierarchical result structure preserving document structure
         *
         * @var string
         * @default 'unified'
         */
        public string $resultFormat = 'unified',

        /**
         * Output encoding format.
         *
         * Specifies the encoding for text output. Common values:
         * - 'plain': Plain text (UTF-8)
         * - 'json': JSON-encoded text
         * - 'base64': Base64-encoded content
         *
         * @var string
         * @default 'plain'
         */
        public string $outputEncoding = 'plain',

        /**
         * HTML to Markdown conversion options.
         *
         * Configures how HTML documents are converted to Markdown, including heading styles,
         * list formatting, code block styles, and preprocessing options.
         *
         * @var array<string, mixed>|null
         * @default null
         */
        public ?array $htmlOptions = null,
    ) {
    }

    /**
     * Create configuration from array data.
     *
     * @param array<string, mixed> $data
     */
    public static function fromArray(array $data): self
    {
        /** @var bool $extractImages */
        $extractImages = $data['extract_images'] ?? false;
        if (!is_bool($extractImages)) {
            /** @var bool $extractImages */
            $extractImages = (bool) $extractImages;
        }

        /** @var bool $extractTables */
        $extractTables = $data['extract_tables'] ?? true;
        if (!is_bool($extractTables)) {
            /** @var bool $extractTables */
            $extractTables = (bool) $extractTables;
        }

        /** @var bool $preserveFormatting */
        $preserveFormatting = $data['preserve_formatting'] ?? false;
        if (!is_bool($preserveFormatting)) {
            /** @var bool $preserveFormatting */
            $preserveFormatting = (bool) $preserveFormatting;
        }

        /** @var string|null $outputFormat */
        $outputFormat = $data['output_format'] ?? null;
        if ($outputFormat !== null && !is_string($outputFormat)) {
            /** @var string $outputFormat */
            $outputFormat = (string) $outputFormat;
        }

        /** @var bool $useCache */
        $useCache = $data['use_cache'] ?? false;
        if (!is_bool($useCache)) {
            /** @var bool $useCache */
            $useCache = (bool) $useCache;
        }

        /** @var bool $enableQualityProcessing */
        $enableQualityProcessing = $data['enable_quality_processing'] ?? false;
        if (!is_bool($enableQualityProcessing)) {
            /** @var bool $enableQualityProcessing */
            $enableQualityProcessing = (bool) $enableQualityProcessing;
        }

        /** @var bool $forceOcr */
        $forceOcr = $data['force_ocr'] ?? false;
        if (!is_bool($forceOcr)) {
            /** @var bool $forceOcr */
            $forceOcr = (bool) $forceOcr;
        }

        /** @var int $maxConcurrentExtractions */
        $maxConcurrentExtractions = $data['max_concurrent_extractions'] ?? 4;
        if (!is_int($maxConcurrentExtractions)) {
            /** @var int $maxConcurrentExtractions */
            $maxConcurrentExtractions = (int) $maxConcurrentExtractions;
        }

        /** @var string $resultFormat */
        $resultFormat = $data['result_format'] ?? 'unified';
        if (!is_string($resultFormat)) {
            /** @var string $resultFormat */
            $resultFormat = (string) $resultFormat;
        }

        /** @var string $outputEncoding */
        $outputEncoding = $data['output_encoding'] ?? 'plain';
        if (!is_string($outputEncoding)) {
            /** @var string $outputEncoding */
            $outputEncoding = (string) $outputEncoding;
        }

        $ocr = null;
        if (isset($data['ocr']) && is_array($data['ocr'])) {
            /** @var array<string, mixed> $ocrData */
            $ocrData = $data['ocr'];
            $ocr = OcrConfig::fromArray($ocrData);
        }

        $pdf = null;
        if (isset($data['pdf_options']) && is_array($data['pdf_options'])) {
            /** @var array<string, mixed> $pdfData */
            $pdfData = $data['pdf_options'];
            $pdf = PdfConfig::fromArray($pdfData);
        } elseif (isset($data['pdf']) && is_array($data['pdf'])) {
            /** @var array<string, mixed> $pdfData */
            $pdfData = $data['pdf'];
            $pdf = PdfConfig::fromArray($pdfData);
        }

        $chunking = null;
        if (isset($data['chunking']) && is_array($data['chunking'])) {
            /** @var array<string, mixed> $chunkingData */
            $chunkingData = $data['chunking'];
            $chunking = ChunkingConfig::fromArray($chunkingData);
        }

        $embedding = null;
        if (isset($data['embedding']) && is_array($data['embedding'])) {
            /** @var array<string, mixed> $embeddingData */
            $embeddingData = $data['embedding'];
            $embedding = EmbeddingConfig::fromArray($embeddingData);
        }

        $imageExtraction = null;
        if (isset($data['images']) && is_array($data['images'])) {
            /** @var array<string, mixed> $imageExtractionData */
            $imageExtractionData = $data['images'];
            $imageExtraction = ImageExtractionConfig::fromArray($imageExtractionData);
        } elseif (isset($data['image_extraction']) && is_array($data['image_extraction'])) {
            /** @var array<string, mixed> $imageExtractionData */
            $imageExtractionData = $data['image_extraction'];
            $imageExtraction = ImageExtractionConfig::fromArray($imageExtractionData);
        }

        $page = null;
        if (isset($data['pages']) && is_array($data['pages'])) {
            /** @var array<string, mixed> $pageData */
            $pageData = $data['pages'];
            $page = PageConfig::fromArray($pageData);
        } elseif (isset($data['page']) && is_array($data['page'])) {
            /** @var array<string, mixed> $pageData */
            $pageData = $data['page'];
            $page = PageConfig::fromArray($pageData);
        }

        $languageDetection = null;
        if (isset($data['language_detection']) && is_array($data['language_detection'])) {
            /** @var array<string, mixed> $languageDetectionData */
            $languageDetectionData = $data['language_detection'];
            $languageDetection = LanguageDetectionConfig::fromArray($languageDetectionData);
        }

        $keywords = null;
        if (isset($data['keywords']) && is_array($data['keywords'])) {
            /** @var array<string, mixed> $keywordsData */
            $keywordsData = $data['keywords'];
            $keywords = KeywordConfig::fromArray($keywordsData);
        }

        $htmlOptions = null;
        if (isset($data['html_options']) && is_array($data['html_options'])) {
            /** @var array<string, mixed> $htmlOptionsData */
            $htmlOptionsData = $data['html_options'];
            $htmlOptions = $htmlOptionsData;
        }

        $postprocessor = null;
        if (isset($data['postprocessor']) && is_array($data['postprocessor'])) {
            /** @var array<string, mixed> $postprocessorData */
            $postprocessorData = $data['postprocessor'];
            $postprocessor = PostProcessorConfig::fromArray($postprocessorData);
        }

        $tokenReduction = null;
        if (isset($data['token_reduction']) && is_array($data['token_reduction'])) {
            /** @var array<string, mixed> $tokenReductionData */
            $tokenReductionData = $data['token_reduction'];
            $tokenReduction = TokenReductionConfig::fromArray($tokenReductionData);
        }

        return new self(
            ocr: $ocr,
            pdfOptions: $pdf,
            chunking: $chunking,
            embedding: $embedding,
            images: $imageExtraction,
            pages: $page,
            languageDetection: $languageDetection,
            keywords: $keywords,
            extractImages: $extractImages,
            extractTables: $extractTables,
            preserveFormatting: $preserveFormatting,
            outputFormat: $outputFormat,
            useCache: $useCache,
            enableQualityProcessing: $enableQualityProcessing,
            forceOcr: $forceOcr,
            maxConcurrentExtractions: $maxConcurrentExtractions,
            resultFormat: $resultFormat,
            outputEncoding: $outputEncoding,
            htmlOptions: $htmlOptions,
            postprocessor: $postprocessor,
            tokenReduction: $tokenReduction,
        );
    }

    /**
     * Create configuration from JSON string.
     */
    public static function fromJson(string $json): self
    {
        $data = json_decode($json, true);
        if (json_last_error() !== JSON_ERROR_NONE) {
            throw new \InvalidArgumentException('Invalid JSON: ' . json_last_error_msg());
        }
        if (!is_array($data)) {
            throw new \InvalidArgumentException('JSON must decode to an object/array');
        }
        /** @var array<string, mixed> $data */
        return self::fromArray($data);
    }

    /**
     * Simple TOML parser for kreuzberg configuration format.
     *
     * @param string $toml TOML content
     * @return array<string, mixed> Parsed configuration
     */
    private static function parseTOML(string $toml): array
    {
        $result = [];
        $lines = explode("\n", $toml);
        $currentSection = null;

        foreach ($lines as $line) {
            // Remove comments and trim
            if (($commentPos = strpos($line, '#')) !== false) {
                $line = substr($line, 0, $commentPos);
            }
            $line = trim($line);

            if (empty($line)) {
                continue;
            }

            // Parse section header [section_name]
            if (preg_match('/^\[([^\]]+)\]$/', $line, $matches)) {
                $currentSection = $matches[1];
                if (!isset($result[$currentSection])) {
                    $result[$currentSection] = [];
                }
                continue;
            }

            // Parse key = value
            if (preg_match('/^([^=]+)=(.+)$/', $line, $matches)) {
                $key = trim($matches[1]);
                $value = trim($matches[2]);

                // Convert value type
                if (strtolower($value) === 'true') {
                    $value = true;
                } elseif (strtolower($value) === 'false') {
                    $value = false;
                } elseif (is_numeric($value)) {
                    $value = strpos($value, '.') !== false ? (float) $value : (int) $value;
                } else {
                    // Remove quotes if present
                    $value = preg_replace('/^["\']|["\']$/', '', $value);
                }

                if ($currentSection !== null) {
                    /** @var array<string, mixed> $sectionArray */
                    $sectionArray = $result[$currentSection];
                    $sectionArray[$key] = $value;
                    $result[$currentSection] = $sectionArray;
                } else {
                    $result[$key] = $value;
                }
            }
        }

        return $result;
    }

    /**
     * Create configuration from JSON file.
     */
    public static function fromFile(string $path): self
    {
        if (!file_exists($path)) {
            throw new \InvalidArgumentException("File not found: {$path}");
        }
        $contents = file_get_contents($path);
        if ($contents === false) {
            throw new \InvalidArgumentException("Unable to read file: {$path}");
        }

        // Detect format from file extension
        if (str_ends_with($path, '.toml')) {
            $data = self::parseTOML($contents);
        } else {
            // Default to JSON
            $data = json_decode($contents, true);
            if (json_last_error() !== JSON_ERROR_NONE) {
                throw new \InvalidArgumentException('Invalid JSON: ' . json_last_error_msg());
            }
            if (!is_array($data)) {
                throw new \InvalidArgumentException('JSON must decode to an object/array');
            }
        }

        /** @var array<string, mixed> $data */
        return self::fromArray($data);
    }

    /**
     * Discover and load configuration from current or parent directories.
     *
     * Searches for kreuzberg.toml configuration file in the current working
     * directory and parent directories up the filesystem tree. Returns null
     * if no configuration file is found.
     *
     * @return self|null Loaded configuration, or null if not found
     * @throws \InvalidArgumentException If configuration file is invalid
     */
    public static function discover(): ?self
    {
        $cwd = getcwd();
        if ($cwd === false) {
            return null;
        }

        $current = $cwd;
        while (true) {
            $configPath = $current . '/kreuzberg.toml';
            if (file_exists($configPath)) {
                return self::fromFile($configPath);
            }

            $parent = dirname($current);
            if ($parent === $current) {
                // Reached filesystem root
                break;
            }
            $current = $parent;
        }

        return null;
    }

    /**
     * Convert configuration to array for FFI.
     *
     * @return array<string, mixed>
     */
    public function toArray(): array
    {
        // If embedding config is provided but chunking is not, auto-enable chunking
        // so embeddings can be generated for chunks. The Rust pipeline generates
        // embeddings for chunks, not at the document level.
        $chunking = $this->chunking;
        if ($this->embedding !== null) {
            if ($chunking === null) {
                $chunking = new ChunkingConfig(embedding: $this->embedding);
            } elseif ($chunking->embedding === null) {
                // If chunking exists but has no embedding, use the top-level embedding
                $chunking = new ChunkingConfig(
                    maxChars: $chunking->maxChars,
                    maxOverlap: $chunking->maxOverlap,
                    respectSentences: $chunking->respectSentences,
                    respectParagraphs: $chunking->respectParagraphs,
                    embedding: $this->embedding,
                );
            }
        }

        // If extractImages is enabled but imageExtraction config is not provided,
        // auto-create an ImageExtractionConfig to enable image extraction
        $imageExtraction = $this->images;
        if ($this->extractImages && $imageExtraction === null) {
            $imageExtraction = new ImageExtractionConfig(extractImages: true);
        } elseif (!$this->extractImages && $imageExtraction !== null) {
            // If extractImages is false but images config exists, respect the config
            $imageExtraction = $this->images;
        }

        $result = [
            'ocr' => $this->ocr?->toArray(),
            'pdf_options' => $this->pdfOptions?->toArray(),
            'chunking' => $chunking?->toArray(),
            'embedding' => $this->embedding?->toArray(),
            'images' => $imageExtraction?->toArray(),
            'pages' => $this->pages?->toArray(),
            'language_detection' => $this->languageDetection?->toArray(),
            'keywords' => $this->keywords?->toArray(),
            'html_options' => $this->htmlOptions,
            'postprocessor' => $this->postprocessor?->toArray(),
            'token_reduction' => $this->tokenReduction?->toArray(),
        ];

        // Add simple boolean/string fields only if explicitly set to non-default values
        if ($this->extractImages) {
            $result['extract_images'] = true;
        }
        if (!$this->extractTables) {
            $result['extract_tables'] = false;
        }
        if ($this->preserveFormatting) {
            $result['preserve_formatting'] = true;
        }
        if ($this->outputFormat !== null) {
            $result['output_format'] = $this->outputFormat;
        }
        if ($this->useCache) {
            $result['use_cache'] = true;
        }
        if ($this->enableQualityProcessing) {
            $result['enable_quality_processing'] = true;
        }
        if ($this->forceOcr) {
            $result['force_ocr'] = true;
        }
        if ($this->maxConcurrentExtractions !== 4) {
            $result['max_concurrent_extractions'] = $this->maxConcurrentExtractions;
        }
        if ($this->resultFormat !== 'unified') {
            $result['result_format'] = $this->resultFormat;
        }
        if ($this->outputEncoding !== 'plain') {
            $result['output_encoding'] = $this->outputEncoding;
        }

        return array_filter($result, static fn ($value): bool => $value !== null);
    }

    /**
     * Convert configuration to JSON string.
     */
    public function toJson(): string
    {
        $json = json_encode($this->toArray(), JSON_PRETTY_PRINT);
        if ($json === false) {
            throw new \RuntimeException('Failed to encode configuration to JSON');
        }
        return $json;
    }

    /**
     * Create a new configuration builder instance.
     *
     * @return ExtractionConfigBuilder A builder for creating ExtractionConfig instances
     */
    public static function builder(): ExtractionConfigBuilder
    {
        return new ExtractionConfigBuilder();
    }
}
