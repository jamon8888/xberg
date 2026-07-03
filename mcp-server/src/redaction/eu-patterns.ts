import { isValidBelgianRegistre, isValidBsn, isValidPesel } from "./eu-checksums.js";
import type { PiiFinding } from "./detect.js";

interface RawMatch {
  category: string;
  original: string;
  start: number;
  end: number;
  confidence: number;
}

/** True if [start, end) overlaps any existing match's span. */
export function overlapsExisting(matches: RawMatch[], start: number, end: number): boolean {
  return matches.some((m) => start < m.end && m.start < end);
}

function findAllNonOverlapping(
  text: string,
  pattern: RegExp,
  category: string,
  confidence: number,
  existing: RawMatch[],
  validate?: (matchText: string) => boolean,
): RawMatch[] {
  const found: RawMatch[] = [];
  const flags = pattern.flags.includes("g") ? pattern.flags : `${pattern.flags}g`;
  const regex = new RegExp(pattern.source, flags);
  let match: RegExpExecArray | null;
  while ((match = regex.exec(text)) !== null) {
    const matchText = match[0];
    const start = match.index;
    const end = start + matchText.length;
    if (validate && !validate(matchText)) continue;
    if (overlapsExisting(existing, start, end) || overlapsExisting(found, start, end)) continue;
    found.push({ category, original: matchText, start, end, confidence });
  }
  return found;
}

/**
 * Scan for EU-specific structured PII: national IDs, tax identifiers, and
 * EU vehicle license plates. Does NOT include GDPR Art. 9 keyword patterns --
 * see `scanArt9Keywords` for those.
 *
 * Ordering matters: national IDs run before tax IDs (SIRET before SIREN, so
 * the 9-digit SIREN prefix inside a 14-digit SIRET doesn't get double-flagged),
 * matching the priority order in anno's `pii.rs::scan_eu_structured`.
 */
export function scanEuStructured(text: string): RawMatch[] {
  const results: RawMatch[] = [];

  // --- National IDs ---
  results.push(
    ...findAllNonOverlapping(
      text,
      /\b[12]\d{2}(?:0[1-9]|1[0-2])\d{2}\d{3}\d{3}\d{2}\b/g,
      "NATIONAL_ID_FR",
      0.97,
      results,
    ),
  );
  results.push(
    ...findAllNonOverlapping(text, /\b(?:[XYZ]\d{7}|\d{8})[A-Z]\b/g, "NATIONAL_ID_ES", 0.97, results),
  );
  results.push(
    ...findAllNonOverlapping(
      text,
      /\b[A-Z]{6}\d{2}[A-Z]\d{2}[A-Z]\d{3}[A-Z]\b/g,
      "NATIONAL_ID_IT",
      0.97,
      results,
    ),
  );
  results.push(
    ...findAllNonOverlapping(text, /\b\d{11}\b/g, "NATIONAL_ID_PL", 0.97, results, isValidPesel),
  );
  results.push(
    ...findAllNonOverlapping(text, /\b\d{9}\b/g, "NATIONAL_ID_NL", 0.97, results, isValidBsn),
  );
  results.push(
    ...findAllNonOverlapping(
      text,
      /\b\d{2}[0-1]\d[0-3]\d\d{5}\b/g,
      "NATIONAL_ID_BE",
      0.97,
      results,
      isValidBelgianRegistre,
    ),
  );

  // --- Tax identifiers ---
  results.push(
    ...findAllNonOverlapping(
      text,
      /\b\d{3}\s?\d{3}\s?\d{3}\s?\d{5}\b/g,
      "TAX_ID_SIRET",
      0.9,
      results,
      (m) => m.replace(/\D/g, "").length === 14,
    ),
  );
  results.push(
    ...findAllNonOverlapping(
      text,
      /\b\d{3}\s?\d{3}\s?\d{3}\b/g,
      "TAX_ID_SIREN",
      0.9,
      results,
      (m) => m.replace(/\D/g, "").length === 9,
    ),
  );
  results.push(
    ...findAllNonOverlapping(
      text,
      /\b(?:AT|BE|BG|CY|CZ|DE|DK|EE|EL|ES|FI|FR|GB|HR|HU|IE|IT|LT|LU|LV|MT|NL|PL|PT|RO|SE|SI|SK)\d{8,12}\b/g,
      "TAX_ID_VAT",
      0.9,
      results,
    ),
  );
  results.push(
    ...findAllNonOverlapping(
      text,
      /\b(?:DE|FR|IT|ES|PL|NL|BE|PT|CZ|HU|SE|AT|CH|RO|BG|DK|FI|GR|IE|SK|SI|HR|LT|LV|EE|LU|MT|CY)\s?-?\d[\w-]{2,6}\b/g,
      "LICENSE_PLATE_EU",
      0.75,
      results,
    ),
  );

  return results;
}
