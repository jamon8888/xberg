import { describe, it, expect } from "vitest";
import { isValidPesel, isValidBsn, isValidBelgianRegistre } from "../src/redaction/eu-checksums.js";

describe("isValidPesel", () => {
  it("accepts a valid PESEL", () => {
    // sum(d*w for weights [1,3,7,9,1,3,7,9,1,3]) = 89, check = (10-9)%10 = 1
    expect(isValidPesel("80051501231")).toBe(true);
  });

  it("rejects a PESEL with a wrong check digit", () => {
    expect(isValidPesel("80051501230")).toBe(false);
  });

  it("rejects the wrong length", () => {
    expect(isValidPesel("8005150123")).toBe(false);
    expect(isValidPesel("800515012345")).toBe(false);
  });

  it("rejects non-digit input", () => {
    expect(isValidPesel("8005150123X")).toBe(false);
  });
});

describe("isValidBsn", () => {
  it("accepts a valid BSN", () => {
    expect(isValidBsn("123456782")).toBe(true);
  });

  it("rejects a BSN with a wrong check digit", () => {
    expect(isValidBsn("123456780")).toBe(false);
  });

  it("rejects the wrong length", () => {
    expect(isValidBsn("12345678")).toBe(false);
  });
});

describe("isValidBelgianRegistre", () => {
  it("accepts a valid pre-2000 Registre National number", () => {
    // 800515012 % 97 = 8, check = 97 - 8 = 89
    expect(isValidBelgianRegistre("80051501289")).toBe(true);
  });

  it("rejects a pre-2000 number with the wrong check digits", () => {
    expect(isValidBelgianRegistre("80051501294")).toBe(false);
  });

  it("accepts a valid post-2000 Registre National number", () => {
    // Born 2001-05-15, sequence 012: n = 010515012
    // 2_000_000_000 % 97 = 68; 10_515_012 % 97 = 18; (68+18)%97 = 86; check = 97-86 = 11
    expect(isValidBelgianRegistre("01051501211")).toBe(true);
  });

  it("rejects the wrong length", () => {
    expect(isValidBelgianRegistre("8005150128")).toBe(false);
  });
});
