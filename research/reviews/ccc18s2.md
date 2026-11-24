# ccc18s2

Score: 4/5
Dream: 4/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R1: header says why exactly one orientation increases both ways.
- R2: rotate until rows and columns increase: the editorial.
- R3: `turned` (L14) hand-indexes a clockwise rotation, `g[I(n - 1 - c.ord), r]`; std has `transposed` but no rotation (now reported, `rotated_cw`: E0203).
- R10: 0.00 s both.

## Owed rewrites

None.

## Language gaps

- No strict `is_sorted` (in the report).
- `iterate` needs a `?T` step (in the report).
- No grid rotation in std (added, with a `cpl run`).

## Dream notes

`g.rows.each()` is an odd spelling for walking the rows; `g.rows` should iterate directly. Otherwise the editorial in three lines.
