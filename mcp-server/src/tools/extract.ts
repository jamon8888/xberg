import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import type { Chunk, ExtractionConfig, ExtractInput, ExtractInputKind, ChunkingConfig, KeywordConfig, KeywordAlgorithm } from "@xberg-io/xberg";
import {
  extract,
  extractBatch,
  listSupportedFormats,
} from "@xberg-io/xberg";

const ExtractInputSchema = z.object({
  uri: z.string().optional(),
  bytes: z.array(z.number().int().min(0).max(255)).optional(),
  mime_type: z.string().optional(),
  filename: z.string().optional(),
});

const ChunkingConfigSchema = z.object({
  max_size: z.number().int().min(64).max(16384).optional(),
  overlap: z.number().int().min(0).max(1024).optional(),
});

const KeywordConfigSchema = z.object({
  algorithm: z.enum(["yake", "rake"]).optional(),
  max_keywords: z.number().int().min(1).max(100).optional(),
});

const OcrConfigSchema = z.object({
  backend: z.enum(["tesseract", "paddleocr"]).optional(),
  languages: z.array(z.string()).optional(),
});

const ExtractionConfigSchema = z.object({
  force_ocr: z.boolean().optional(),
  disable_ocr: z.boolean().optional(),
  use_cache: z.boolean().optional(),
  chunking: ChunkingConfigSchema.optional(),
  keywords: KeywordConfigSchema.optional(),
  ocr: OcrConfigSchema.optional(),
});

function toNativeConfig(config: z.infer<typeof ExtractionConfigSchema> | undefined): ExtractionConfig | null {
  if (!config) return null;

  const chunkingConfig: ChunkingConfig | undefined = config.chunking
    ? {
        maxCharacters: config.chunking.max_size,
        overlap: config.chunking.overlap,
      }
    : undefined;

  const keywordAlgo = config.keywords?.algorithm as KeywordAlgorithm | undefined;
  const keywordConfig: KeywordConfig | undefined = config.keywords
    ? {
        algorithm: keywordAlgo,
        maxKeywords: config.keywords.max_keywords,
      }
    : undefined;

  return {
    forceOcr: config.force_ocr,
    disableOcr: config.disable_ocr,
    useCache: config.use_cache,
    chunking: chunkingConfig,
    keywords: keywordConfig,
    ocr: config.ocr
      ? { backend: config.ocr.backend, language: config.ocr.languages }
      : undefined,
  };
}

export function registerExtractTools(server: McpServer): void {
  server.tool(
    "extract_document",
    "Extract text, tables, and metadata from a document (91+ formats: PDF, DOCX, XLSX, images with OCR, HTML, email, code, and more). " +
    "Provide uri (file path or HTTPS URL) or bytes (number array). " +
    "Config: force_ocr, disable_ocr, use_cache, " +
    "chunking {max_size, overlap}, keywords {algorithm: yake|rake, max_keywords}, " +
    "ocr {backend: tesseract|paddleocr, languages: [eng, deu, ...]}.",
    {
      input: ExtractInputSchema.optional(),
      config: ExtractionConfigSchema.optional(),
    },
    async ({ input, config }) => {
      try {
        let extractInput: ExtractInput | undefined;
        if (input?.bytes) {
          extractInput = {
            kind: "bytes" as ExtractInputKind,
            bytes: new Uint8Array(input.bytes),
            mimeType: input.mime_type ?? "application/octet-stream",
            filename: input.filename ?? undefined,
          };
        } else if (input?.uri) {
          extractInput = {
            kind: "uri" as ExtractInputKind,
            uri: input.uri,
          };
        } else {
          return {
            content: [{ type: "text" as const, text: "Error: must provide either input.uri or input.bytes" }],
            isError: true,
          };
        }

        const result = await extract(extractInput, toNativeConfig(config));

        const structured = {
          results: (result.results ?? []).map((doc) => ({
            content: doc.content ?? "",
            mimeType: doc.mimeType,
            metadata: doc.metadata,
            tables: doc.tables ?? [],
            detectedLanguages: doc.detectedLanguages ?? [],
            pages: doc.pages?.length ?? 0,
            chunks: (doc.chunks ?? []).map((c: Chunk) => ({
              content: c.content,
              index: c.metadata.chunkIndex,
            })),
            keywords: (doc.extractedKeywords ?? []).map((k: { text: string; score?: number }) => ({
              text: k.text,
              score: k.score ?? null,
            })),
            confidence: doc.metadata?.additional?.quality_score ?? null,
          })),
        };

        return {
          content: [{ type: "text" as const, text: JSON.stringify(structured, null, 2) }],
        };
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        return {
          content: [{ type: "text" as const, text: `Extraction failed: ${msg}` }],
          isError: true,
        };
      }
    }
  );

  server.tool(
    "extract_batch",
    "Extract from multiple documents in parallel.",
    {
      inputs: z.array(ExtractInputSchema),
      config: ExtractionConfigSchema.optional(),
    },
    async ({ inputs, config }) => {
      try {
        const nativeInputs: ExtractInput[] = inputs.map((inp) => {
          if (inp.bytes) {
            return {
              kind: "bytes" as ExtractInputKind,
              bytes: new Uint8Array(inp.bytes),
              mimeType: inp.mime_type ?? "application/octet-stream",
              filename: inp.filename ?? undefined,
            };
          }
          return {
            kind: "uri" as ExtractInputKind,
            uri: inp.uri ?? "",
          };
        });

        const result = await extractBatch(nativeInputs, toNativeConfig(config));

        return {
          content: [{ type: "text" as const, text: JSON.stringify(result, null, 2) }],
        };
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        return {
          content: [{ type: "text" as const, text: `Batch extraction failed: ${msg}` }],
          isError: true,
        };
      }
    }
  );

  server.tool(
    "list_formats",
    "List all document formats xberg can extract from.",
    {},
    async () => {
      const formats = listSupportedFormats();
      return {
        content: [{ type: "text" as const, text: JSON.stringify(formats, null, 2) }],
      };
    }
  );
}
