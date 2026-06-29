export interface PiiFinding {
  token: string;
  category: string;
  original: string;
  start: number;
  end: number;
  confidence: number;
}

const PATTERNS: Array<{ category: string; pattern: RegExp; confidence: number }> = [
  { category: "EMAIL", pattern: /\b[A-Za-z0-9._%+-]+@[A-Za-z0-9.-]+\.[A-Za-z]{2,}\b/g, confidence: 0.95 },
  { category: "PHONE", pattern: /\b(?:\+?\d{1,3}[-.\s]?)?\(?\d{3}\)?[-.\s]?\d{3}[-.\s]?\d{4}\b/g, confidence: 0.85 },
  { category: "SSN", pattern: /\b\d{3}-\d{2}-\d{4}\b/g, confidence: 0.9 },
  { category: "CREDIT_CARD", pattern: /\b(?:\d{4}[-\s]?){3}\d{4}\b/g, confidence: 0.9 },
  { category: "IP_ADDRESS", pattern: /\b(?:\d{1,3}\.){3}\d{1,3}\b/g, confidence: 0.8 },
  { category: "DATE_ISO", pattern: /\b\d{4}-\d{2}-\d{2}\b/g, confidence: 0.7 },
  { category: "DATE_MDY", pattern: /\b\d{1,2}\/\d{1,2}\/\d{2,4}\b/g, confidence: 0.7 },
  { category: "IBAN", pattern: /\b[A-Z]{2}\d{2}[A-Z0-9]{4,30}\b/g, confidence: 0.85 },
  { category: "SWIFT_BIC", pattern: /\b[A-Z]{6}[A-Z0-9]{2}([A-Z0-9]{3})?\b/g, confidence: 0.8 },
  { category: "POSTAL_CODE_US", pattern: /\b\d{5}(?:-\d{4})?\b/g, confidence: 0.75 },
  { category: "POSTAL_CODE_UK", pattern: /\b[A-Z]{1,2}\d[A-Z\d]?\s?\d[A-Z]{2}\b/g, confidence: 0.75 },
];

export function detectPii(text: string, filterCategories?: string[]): PiiFinding[] {
  const findings: PiiFinding[] = [];
  const counters: Record<string, number> = {};

  for (const { category, pattern, confidence } of PATTERNS) {
    if (filterCategories && !filterCategories.includes(category)) continue;

    const regex = new RegExp(pattern.source, pattern.flags);
    let match: RegExpExecArray | null;
    while ((match = regex.exec(text)) !== null) {
      counters[category] = (counters[category] ?? 0) + 1;
      findings.push({
        token: `[${category}_${counters[category]}]`,
        category,
        original: match[0],
        start: match.index,
        end: match.index + match[0].length,
        confidence,
      });
    }
  }

  return findings.sort((a, b) => a.start - b.start);
}

export interface NerEntity {
  text: string;
  label: string;
  score: number;
  start?: number;
  end?: number;
}

const NER_LABEL_TO_CATEGORY: Record<string, string> = {
  PERSON: "NAME",
  PER: "NAME",
  ORG: "ORG",
  GPE: "LOCATION",
  LOC: "LOCATION",
  LOCATION: "LOCATION",
  EMAIL: "EMAIL",
  PHONE: "PHONE",
};

function spansOverlap(a: PiiFinding, b: { start: number; end: number }): boolean {
  return a.start < b.end && b.start < a.end;
}

export function mergeNerEntities(regex: PiiFinding[], entities: NerEntity[], text: string): PiiFinding[] {
  const findings = [...regex];
  const counters: Record<string, number> = {};
  for (const f of findings) {
    counters[f.category] = Math.max(counters[f.category] ?? 0, Number(f.token.match(/_(\d+)\]$/)?.[1] ?? 0));
  }

  for (const entity of entities) {
    const category = NER_LABEL_TO_CATEGORY[entity.label.toUpperCase()] ?? `NER_${entity.label.toUpperCase()}`;
    const entityText = entity.text;

    let start: number;
    let end: number;
    if (entity.start != null && entity.end != null) {
      start = entity.start;
      end = entity.end;
    } else {
      const idx = text.indexOf(entityText);
      if (idx < 0) continue;
      start = idx;
      end = idx + entityText.length;
    }

    const overlap = findings.find((f) => spansOverlap(f, { start, end }));
    if (overlap) {
      if (entity.score > overlap.confidence) {
        overlap.category = category;
        overlap.confidence = entity.score;
        overlap.original = entityText;
        overlap.start = start;
        overlap.end = end;
      }
      continue;
    }

    counters[category] = (counters[category] ?? 0) + 1;
    findings.push({
      token: `[${category}_${counters[category]}]`,
      category,
      original: entityText,
      start,
      end,
      confidence: entity.score,
    });
  }

  return findings.sort((a, b) => a.start - b.start);
}

export function groupByCategory(findings: PiiFinding[]): Record<string, number> {
  const grouped: Record<string, number> = {};
  for (const f of findings) {
    grouped[f.category] = (grouped[f.category] ?? 0) + 1;
  }
  return grouped;
}
