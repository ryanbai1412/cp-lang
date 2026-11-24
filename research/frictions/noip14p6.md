# Friction Report: noip14p6

## 1. Reading a huge integer modulo a prime
- **Where:** `fn residue(s: [domain S]Char) -> Z` folds the digits and the sign by hand.
- **What felt wrong:** "this decimal literal, mod P" is the whole reading step; the dream spells it
  `s.parse<Mod<P>>()`.
- **Goal it hurts:** express.
- **Ideal code:** `let coef: [0..=n]Z = read()` reading arbitrary-length decimals straight into `Mod<P>`.
- **General fix:** `read` and `parse` for `Mod<P>` from decimal text of any length.
- **Severity:** medium.

## 2. One 61-bit prime instead of two 30-bit ones
- **Where:** `type Z = Mod<2305843009213693951>`.
- **What felt wrong:** nothing; the v0.2 port kept two residues in a `[0..2, 0..=n]Int` table to keep the
  false-positive rate low. `Mod<P>` multiplies in 128 bits above 3.03e9, so one prime does the job of two.
- **Severity:** none (a simplification).

## Measurements
Stress: 8/8 tests pass; 30/30 agree with the v0.2 port and with AC 5205623 on planted-root polynomials; max (n = 100, m = 1e6, 10^4-digit coefficients) 0.68 s vs C++ 0.69 s, same output.

## Dream
The ungated ideal (does not check today):

```cp
let (n, m): (1..=100, 1..=1e6) = read()
let coef: [0..=n]Mod<2305843009213693951> = read()
let roots = [x for x in 1..=m if coef.eval(x) == 0]
print(roots.len)
print(roots, sep: "\n")
```
