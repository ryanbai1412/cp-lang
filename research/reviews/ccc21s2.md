# ccc21s2

Score: 4/5
Dream: 3/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R1: header derives the parity rule and the closing formula.
- R2: one parity per row and column, then `r (N - c) + c (M - r)`: the editorial.
- R6: the stroke index is read as `Int` and converted per branch (reported).
- R10: 0.01 s vs 0.04 s.

## Owed rewrites

None.

## Language gaps

- No tagged-enum read (in the report).

## Dream notes

`Row.count(x => strokes.count(R(x)).odd)` is O(M K), not the editorial's O(M + N + K); its own comment says "as parities: Row.count over a xor-fold". That caps the dream at 3: fold the strokes once into parities, then count.
