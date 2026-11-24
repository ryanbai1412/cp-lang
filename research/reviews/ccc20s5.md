# ccc20s5

Score: 4/5
Dream: 4/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R1: header derives the chain argument and `sum dp / n`.
- R2: one backward scan with the last occurrence and a suffix sum: the editorial.
- R5: `?Pos` for the last occurrence; `F64` only for probabilities.
- R8: `i.idx` (L30) where the corpus spells `i.ord`.
- R3: the suffix sum is kept by hand (reported).
- R10: 0.02 s vs 0.07 s.

## Owed rewrites

- `i.idx` to `i.ord` (L30).

## Language gaps

- No reverse builder exposing the suffix fold (in the report).

## Dream notes

Reads well.
