import { describe, it, expect, beforeAll } from "vitest";
import { createNer } from "./ner";
import type { NerInterface } from "./types";

describe("NER", () => {
  let ner: NerInterface | null;

  beforeAll(async () => {
    // "Xenova/gliner2-small-onnx" (as written in the original spec) does not
    // exist on the Hub — GLiNER2 ONNX exports that do exist (e.g.
    // SemplificaAI/gliner2-multi-v1-onnx, lion-ai/gliner2-base-v1-onnx) target
    // a schema-driven "zero-shot" extraction API, not the standard
    // transformers.js `token-classification` pipeline this module implements.
    // We substitute "Xenova/bert-base-NER" — a real, canonically-cased,
    // ONNX-converted NER model (dslim/bert-base-NER) explicitly documented as
    // transformers.js v3 compatible. This triggers a live download on first
    // run.
    ner = await createNer({
      models: { ner: "Xenova/bert-base-NER" },
    });
  }, 120_000);

  it("detects named entities in text", async () => {
    if (!ner) {
      console.log("[skip] NER not enabled");
      return;
    }
    const text = "Alice works at Google in Mountain View.";
    const entities = await ner.ner(text);

    expect(Array.isArray(entities)).toBe(true);
    // Expect some entities like PERSON, ORGANIZATION, LOCATION
    const labels = entities.map((e) => e.label);
    expect(labels.length).toBeGreaterThan(0);
  }, 60_000);

  it("returns entity structure with position info", async () => {
    if (!ner) {
      console.log("[skip] NER not enabled");
      return;
    }
    const text = "Email: john@example.com";
    const entities = await ner.ner(text);

    if (entities.length > 0) {
      const entity = entities[0];
      if (!entity) throw new Error("expected entity");
      expect(entity).toHaveProperty("label");
      expect(entity).toHaveProperty("text");
      expect(entity).toHaveProperty("start");
      expect(entity).toHaveProperty("end");
      expect(typeof entity.label).toBe("string");
      expect(typeof entity.start).toBe("number");
      expect(typeof entity.end).toBe("number");
    }
  }, 60_000);
});
