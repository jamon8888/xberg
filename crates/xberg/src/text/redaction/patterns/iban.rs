//! IBAN (International Bank Account Number) detection.
//!
//! Format: two-letter ISO 3166-1 country code + two check digits + up to 30
//! alphanumeric BBAN characters. The regex matches IBANs with optional space
//! separators every four characters (the common pretty-print form). Matches
//! are further validated against the ISO 13616 mod-97 checksum.

use super::PatternMatch;
use crate::types::redaction::PiiCategory;
use once_cell::sync::Lazy;
use regex::Regex;

// Known IBAN-using country codes (ISO 3166-1 alpha-2). Filter cuts down the
// false-positive surface that any two upper-case letters would otherwise allow.
const IBAN_COUNTRIES: &[&str] = &[
    "AD", "AE", "AL", "AT", "AZ", "BA", "BE", "BG", "BH", "BR", "BY", "CH", "CR", "CY", "CZ", "DE", "DK", "DO", "EE",
    "EG", "ES", "FI", "FO", "FR", "GB", "GE", "GI", "GL", "GR", "GT", "HR", "HU", "IE", "IL", "IQ", "IS", "IT", "JO",
    "KW", "KZ", "LB", "LC", "LI", "LT", "LU", "LV", "LY", "MC", "MD", "ME", "MK", "MR", "MT", "MU", "NL", "NO", "PK",
    "PL", "PS", "PT", "QA", "RO", "RS", "SA", "SC", "SE", "SI", "SK", "SM", "ST", "SV", "TL", "TN", "TR", "UA", "VA",
    "VG", "XK",
];

static RE_IBAN: Lazy<Regex> = Lazy::new(|| {
    // Country (2 letters) + check (2 digits) + BBAN as whole 4-character groups
    // (2-7 of them) plus an optional short trailing group (1-3 chars), each
    // group preceded by at most one space. Requiring *whole* groups (rather
    // than an optional space before every single character) keeps the match
    // from bleeding into an adjacent all-caps word that happens to be
    // separated by a single space, e.g. "...0130 00 VIA SEPA" would
    // otherwise be swallowed whole because "VIA"/"SEPA" also look like valid
    // alphanumeric BBAN characters.
    Regex::new(r"\b[A-Z]{2}\d{2}(?:[ ]?[A-Z0-9]{4}){2,7}(?:[ ]?[A-Z0-9]{1,3})?\b").expect("iban regex compiles")
});

/// Find all IBAN spans in `text`, validated against the country-code allowlist,
/// length range, and the ISO 13616 mod-97 checksum.
pub fn find_all(text: &str) -> Vec<PatternMatch> {
    let upper = text.to_ascii_uppercase();

    RE_IBAN
        .find_iter(&upper)
        .filter_map(|m| {
            let raw = &upper[m.start()..m.end()];
            let cc = &raw[..2];
            if !IBAN_COUNTRIES.contains(&cc) {
                return None;
            }
            // Strip whitespace and verify total length is within the IBAN range (15-34 chars).
            let compact: String = raw.chars().filter(|c| !c.is_whitespace()).collect();
            if !(15..=34).contains(&compact.len()) {
                return None;
            }
            if !iban_checksum_valid(&compact) {
                return None;
            }
            Some(PatternMatch {
                start: m.start(),
                end: m.end(),
                category: PiiCategory::Iban,
                text: text[m.start()..m.end()].to_string(),
            })
        })
        .collect()
}

/// ISO 13616 IBAN checksum: move the first 4 characters to the end, convert
/// letters to numbers (A=10, B=11, ... Z=35), and verify the resulting
/// number mod 97 equals 1. Rejects the ~1-in-100 non-checksum-valid strings
/// that the country-code + length filter alone lets through.
fn iban_checksum_valid(compact: &str) -> bool {
    if compact.len() < 4 {
        return false;
    }
    let rearranged = format!("{}{}", &compact[4..], &compact[..4]);
    let mut remainder: u64 = 0;
    for c in rearranged.chars() {
        let value = if c.is_ascii_digit() {
            c.to_digit(10).unwrap_or(0) as u64
        } else if c.is_ascii_uppercase() {
            (c as u64) - ('A' as u64) + 10
        } else {
            return false;
        };
        // Fold digit-by-digit (or two-digit for letters) to avoid overflow
        // on IBANs up to 34 chars (~68 decimal digits after expansion).
        let digits = if value >= 10 { 2 } else { 1 };
        remainder = (remainder * 10u64.pow(digits) + value) % 97;
    }
    remainder == 1
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checksum_valid_iban_is_detected() {
        let matches = find_all("IBAN: FR7630006000011234567890189 for transfer.");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].category, PiiCategory::Iban);
        assert_eq!(matches[0].text, "FR7630006000011234567890189");
    }

    #[test]
    fn checksum_invalid_iban_is_rejected() {
        // Same IBAN as above with the last BBAN digit flipped (9 -> 8).
        let matches = find_all("IBAN: FR7630006000011234567890188 for transfer.");
        assert!(matches.is_empty());
    }

    #[test]
    fn does_not_bleed_into_trailing_uppercase_words() {
        // Regression test: the regex must stop at the end of the real IBAN and
        // not swallow following all-caps words that happen to look like
        // continued alphanumeric BBAN characters.
        let matches = find_all("Pay DE89 3704 0044 0532 0130 00 via SEPA.");
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "DE89 3704 0044 0532 0130 00");
    }
}
