# ioi15p1 review

Score: 4/5
Dream: 4/5

Scored cold against the checklist, the port and the dream (the `## Dream` section of research/frictions/ioi15p1.md).

Reads like the editorial: `cw`/`ccw` over `Team.cuts` as O(1) recurrences, and the best split
with or without one lap, in one line. The cut stepping by `k` through `Int` and the dependent
tuple read are reported, the first with its E0203 run.

## Checklist failures

- `Team.cuts.at(Int(t.r) - k) ?? Team.cuts.first!` and its mirrors (lines 23, 25, 28) step a
  cut by `k` through `Int` (R6); reported.

## Language gaps

- All in research/frictions/ioi15p1.md.

## Dream notes

- At bar: `t.r.prev(k, clamp: true)` and one `print` line.
