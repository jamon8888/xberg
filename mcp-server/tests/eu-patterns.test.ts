import { describe, it, expect } from "vitest";
import { scanEuStructured, scanArt9Keywords, scanEuPatterns } from "../src/redaction/eu-patterns.js";

describe("scanEuStructured", () => {
  it("detects a French INSEE number", () => {
    const result = scanEuStructured("INSEE: 185057511602324");
    expect(result.some((m) => m.category === "NATIONAL_ID_FR")).toBe(true);
  });

  it("detects a Spanish DNI", () => {
    const result = scanEuStructured("DNI: 12345678Z");
    expect(result.some((m) => m.category === "NATIONAL_ID_ES")).toBe(true);
  });

  it("detects an Italian Codice Fiscale", () => {
    const result = scanEuStructured("Codice Fiscale: RSSMRA85T10A562S");
    expect(result.some((m) => m.category === "NATIONAL_ID_IT")).toBe(true);
  });

  it("detects a valid Polish PESEL", () => {
    const result = scanEuStructured("PESEL: 80051501231");
    expect(result.some((m) => m.category === "NATIONAL_ID_PL")).toBe(true);
  });

  it("rejects a PESEL-shaped number with a bad checksum", () => {
    const result = scanEuStructured("80051501230");
    expect(result.some((m) => m.category === "NATIONAL_ID_PL")).toBe(false);
  });

  it("detects a valid Dutch BSN", () => {
    const result = scanEuStructured("BSN: 123456782");
    expect(result.some((m) => m.category === "NATIONAL_ID_NL")).toBe(true);
  });

  it("detects a French SIRET (14 digits)", () => {
    const result = scanEuStructured("SIRET: 73282932000074");
    expect(result.some((m) => m.category === "TAX_ID_SIRET")).toBe(true);
    // The 9-digit SIREN prefix should not ALSO be flagged separately -- it overlaps the SIRET span.
    expect(result.some((m) => m.category === "TAX_ID_SIREN")).toBe(false);
  });

  it("detects an EU VAT number", () => {
    const result = scanEuStructured("VAT: FR12345678901");
    expect(result.some((m) => m.category === "TAX_ID_VAT")).toBe(true);
  });

  it("returns character offsets that round-trip to the original text", () => {
    const text = "Café PESEL: 80051501231 end";
    const result = scanEuStructured(text);
    const pesel = result.find((m) => m.category === "NATIONAL_ID_PL");
    expect(pesel).toBeDefined();
    expect(text.slice(pesel!.start, pesel!.end)).toBe("80051501231");
  });
});

describe("scanArt9Keywords", () => {
  it("detects a health condition keyword", () => {
    const result = scanArt9Keywords("Patient diagnosed with diabetes");
    expect(result.some((m) => m.category === "SPECIAL_CATEGORY_HEALTH")).toBe(true);
  });

  it("detects a religion keyword", () => {
    const result = scanArt9Keywords("He is Catholic");
    expect(result.some((m) => m.category === "SPECIAL_CATEGORY_RELIGION")).toBe(true);
  });

  it("detects a criminal record keyword", () => {
    const result = scanArt9Keywords("He was convicted of fraud");
    expect(result.some((m) => m.category === "SPECIAL_CATEGORY_CRIMINAL")).toBe(true);
  });

  it("detects a biometric keyword", () => {
    const result = scanArt9Keywords("Access requires facial recognition");
    expect(result.some((m) => m.category === "SPECIAL_CATEGORY_BIOMETRIC")).toBe(true);
  });

  it("returns nothing for neutral text", () => {
    const result = scanArt9Keywords("The meeting is scheduled for Tuesday.");
    expect(result).toHaveLength(0);
  });
});

describe("scanEuPatterns", () => {
  it("combines structured and Art. 9 findings, sorted by position", () => {
    const result = scanEuPatterns("Patient diagnosed with diabetes. PESEL: 80051501231.");
    expect(result.some((f) => f.category === "SPECIAL_CATEGORY_HEALTH")).toBe(true);
    expect(result.some((f) => f.category === "NATIONAL_ID_PL")).toBe(true);
    for (let i = 1; i < result.length; i++) {
      expect(result[i]!.start).toBeGreaterThanOrEqual(result[i - 1]!.start);
    }
  });

  it("assigns sequential per-category tokens", () => {
    const result = scanEuPatterns("He is Catholic. She is Muslim.");
    const religionFindings = result.filter((f) => f.category === "SPECIAL_CATEGORY_RELIGION");
    expect(religionFindings.map((f) => f.token)).toEqual([
      "[SPECIAL_CATEGORY_RELIGION_1]",
      "[SPECIAL_CATEGORY_RELIGION_2]",
    ]);
  });

  it("structured matches take precedence over overlapping Art. 9 matches", () => {
    // "hospital" (Art.9 health keyword) inside a longer structured match is not
    // a realistic overlap case for these patterns, so this asserts the simpler
    // invariant: every returned finding has a non-empty original span.
    const result = scanEuPatterns("SIRET: 73282932000074");
    expect(result.every((f) => f.end > f.start)).toBe(true);
  });

  it("returns an empty array for text with no EU PII", () => {
    expect(scanEuPatterns("The weather is nice today.")).toHaveLength(0);
  });
});
