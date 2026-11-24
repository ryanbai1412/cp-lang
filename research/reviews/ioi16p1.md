# ioi16p1 review

Score: 4/5
Dream: 4/5

Scored cold against the checklist, the port and the dream (the `## Dream` section of research/frictions/ioi16p1.md).

Reads like the editorial: sort by weight, a window of the sorted order that drops its lightest
while it weighs over `u`, the first window reaching `l`. The two-pointer bookkeeping
(`lo`, `total`) is reported.

## Checklist failures

- `var (lo, total)` and the `while total > u` loop (lines 22-28) move the window by hand (R3);
  reported.

## Owed rewrites

- research/frictions/ioi16p1.md's Where cites `order.take(hi + 1).drop(lo)`; the port now
  prints `order[picked]` with `picked = lo..hi + 1`.

## Language gaps

- A shrinking window over a list and the grader task: in research/frictions/ioi16p1.md.

## Dream notes

- At bar: `order.windows(shrink_while: ...).first(...)`.
