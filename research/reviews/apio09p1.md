# apio09p1

Score: 3/5
Dream: 4/5

Cold review of `port.cp` against the ideal under `## Dream` in `research/frictions/apio09p1.md`
(there is no `dream.cp`), without the C++. The header proves the one-cut-then-one-cut structure
and the six layouts. `Prefix(oil)` gives square totals. Separate domains for cells
(`Row`/`Col`) and cuts (`RowCut`/`ColCut`) keep the two index spaces apart, and the four corner
tables use `p.up?`/`p.left?` with `?? 0` at the edges.

## Checklist failures
- idiomatic-code/R5: `square` returns a sentinel 0 for a square that does not fit, instead of
  `?Haul`.
- idiomatic-code/R10 (memory): 71.5 MB against the C++'s 37 MB. Most of the difference is the
  read table stored as `ll`, a codegen gap routed in the friction.

## Owed rewrites
- `square -> ?Haul` through `Row.at`/`Col.at`.

## Language gaps
- Read tables don't get range-narrowed storage (Core).

## Dream notes
Only `square` changes, so the six-layout block stays as long as it is. That block is the
problem, not translation; 4/5.
