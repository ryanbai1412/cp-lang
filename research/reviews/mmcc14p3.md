# mmcc14p3

Score: 4/5
Dream: 3/5

Cold review: bisect the radius with `first_true`, and the leaves-up greedy keeps the farthest uncovered vertex and the nearest outpost per vertex, as the header says (lines 17-37). Port score provisional until the dream is rewritten.

## Checklist failures

- None beyond the reported gaps.

## Owed rewrites

- None.

## Language gaps

- The optional minimum spelled `Some(if let x = near: min(x, d + 1) else: d + 1)` (line 25) and the side counter beside a bottom-up walk: both in research/frictions/mmcc14p3.md.

## Dream notes

- `t.fold_up((far: 0, near: None), merge).count_placed` leaves `merge` undefined, so the placing rule, the whole greedy, is not in the dream. It should write the merge (the `d + 1 > radius` placement and the `far + near <= radius` cover test) as the fold's step.
