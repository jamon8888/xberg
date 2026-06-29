export class WarmupManager {
  constructor(private cacheDir: string) {}

  checkModels(): { name: string; cached: boolean; path: string }[] {
    return [
      { name: "BGE-M3", cached: false, path: "embeddings/BAAI--bge-m3/model.onnx" },
      { name: "bge-reranker-base", cached: false, path: "reranker/BAAI--bge-reranker-base/model.onnx" },
      { name: "GLiNER2-PII", cached: false, path: "ner/okasi--gliner2-privacy-filter-pii-multi-onnx/model.onnx" },
    ];
  }

  getMissingModels(): string[] {
    return this.checkModels()
      .filter((m) => !m.cached)
      .map((m) => m.name);
  }
}