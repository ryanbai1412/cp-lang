# modinv

Score: 4/5
Dream: 4/5

Cold review: extended Euclid on `U64` with the sign flag the header justifies (coefficients alternate in sign and stay under M), and the N = 0 case derived in the header (lines 12-21).

## Checklist failures

- None beyond the reported gaps.

## Owed rewrites

- None.

## Language gaps

- No `inverse_mod` over `U64` (std `Mod<P>` needs a compile-time `Int` modulus): in research/frictions/modinv.md.

## Dream notes

- None.
