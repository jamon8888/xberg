import * as fs from "fs";
import * as path from "path";
import { homedir } from "os";
import { env } from "@huggingface/transformers";
import { createEmbedder } from "./embedder";
import { createNer } from "./ner";

declare global {
  interface Window {
    ort?: { env: { wasm: { wasmPaths: string } } };
  }
}

interface ModelInfo {
  name: string;
  repo: string;
  path: string;
  size: number;
}

/**
 * Pipeline handles that `warm()` can pre-download. Each maps to a factory in
 * this package whose `pipeline(...)` call performs the actual model fetch.
 */
type WarmHandle = "embedding" | "ner";

interface WarmOptions {
  /** Restrict warm-up to these model display names (see `MODELS`). */
  modelNames?: string[];
  /** Called once per pipeline handle before it is downloaded. */
  onProgress?: (phase: string) => void;
}

/** Maps legacy `MODELS` display names to the pipeline handles `warm()` knows. */
const MODEL_NAME_TO_HANDLE: Record<string, WarmHandle> = {
  "Embedder (minilm-l6-v2)": "embedding",
  "GLiNER2 NER": "ner",
};

const MODELS: ModelInfo[] = [
  {
    name: "Embedder (minilm-l6-v2)",
    repo: "Xenova/minilm-l6-v2",
    path: "embeddings/minilm-l6-v2.onnx",
    size: 90000000,
  },
  {
    name: "GLiNER2 NER",
    repo: "Xenova/gliner2-small-onnx",
    path: "ner/gliner2-small.onnx",
    size: 310000000,
  },
  {
    name: "PP-OCRv6 OCR",
    repo: "paddleocr/pp-ocrv6",
    path: "ocr/pp-ocrv6.onnx",
    size: 320000000,
  },
];

/**
 * Manages model cache in OPFS (browser, not yet implemented) or ~/.cache/xberg (Node).
 * Mirrors the MCP WarmupManager responsibilities.
 */
export class CacheManager {
  private cacheDir: string;

  constructor(cacheDir?: string) {
    this.cacheDir =
      cacheDir ??
      this.defaultCacheDir();
  }

  private defaultCacheDir(): string {
    if (typeof window === "undefined") {
      // Node.js
      const base =
        process.platform === "win32"
          ? process.env.LOCALAPPDATA ?? path.join(homedir(), "AppData", "Local")
          : path.join(homedir(), ".cache");
      return path.join(base, "xberg");
    }
    // Browser: OPFS virtual path (actual I/O handled by wa-sqlite)
    return "/opfs/xberg";
  }

  async status(): Promise<{
    cached: string[];
    size: number;
  }> {
    const cached: string[] = [];
    let totalSize = 0;

    for (const model of MODELS) {
      const modelPath = path.join(this.cacheDir, model.path);
      try {
        if (typeof window === "undefined" && fs.existsSync(modelPath)) {
          const stats = fs.statSync(modelPath);
          cached.push(model.name);
          totalSize += stats.size;
        } else if (typeof window !== "undefined") {
          // Browser: check OPFS (simplified; actual check would use storage API)
          // For now, assume not cached in CI
        }
      } catch (err) {
        // File not found is expected; other errors should be logged
        if (err instanceof Error && "code" in err && err.code !== "ENOENT") {
          console.warn(`[cache] unexpected error checking ${model.name}:`, err);
        }
      }
    }

    return { cached, size: totalSize };
  }

  /**
   * Pre-download and cache the model artifacts used by the SDK's embedder and
   * NER pipelines so cold-start never blocks on a network fetch.
   *
   * The download is performed by routing `@huggingface/transformers` to this
   * cache directory (`env.cacheDir`) and invoking the existing `createEmbedder`
   * / `createNer` factories, whose `pipeline(...)` calls are what actually
   * fetch and persist the model files. No model-download logic is reimplemented
   * here.
   *
   * Accepts either a list of model display names (legacy form, see `MODELS`)
   * or an options object carrying an `onProgress` callback. Returns the set of
   * pipeline handles that succeeded / failed.
   */
  async warm(modelNames?: string[]): Promise<{ success: string[]; failed: string[] }>;
  async warm(opts?: WarmOptions): Promise<{ success: string[]; failed: string[] }>;
  async warm(
    arg?: string[] | WarmOptions
  ): Promise<{ success: string[]; failed: string[] }> {
    const opts: WarmOptions = Array.isArray(arg)
      ? { modelNames: arg }
      : (arg ?? {});

    const selected = opts.modelNames
      ? opts.modelNames
          .map((name) => MODEL_NAME_TO_HANDLE[name])
          .filter((h): h is WarmHandle => h !== undefined)
      : (["embedding", "ner"] as WarmHandle[]);

    // De-duplicate while preserving order: embedding first, then ner.
    const handles = ["embedding", "ner"].filter(
      (h) => selected.includes(h as WarmHandle)
    ) as WarmHandle[];

    const success: string[] = [];
    const failed: string[] = [];

    // Route transformers.js downloads into this cache directory.
    env.cacheDir = this.cacheDir;

    for (const handle of handles) {
      try {
        opts.onProgress?.(handle);
        // eslint-disable-next-line no-await-in-loop -- download sequentially to bound concurrency
        if (handle === "embedding") {
          await createEmbedder({ nodeCachePath: this.cacheDir });
        } else {
          await createNer({ nodeCachePath: this.cacheDir });
        }
        success.push(handle);
      } catch (err) {
        console.error(`[cache] warm failed for ${handle}:`, err);
        failed.push(handle);
      }
    }

    return { success, failed };
  }

  /**
   * Set ONNX Runtime wasm binary paths to self-hosted location (no CDN).
   */
  setWasmPaths(wasmDir: string): void {
    try {
      if (typeof window !== "undefined" && "ort" in window && window.ort) {
        window.ort.env.wasm.wasmPaths = wasmDir;
        console.debug(`[cache] ORT wasm paths set to ${wasmDir}`);
      } else {
        console.debug(`[cache] window.ort not found; setWasmPaths is a no-op`);
      }
    } catch (err) {
      console.warn(`[cache] failed to set ORT wasm paths:`, err);
    }
  }
}
