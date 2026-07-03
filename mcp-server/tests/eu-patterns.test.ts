import { describe, it, expect } from "vitest";
import { scanEuStructured } from "../src/redaction/eu-patterns.js";

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
