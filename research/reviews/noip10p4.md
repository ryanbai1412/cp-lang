# noip10p4

Score: 4/5
Dream: 3/5

Cold review: a memoised flow gives each top cell its span of bottom cities, then the dry count or a greedy interval cover (lines 25-52), as the header argues. Port score provisional until the dream is rewritten.

## Checklist failures

- None beyond the reported gaps.

## Owed rewrites

- None.

## Language gaps

- The interval-hull monoid and `min_cover`: both in research/frictions/noip10p4.md, now with the `cpl run` errors.

## Dream notes

- `count(c not in union(tops) for c in C)` counts cities outside the hulls, but a hull is only the reachable set when every city is fed; the port counts `not seen[R.last!, c]` (line 37), which is right. The dream should count unreached cities.
