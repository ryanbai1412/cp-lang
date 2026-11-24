# ccc20s4

Score: 3/5
Dream: 4/5

Cold review against the idiomatic checklist, of the current `port.cp` (the friction
report's first section describes an older spec.cp).

- R1: header has Source, limits, Problem, Solution, original; no `// Bounds:` line, and
  the statement's N <= 1e6 is not stated anywhere.
- R2: "fix where A's block starts and which group follows, pair up the cross-misplaced"
  is the editorial and `seated(x, y)` is that count.
- R4: `Counted = 0..2` stores prefix counts for two groups and derives the third as
  `k - before[0] - before[1]`; that saves memory but splits one fact (a group's count)
  across `before`, `prefix_of` and `upto`.
- R5: `let s: Str = read()` is unbounded; it should be `Str<1..=1e6>`.
- R8: the formatter breaks `prefix_of` and `upto` into odd shapes (`before[\n Counted(1),\n k,\n ]`).
- R13: the `// WISH C5` comment is a stale note, not code documentation.
- R10: 0.23x time, same memory.
- Dream: one `before: [Who]List<Int>` over the doubled table and a `c[x, y]` table; it
  counts the leftover 3-cycles too, which the port's two-block formula avoids.

## Checklist failures

R1, R4, R5, R8 (formatting), R13.

## Owed rewrites

Bound the read, add `// Bounds:`, drop the WISH comment, and fold `prefix_of`/`upto`
into one cyclic `among`.

## Language gaps

No cyclic prefix-count spelling (the WISH comment's ask); see the friction report.
