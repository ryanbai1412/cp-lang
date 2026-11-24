# noimps23p1

Score: 3/5
Dream: none (no dream.cp and no `## Dream` in the friction report)

Cold review on main 6a989896d: port, dream and friction report, not the C++. There is no dream (neither dream.cp nor a `## Dream` in the friction report), so the score is capped at 3 (research/reviews/README.md). Otherwise near exemplar: typed stations and tracks, an `IntervalSet`'s `next_gap` as the skip pointer, `buckets()` for the tracks starting at a station, and mirroring for the other direction.

## Checklist failures

- Line 39 `stop[x] = false` has no effect: every ridden track has `r > x` (initial tracks satisfy `l <= x < r`, later ones start at a swept station >= x and `l < r`) (idiomatic-code/R13).

## Owed rewrites

- Write the dream.
- Drop line 39.

## Language gaps

- None.
