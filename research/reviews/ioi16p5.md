# ioi16p5 review

Score: 3/5
Dream: 3/5

Scored cold against the checklist, the port and the dream (the `## Dream` section of research/frictions/ioi16p5.md).

Correct and close to the header's levels-of-bits argument: write `shape(g, a)` for the
positions with bit d, compile, then ask the same shape for each position. The recursion as
levels and the interactive package are reported. The two phases repeat the same level split
by hand.

## Checklist failures

- Lines 33-40 and 44-52 both walk `for d in Level`, split each group with `partition`, and
  rebuild `groups` through `next += [high, low]` (R4); one `fn refine(split)` over the levels
  would say it once. Not reported.
- `g.partition(a => member(shape(g, a)))` asks the judge inside a partition predicate
  (line 49); fine for one call per element, but the query count is the protocol's budget.

## Owed rewrites

- One level-refinement function used by both phases, its predicate the only difference.

## Language gaps

- Levels of a recursion and the interactive package: in research/frictions/ioi16p5.md.

## Dream notes

- One line (`split_by_levels`) for the grouping; it leaves the two phases and the answer
  bits out. It should show both phases using it.
