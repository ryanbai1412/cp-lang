# ccc19s3

Score: 3/5
Dream: 4/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R1: header states forced cells, the fill and the guess.
- R2: fill forced cells until none is left, then guess: the original's construction, which the header names.
- R3: `read_cell` hand-parses `X` as `None`; now reported (`read(none: "X")`: E0302).
- R4: `lines` spells every row and column as cell tuples (reported).
- R10: 0.00 s both.

## Owed rewrites

None.

## Language gaps

- No retry-until-some helper (in the report).
- No `rows`/`cols` views over cells (in the report).
- No marker-token read into `?Int` (added, with a `cpl run`).

## Dream notes

`while settle(mut sq)` hides the fill in the loop condition, but `mut` makes it visible; acceptable.
