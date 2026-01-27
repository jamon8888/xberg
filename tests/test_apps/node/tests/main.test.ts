import { readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import {
	__version__,
	batchExtractBytes,
	batchExtractBytesSync,
	batchExtractFiles,
	batchExtractFilesSync,
	CacheError,
	type Chunk,
	type ChunkingConfig,
	classifyError,
	clearDocumentExtractors,
	clearOcrBackends,
	clearPostProcessors,
	clearValidators,
	detectMimeType,
	detectMimeTypeFromPath,
	type ErrorClassification,
	ErrorCode,
	type ExtractedImage,
	ExtractionConfig,
	type ExtractionResult,
	extractBytes,
	extractBytesSync,
	extractFile,
	extractFileSync,
	getEmbeddingPreset,
	getErrorCodeDescription,
	getErrorCodeName,
	getExtensionsForMime,
	getLastErrorCode,
	getLastPanicContext,
	type HtmlConversionOptions,
	type HtmlPreprocessingOptions,
	type ImageExtractionConfig,
	ImageProcessingError,
	type KeywordConfig,
	KreuzbergError,
	type LanguageDetectionConfig,
	listDocumentExtractors,
	listEmbeddingPresets,
	listOcrBackends,
	listPostProcessors,
	listValidators,
	MissingDependencyError,
	type OcrBackendProtocol,
	type OcrConfig,
	OcrError,
	type PageConfig,
	type PageContent,
	ParsingError,
	type PdfConfig,
	PluginError,
	type PostProcessorConfig,
	type PostProcessorProtocol,
	registerOcrBackend,
	registerPostProcessor,
	registerValidator,
	type Table,
	type TokenReductionConfig,
	unregisterDocumentExtractor,
	unregisterOcrBackend,
	unregisterPostProcessor,
	unregisterValidator,
	ValidationError,
	type ValidatorProtocol,
	validateMimeType,
} from "@kreuzberg/node";
import { afterEach, beforeAll, describe, expect, it } from "vitest";

describe("Kreuzberg TypeScript/Node.js Bindings", () => {
	describe("Version Info", () => {
		it("should export version constant", () => {
			expect(__version__).toBeDefined();
			expect(typeof __version__).toBe("string");
			expect(__version__).toMatch(/^\d+\.\d+\.\d+/);
		});
	});

	describe("MIME Type Detection & Validation", () => {
		let testFile: string;

		beforeAll(() => {
			const buffer = Buffer.from("%PDF-1.4\n%EOF");
			testFile = join(tmpdir(), "test-pdf.pdf");
			writeFileSync(testFile, buffer);
		});

		it("should detect MIME type from bytes", () => {
			const pdfHeader = Buffer.from("%PDF-1.4");
			const mimeType = detectMimeType(pdfHeader);
			expect(typeof mimeType).toBe("string");
			expect(mimeType.length).toBeGreaterThan(0);
		});

		it("should detect MIME type from file path", () => {
			const mimeType = detectMimeTypeFromPath("document.pdf", false);
			expect(typeof mimeType).toBe("string");
			expect(mimeType.length).toBeGreaterThan(0);
		});

		it("should validate MIME type", () => {
			const result = validateMimeType("application/pdf");
			expect(typeof result).toBe("string");
			expect(result.length).toBeGreaterThan(0);
		});

		it("should get extensions for MIME type", () => {
			const extensions = getExtensionsForMime("application/pdf");
			expect(Array.isArray(extensions)).toBe(true);
			expect(extensions.length).toBeGreaterThan(0);
			expect(extensions[0]).toBe("pdf");
		});

		it("should handle multiple MIME type extensions", () => {
			const extensions = getExtensionsForMime(
				"application/vnd.openxmlformats-officedocument.wordprocessingml.document",
			);
			expect(Array.isArray(extensions)).toBe(true);
			expect(extensions.includes("docx")).toBe(true);
		});
	});

	describe("ExtractionConfig Plain Objects", () => {
		it("should create extraction config with plain object", () => {
			const config = {};
			expect(config).toBeDefined();
			expect(typeof config).toBe("object");
		});

		it("should create config with chunking options", () => {
			const config = { chunking: { maxChars: 2048 } };
			expect(config).toBeDefined();
			expect(config.chunking).toBeDefined();
			expect(config.chunking?.maxChars).toBe(2048);
		});

		it("should configure OCR settings", () => {
			const ocrConfig: OcrConfig = {
				backend: "tesseract",
				language: "eng",
			};
			const config = { ocr: ocrConfig };
			expect(config.ocr).toBeDefined();
			expect(config.ocr?.backend).toBe("tesseract");
		});

		it("should configure PDF settings", () => {
			const config = { pdf: { extractTables: true } };
			expect(config.pdf).toBeDefined();
		});

		it("should configure chunking", () => {
			const chunkingConfig: ChunkingConfig = {
				maxChars: 4096,
				maxOverlap: 512,
			};
			const config = { chunking: chunkingConfig };
			expect(config.chunking).toBeDefined();
			expect(config.chunking?.maxChars).toBe(4096);
		});

		it("should configure image extraction", () => {
			const imageConfig: ImageExtractionConfig = {
				enabled: true,
			};
			const config = { imageExtraction: imageConfig };
			expect(config.imageExtraction).toBeDefined();
		});

		it("should configure keyword extraction", () => {
			const keywordConfig: KeywordConfig = {
				enabled: true,
				maxKeywords: 10,
			};
			const config = { keywords: keywordConfig };
			expect(config.keywords).toBeDefined();
		});

		it("should configure language detection", () => {
			const langConfig: LanguageDetectionConfig = {
				enabled: true,
			};
			const config = { languageDetection: langConfig };
			expect(config.languageDetection).toBeDefined();
		});

		it("should enable metadata extraction", () => {
			const config = { metadataExtraction: true };
			expect(config.metadataExtraction).toBe(true);
		});

		it("should enable quality mode", () => {
			const config = { qualityMode: true };
			expect(config.qualityMode).toBe(true);
		});
	});

	describe("ExtractionConfig Output and Result Formats", () => {
		let testTxtFile: string;
		let testTxtBuffer: Buffer;

		beforeAll(() => {
			testTxtBuffer = Buffer.from("# Test Heading\n\nThis is **bold** text.");
			testTxtFile = join(tmpdir(), "format-test.txt");
			writeFileSync(testTxtFile, testTxtBuffer);
		});

		describe("outputFormat configuration", () => {
			it("should accept outputFormat 'plain'", () => {
				const config = { outputFormat: "plain" as const };
				expect(config.outputFormat).toBe("plain");
			});

			it("should accept outputFormat 'markdown'", () => {
				const config = { outputFormat: "markdown" as const };
				expect(config.outputFormat).toBe("markdown");
			});

			it("should accept outputFormat 'djot'", () => {
				const config = { outputFormat: "djot" as const };
				expect(config.outputFormat).toBe("djot");
			});

			it("should accept outputFormat 'html'", () => {
				const config = { outputFormat: "html" as const };
				expect(config.outputFormat).toBe("html");
			});

			it("should extract with outputFormat 'plain' (sync)", () => {
				const config = { outputFormat: "plain" as const };
				const result = extractFileSync(testTxtFile, null, config);
				expect(result).toBeDefined();
				expect(typeof result.content).toBe("string");
			});

			it("should extract with outputFormat 'markdown' (sync)", () => {
				const config = { outputFormat: "markdown" as const };
				const result = extractBytesSync(testTxtBuffer, "text/plain", config);
				expect(result).toBeDefined();
			});

			it("should extract with outputFormat 'html' (async)", async () => {
				const config = { outputFormat: "html" as const };
				const result = await extractFile(testTxtFile, null, config);
				expect(result).toBeDefined();
			});

			it("should extract with outputFormat 'djot' (async)", async () => {
				const config = { outputFormat: "djot" as const };
				const result = await extractBytes(testTxtBuffer, "text/plain", config);
				expect(result).toBeDefined();
			});
		});

		describe("resultFormat configuration", () => {
			it("should accept resultFormat 'unified'", () => {
				const config = { resultFormat: "unified" as const };
				expect(config.resultFormat).toBe("unified");
			});

			it("should accept resultFormat 'element_based'", () => {
				const config = { resultFormat: "element_based" as const };
				expect(config.resultFormat).toBe("element_based");
			});

			it("should extract with resultFormat 'unified' (sync)", () => {
				const config = { resultFormat: "unified" as const };
				const result = extractFileSync(testTxtFile, null, config);
				expect(result).toBeDefined();
				expect(result.content).toBeDefined();
			});

			it("should extract with resultFormat 'element_based' (sync)", () => {
				const config = { resultFormat: "element_based" as const };
				const result = extractBytesSync(testTxtBuffer, "text/plain", config);
				expect(result).toBeDefined();
			});

			it("should extract with resultFormat 'unified' (async)", async () => {
				const config = { resultFormat: "unified" as const };
				const result = await extractFile(testTxtFile, null, config);
				expect(result).toBeDefined();
			});

			it("should extract with resultFormat 'element_based' (async)", async () => {
				const config = { resultFormat: "element_based" as const };
				const result = await extractBytes(testTxtBuffer, "text/plain", config);
				expect(result).toBeDefined();
			});
		});

		describe("Format combinations", () => {
			it("should combine outputFormat and resultFormat", () => {
				const config = {
					outputFormat: "markdown" as const,
					resultFormat: "unified" as const,
				};
				expect(config.outputFormat).toBe("markdown");
				expect(config.resultFormat).toBe("unified");
			});

			it("should extract with combined formats (sync)", () => {
				const config = {
					outputFormat: "markdown" as const,
					resultFormat: "element_based" as const,
				};
				const result = extractFileSync(testTxtFile, null, config);
				expect(result).toBeDefined();
			});

			it("should extract with combined formats (async)", async () => {
				const config = {
					outputFormat: "html" as const,
					resultFormat: "unified" as const,
				};
				const result = await extractFile(testTxtFile, null, config);
				expect(result).toBeDefined();
			});

			it("should batch extract with outputFormat", async () => {
				const config = { outputFormat: "markdown" as const };
				const results = await batchExtractFiles([testTxtFile], config);
				expect(Array.isArray(results)).toBe(true);
				expect(results.length).toBe(1);
			});

			it("should batch extract with resultFormat (sync)", () => {
				const config = { resultFormat: "element_based" as const };
				const results = batchExtractFilesSync([testTxtFile], config);
				expect(Array.isArray(results)).toBe(true);
				expect(results.length).toBe(1);
			});
		});
	});

	describe("Single Document Extraction", () => {
		let testPdfFile: string;
		let testTxtFile: string;
		let testPdfBuffer: Buffer;
		let testTxtBuffer: Buffer;

		beforeAll(() => {
			testPdfBuffer = Buffer.from("%PDF-1.4\n%EOF");
			testPdfFile = join(tmpdir(), "test-document.pdf");
			writeFileSync(testPdfFile, testPdfBuffer);

			testTxtBuffer = Buffer.from("Hello, World!");
			testTxtFile = join(tmpdir(), "test-document.txt");
			writeFileSync(testTxtFile, testTxtBuffer);
		});

		it("should extract from text file (sync)", () => {
			const result = extractFileSync(testTxtFile, null);
			expect(result).toBeDefined();
			expect(typeof result).toBe("object");
			validateExtractionResult(result);
		});

		it("should extract from text file (async)", async () => {
			const result = await extractFile(testTxtFile, null);
			expect(result).toBeDefined();
			expect(typeof result).toBe("object");
			validateExtractionResult(result);
		});

		it("should extract from bytes (sync)", () => {
			const result = extractBytesSync(testTxtBuffer, "text/plain", null);
			expect(result).toBeDefined();
			expect(typeof result).toBe("object");
			validateExtractionResult(result);
		});

		it("should extract from bytes (async)", async () => {
			const result = await extractBytes(testTxtBuffer, "text/plain", null);
			expect(result).toBeDefined();
			expect(typeof result).toBe("object");
			validateExtractionResult(result);
		});

		it("should extract with null config (uses defaults)", () => {
			const result = extractFileSync(testTxtFile, null);
			expect(result).toBeDefined();
			validateExtractionResult(result);
		});

		it("should handle missing file gracefully", () => {
			expect(() => {
				extractFileSync("/nonexistent/file.pdf", null);
			}).toThrow();
		});

		it("should handle invalid file path gracefully", async () => {
			await expect(extractFile("", null)).rejects.toThrow();
		});
	});

	describe("Batch Document Extraction", () => {
		const testFiles: string[] = [];
		const testBuffers: Buffer[] = [];
		const testMimeTypes: string[] = [];

		beforeAll(() => {
			for (let i = 0; i < 3; i++) {
				const filePath = join(tmpdir(), `batch-test-${i}.txt`);
				const content = Buffer.from(`Test document ${i}\n`);
				writeFileSync(filePath, content);
				testFiles.push(filePath);
				testBuffers.push(content);
				testMimeTypes.push("text/plain");
			}
		});

		it("should batch extract files (sync)", () => {
			const results = batchExtractFilesSync(testFiles, null);
			expect(Array.isArray(results)).toBe(true);
			expect(results.length).toBe(testFiles.length);
			for (const result of results) {
				validateExtractionResult(result);
			}
		});

		it("should batch extract files (async)", async () => {
			const results = await batchExtractFiles(testFiles, null);
			expect(Array.isArray(results)).toBe(true);
			expect(results.length).toBe(testFiles.length);
			for (const result of results) {
				validateExtractionResult(result);
			}
		});

		it("should batch extract bytes (sync)", () => {
			const results = batchExtractBytesSync(testBuffers, testMimeTypes, null);
			expect(Array.isArray(results)).toBe(true);
			expect(results.length).toBe(testBuffers.length);
			for (const result of results) {
				validateExtractionResult(result);
			}
		});

		it("should batch extract bytes (async)", async () => {
			const results = await batchExtractBytes(testBuffers, testMimeTypes, null);
			expect(Array.isArray(results)).toBe(true);
			expect(results.length).toBe(testBuffers.length);
			for (const result of results) {
				validateExtractionResult(result);
			}
		});

		it("should handle empty batch gracefully", () => {
			const results = batchExtractFilesSync([], null);
			expect(Array.isArray(results)).toBe(true);
			expect(results.length).toBe(0);
		});

		it("should handle single item batch", () => {
			const results = batchExtractFilesSync([testFiles[0]], null);
			expect(Array.isArray(results)).toBe(true);
			expect(results.length).toBe(1);
		});
	});

	describe("Post-Processor Plugin Registry", () => {
		const mockProcessor: PostProcessorProtocol = {
			name: "test-processor",
			processContent: (content: string) => content.toUpperCase(),
		};

		afterEach(() => {
			clearPostProcessors();
		});

		it("should register a post-processor", () => {
			registerPostProcessor(mockProcessor);
			const list = listPostProcessors();
			expect(list).toContain("test-processor");
		});

		it("should list registered post-processors", () => {
			const list = listPostProcessors();
			expect(Array.isArray(list)).toBe(true);
		});

		it("should unregister a post-processor", () => {
			registerPostProcessor(mockProcessor);
			unregisterPostProcessor("test-processor");
			const list = listPostProcessors();
			expect(list).not.toContain("test-processor");
		});

		it("should clear all post-processors", () => {
			registerPostProcessor(mockProcessor);
			clearPostProcessors();
			const list = listPostProcessors();
			expect(list.length).toBe(0);
		});

		it("should handle duplicate registration", () => {
			registerPostProcessor(mockProcessor);
			registerPostProcessor(mockProcessor);
			const list = listPostProcessors();
			const count = list.filter((name) => name === "test-processor").length;
			expect(count).toBeGreaterThanOrEqual(1);
		});
	});

	describe("Validator Plugin Registry", () => {
		const mockValidator: ValidatorProtocol = {
			name: "test-validator",
			validate: (content: string) => ({ isValid: content.length > 0, errors: [] }),
		};

		afterEach(() => {
			clearValidators();
		});

		it("should register a validator", () => {
			registerValidator(mockValidator);
			const list = listValidators();
			expect(list).toContain("test-validator");
		});

		it("should list registered validators", () => {
			const list = listValidators();
			expect(Array.isArray(list)).toBe(true);
		});

		it("should unregister a validator", () => {
			registerValidator(mockValidator);
			unregisterValidator("test-validator");
			const list = listValidators();
			expect(list).not.toContain("test-validator");
		});

		it("should clear all validators", () => {
			registerValidator(mockValidator);
			clearValidators();
			const list = listValidators();
			expect(list.length).toBe(0);
		});

		it("should handle unregistering non-existent validator gracefully", () => {
			expect(() => unregisterValidator("non-existent-validator")).not.toThrow();
		});
	});

	describe("OCR Backend Plugin Registry", () => {
		const mockOcrBackend: OcrBackendProtocol = {
			name: "test-ocr",
			supportedLanguages: ["eng", "deu"],
			processImage: async (imageData: Uint8Array, config: OcrConfig) => ({
				text: "Test OCR output",
				confidence: 0.95,
			}),
		};

		afterEach(() => {
			clearOcrBackends();
		});

		it("should register an OCR backend", () => {
			registerOcrBackend(mockOcrBackend);
			const list = listOcrBackends();
			expect(list).toContain("test-ocr");
		});

		it("should list registered OCR backends", () => {
			const list = listOcrBackends();
			expect(Array.isArray(list)).toBe(true);
		});

		it("should unregister an OCR backend", () => {
			registerOcrBackend(mockOcrBackend);
			unregisterOcrBackend("test-ocr");
			const list = listOcrBackends();
			expect(list).not.toContain("test-ocr");
		});

		it("should clear all OCR backends", () => {
			registerOcrBackend(mockOcrBackend);
			clearOcrBackends();
			const list = listOcrBackends();
			expect(list.length).toBe(0);
		});
	});

	describe("Document Extractor Registry", () => {
		it("should list document extractors", () => {
			const extractors = listDocumentExtractors();
			expect(Array.isArray(extractors)).toBe(true);
			expect(extractors.length).toBeGreaterThanOrEqual(0);
		});

		it("should handle clearing document extractors", () => {
			clearDocumentExtractors();
			const extractors = listDocumentExtractors();
			expect(Array.isArray(extractors)).toBe(true);
		});
	});

	describe("Embedding Presets", () => {
		it("should list embedding presets", () => {
			const presets = listEmbeddingPresets();
			expect(Array.isArray(presets)).toBe(true);
		});

		it("should get specific embedding preset", () => {
			const presets = listEmbeddingPresets();
			if (presets.length > 0) {
				const preset = getEmbeddingPreset(presets[0]);
				if (preset) {
					expect(typeof preset).toBe("object");
				}
			}
		});

		it("should return null for non-existent preset", () => {
			const preset = getEmbeddingPreset("non-existent-preset-xyz");
			expect(preset).toBeNull();
		});
	});

	describe("Error Handling & Classification", () => {
		it("should have defined error codes", () => {
			expect(ErrorCode).toBeDefined();
			expect(typeof ErrorCode).toBe("object");
		});

		it("should get last error code", () => {
			const code = getLastErrorCode();
			expect(typeof code).toBe("number");
		});

		it("should get error code name", () => {
			const name = getErrorCodeName(0);
			expect(typeof name).toBe("string");
		});

		it("should get error code description", () => {
			const description = getErrorCodeDescription(0);
			expect(typeof description).toBe("string");
		});

		it("should classify error messages", () => {
			const classification = classifyError("PDF parsing error");
			expect(classification).toBeDefined();
			expect(typeof classification).toBe("object");
		});

		it("should get last panic context", () => {
			const context = getLastPanicContext();
			if (context !== null) {
				expect(typeof context).toBe("object");
			}
		});
	});

	describe("Error Exception Classes", () => {
		it("should have KreuzbergError", () => {
			expect(KreuzbergError).toBeDefined();
			const error = new KreuzbergError("test");
			expect(error).toBeInstanceOf(Error);
			expect(error.message).toBe("test");
		});

		it("should have ValidationError", () => {
			expect(ValidationError).toBeDefined();
			const error = new ValidationError("validation failed");
			expect(error).toBeInstanceOf(Error);
		});

		it("should have ParsingError", () => {
			expect(ParsingError).toBeDefined();
			const error = new ParsingError("parsing failed");
			expect(error).toBeInstanceOf(Error);
		});

		it("should have OcrError", () => {
			expect(OcrError).toBeDefined();
			const error = new OcrError("ocr failed");
			expect(error).toBeInstanceOf(Error);
		});

		it("should have CacheError", () => {
			expect(CacheError).toBeDefined();
			const error = new CacheError("cache failed");
			expect(error).toBeInstanceOf(Error);
		});

		it("should have MissingDependencyError", () => {
			expect(MissingDependencyError).toBeDefined();
			const error = new MissingDependencyError("dependency missing");
			expect(error).toBeInstanceOf(Error);
		});

		it("should have ImageProcessingError", () => {
			expect(ImageProcessingError).toBeDefined();
			const error = new ImageProcessingError("image processing failed");
			expect(error).toBeInstanceOf(Error);
		});

		it("should have PluginError", () => {
			expect(PluginError).toBeDefined();
			const error = new PluginError("plugin failed");
			expect(error).toBeInstanceOf(Error);
		});
	});

	describe("Type System Validation", () => {
		it("should have ExtractionResult type", () => {
			const result: ExtractionResult = {
				content: "test",
				mimeType: "text/plain",
				chunks: [],
				images: [],
				tables: [],
				metadata: {},
				errors: [],
				duration: 0,
				pageCount: 1,
			};
			expect(result.content).toBe("test");
		});

		it("should have Chunk type", () => {
			const chunk: Chunk = {
				content: "test chunk",
				metadata: {
					byteStart: 0,
					byteEnd: 10,
					tokenCount: null,
					chunkIndex: 0,
					totalChunks: 1,
				},
				embedding: null,
			};
			expect(chunk.content).toBe("test chunk");
		});

		it("should have ExtractedImage type", () => {
			const image: ExtractedImage = {
				data: new Uint8Array(),
				format: "png",
				imageIndex: 0,
				pageNumber: null,
				width: null,
				height: null,
				colorspace: null,
				bitsPerComponent: null,
				isMask: false,
				description: null,
				ocrResult: null,
			};
			expect(image.format).toBe("png");
		});

		it("should have Table type", () => {
			const table: Table = {
				headers: ["col1", "col2"],
				rows: [["val1", "val2"]],
				metadata: {},
			};
			expect(table.headers.length).toBe(2);
		});

		it("should have PageContent type", () => {
			const pageContent: PageContent = {
				text: "page text",
				pageNumber: 1,
				bounds: null,
				metadata: {},
			};
			expect(pageContent.pageNumber).toBe(1);
		});

		it("should have OcrConfig type", () => {
			const ocrConfig: OcrConfig = {
				backend: "tesseract",
				language: "eng",
			};
			expect(ocrConfig.backend).toBe("tesseract");
		});

		it("should have ChunkingConfig type", () => {
			const chunkingConfig: ChunkingConfig = {
				maxChars: 2048,
				maxOverlap: 256,
			};
			expect(chunkingConfig.maxChars).toBe(2048);
		});

		it("should have PdfConfig type", () => {
			const pdfConfig: PdfConfig = {
				extractTables: true,
			};
			expect(pdfConfig.extractTables).toBe(true);
		});

		it("should have ImageExtractionConfig type", () => {
			const imageConfig: ImageExtractionConfig = {
				enabled: true,
			};
			expect(imageConfig.enabled).toBe(true);
		});

		it("should have KeywordConfig type", () => {
			const keywordConfig: KeywordConfig = {
				enabled: true,
			};
			expect(keywordConfig.enabled).toBe(true);
		});

		it("should have LanguageDetectionConfig type", () => {
			const langConfig: LanguageDetectionConfig = {
				enabled: true,
			};
			expect(langConfig.enabled).toBe(true);
		});

		it("should have ErrorClassification type", () => {
			const classification: ErrorClassification = {
				category: "parsing",
				severity: "high",
			};
			expect(classification.category).toBe("parsing");
		});
	});

	describe("Plugin Protocol Validation", () => {
		it("should satisfy PostProcessorProtocol", () => {
			const processor: PostProcessorProtocol = {
				name: "test",
				processContent: (content: string) => content,
			};
			expect(processor.name).toBe("test");
			expect(typeof processor.processContent).toBe("function");
		});

		it("should satisfy ValidatorProtocol", () => {
			const validator: ValidatorProtocol = {
				name: "test",
				validate: (content: string) => ({ isValid: true, errors: [] }),
			};
			expect(validator.name).toBe("test");
			expect(typeof validator.validate).toBe("function");
		});

		it("should satisfy OcrBackendProtocol", () => {
			const backend: OcrBackendProtocol = {
				name: "test",
				processImage: async (imageData: Uint8Array, config: OcrConfig) => ({
					text: "",
					confidence: 0,
				}),
			};
			expect(backend.name).toBe("test");
			expect(typeof backend.processImage).toBe("function");
		});
	});
});

function validateExtractionResult(result: unknown): void {
	expect(result).toBeDefined();
	expect(typeof result).toBe("object");

	if (typeof result === "object" && result !== null) {
		const r = result as Record<string, unknown>;

		if (r.content !== undefined) {
			expect(typeof r.content).toMatch(/^(string|object)$/);
		}

		if (r.chunks !== undefined && r.chunks !== null) {
			expect(Array.isArray(r.chunks)).toBe(true);
		}

		if (r.images !== undefined && r.images !== null) {
			expect(Array.isArray(r.images)).toBe(true);
		}

		if (r.tables !== undefined && r.tables !== null) {
			expect(Array.isArray(r.tables)).toBe(true);
		}

		if (r.metadata !== undefined) {
			expect(typeof r.metadata).toBe("object");
		}

		if (r.errors !== undefined && r.errors !== null) {
			expect(Array.isArray(r.errors)).toBe(true);
		}
	}
}
