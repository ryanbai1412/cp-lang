# ccc14s4

Score: 4/5
Dream: 4/5

Cold review of `port.cp` and the dream (`## Dream` of the friction report, or `dream.cp`) against `handbook/guidelines/idiomatic-code-checklist.md`.

## Checklist failures

- R1: header derives the at most 2n - 1 strips per axis and why no data structure is needed.
- R2: compress, difference grid, two running-sum passes, sum of dark cells: the O(n^2) editorial.
- R3: the four corner marks and the two passes are hand-written; the passes are reported, the rectangle add is now reported too (`add_rect`: E0203).
- R8: the formatter splits the closing `sum(...)` generator across L35-41 (`for (\n j,\n j2,\n ) in`); now reported for lint-fmt.
- R10: 0.04 s vs 0.02 s at `--max`, both start-up; 34 MB for the dense grid, within 256 MB.

## Owed rewrites

None.

## Language gaps

- 2-D prefix sums and `.adjacent` on a bare domain (in the report, E0501 quoted).
- Rectangle add on a 2-D difference grid (added, with a `cpl run`).
- Formatter layout of a two-`for` generator (added).

## Dream notes

Names compression, `add_rect` and `prefix_sums`; reads like the editorial.
