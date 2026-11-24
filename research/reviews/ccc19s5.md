# ccc19s5

Score: 4/5
Dream: 4/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R1: header derives the growth step and the cost.
- R2: grow the apex table by about half each round, in place: the editorial.
- R6: `best: [0..n, 0..n]Int` (L14) uses plain ranges and the upper half is unused (reported, E0301 quoted).
- R10: 0.15 s vs 0.50 s.

## Owed rewrites

None.

## Language gaps

- No triangle domain or checked domain offsets (in the report).

## Dream notes

`Triangle<Row, Int>` names the structure; the `iterate` step is wordy.
