/**
 * Validate a Polish PESEL national ID checksum.
 * Mod-10 algorithm with official weights from Polish GUS.
 * https://en.wikipedia.org/wiki/PESEL
 */
export function isValidPesel(pesel: string): boolean {
  if (!/^\d{11}$/.test(pesel)) return false;
  const digits = pesel.split("").map(Number);
  const weights = [1, 3, 7, 9, 1, 3, 7, 9, 1, 3];
  const sum = weights.reduce((acc, w, i) => acc + w * (digits[i] ?? 0), 0);
  const checkDigit = digits[10] ?? 0;
  const expected = (10 - (sum % 10)) % 10;
  return expected === checkDigit;
}

/**
 * Validate a Dutch BSN national ID checksum.
 * Mod-11 algorithm per official Dutch RvIG specification; last digit has weight -1.
 * https://en.wikipedia.org/wiki/Burgerservicenummer
 */
export function isValidBsn(bsn: string): boolean {
  if (!/^\d{9}$/.test(bsn)) return false;
  const digits = bsn.split("").map(Number);
  const weights = [9, 8, 7, 6, 5, 4, 3, 2, -1];
  const sum = weights.reduce((acc, w, i) => acc + w * (digits[i] ?? 0), 0);
  return sum % 11 === 0;
}

/**
 * Validate a Belgian Registre National checksum (97-modulo).
 * Format: YYMMDDXXXXX (6-digit birth date + 3-digit sequence + 2-digit check).
 * Century is ambiguous from YY alone, so both the pre-2000 and post-2000
 * formulas are tried.
 */
export function isValidBelgianRegistre(num: string): boolean {
  if (!/^\d{11}$/.test(num)) return false;
  const n = Number(num.slice(0, 9));
  const checkActual = Number(num.slice(9, 11));
  if (97 - (n % 97) === checkActual) return true;
  return 97 - ((2_000_000_000 + n) % 97) === checkActual;
}
