import { describe, it, expect } from "vitest";
import { detectPii, mergeNerEntities, type NerEntity } from "../src/redaction/detect.js";

describe("mergeNerEntities", () => {
  it("returns sorted regex findings unchanged when entities list is empty", () => {
    const text = "Call me at 555-123-4567 or email me.";
    const regex = detectPii(text);
    const merged = mergeNerEntities(regex, [], text);
    expect(merged).toEqual(regex);
  });

  it("appends NER entity as new finding when no span overlap with regex", () => {
    const text = "Contact Alice Smith for details.";
    const regex = detectPii(text);
    const entities: NerEntity[] = [{ text: "Alice Smith", label: "PERSON", score: 0.92, start: 8, end: 19 }];
    const merged = mergeNerEntities(regex, entities, text);
    const names = merged.filter((f) => f.category === "NAME");
    expect(names).toHaveLength(1);
    expect(names[0]?.original).toBe("Alice Smith");
    expect(names[0]?.confidence).toBe(0.92);
  });

  it("maps NER label PERSON to category NAME", () => {
    const text = "Bob Jones signed.";
    const entities: NerEntity[] = [{ text: "Bob Jones", label: "PERSON", score: 0.88, start: 0, end: 9 }];
    const merged = mergeNerEntities([], entities, text);
    expect(merged[0]?.category).toBe("NAME");
  });

  it("maps ORG and GPE labels to ORG and LOCATION categories", () => {
    const text = "Acme Corp in Berlin.";
    const entities: NerEntity[] = [
      { text: "Acme Corp", label: "ORG", score: 0.9, start: 0, end: 9 },
      { text: "Berlin", label: "GPE", score: 0.87, start: 13, end: 19 },
    ];
    const merged = mergeNerEntities([], entities, text);
    expect(merged.find((f) => f.category === "ORG")?.original).toBe("Acme Corp");
    expect(merged.find((f) => f.category === "LOCATION")?.original).toBe("Berlin");
  });

  it("deduplicates when NER span overlaps regex finding, keeping higher-confidence result", () => {
    const text = "Sent from 555-123-4567.";
    const regex = detectPii(text);
    const phoneRegex = regex.find((f) => f.category === "PHONE");
    expect(phoneRegex).toBeDefined();

    const highConfidenceEntity: NerEntity[] = [
      { text: "555-123-4567", label: "PHONE", score: 0.99, start: phoneRegex!.start, end: phoneRegex!.end },
    ];
    const merged = mergeNerEntities(regex, highConfidenceEntity, text);
    const phones = merged.filter((f) => f.category === "PHONE");
    expect(phones).toHaveLength(1);
    expect(phones[0]?.confidence).toBe(0.99);
  });

  it("keeps lower-confidence regex finding when NER confidence is not higher on overlap", () => {
    const text = "My SSN is 123-45-6789.";
    const regex = detectPii(text);
    const ssn = regex.find((f) => f.category === "SSN");
    expect(ssn).toBeDefined();
    const originalConfidence = ssn!.confidence;

    const lowConfidenceEntity: NerEntity[] = [
      { text: "123-45-6789", label: "SSN", score: 0.5, start: ssn!.start, end: ssn!.end },
    ];
    const merged = mergeNerEntities(regex, lowConfidenceEntity, text);
    const ssns = merged.filter((f) => f.category === "SSN");
    expect(ssns).toHaveLength(1);
    expect(ssns[0]?.confidence).toBe(originalConfidence);
  });

  it("assigns sequential token format [CATEGORY_N] to NER-added findings", () => {
    const text = "Hello from Alice and Bob.";
    const entities: NerEntity[] = [
      { text: "Alice", label: "PERSON", score: 0.9, start: 11, end: 16 },
      { text: "Bob", label: "PERSON", score: 0.88, start: 21, end: 24 },
    ];
    const merged = mergeNerEntities([], entities, text);
    const tokens = merged.map((f) => f.token);
    expect(tokens).toContain("[NAME_1]");
    expect(tokens).toContain("[NAME_2]");
  });
});
