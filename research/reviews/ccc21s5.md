# ccc21s5

Score: 4/5
Dream: 4/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R1: header derives the per-bit difference arrays and the gcd check.
- R2: set bits on ranges, then check every requirement's gcd: the editorial.
- R3: sixteen `Diff` passes where one keyed structure would do (reported).
- R3: a `SegTree<I, Int>(gcd, a)` answers static range gcds, where std `SparseTable` also would; minor.
- R10: 0.04 s vs 0.06 s.

## Owed rewrites

None.

## Language gaps

- Sixteen difference arrays (in the report).

## Dream notes

Reads like the editorial.
