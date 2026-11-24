# ioi19p2 review

Score: 4/5
Dream: 4/5

Scored cold against the checklist, the port and the dream (the `## Dream` section of research/frictions/ioi19p2.md).

Reads like the editorial: the two smaller sets connected, a centroid hub, a `Dsu` of the hub's
branches merged while both are too small, then two BFS patches. The bounded filtered BFS, the
sorted-tuple destructuring and the grader task are reported.

## Checklist failures

- `claim` (lines 43-52) is a hand bounded BFS (R3); reported.
- `part: [Spot]?Int` with `mark: Int` leaves the labels unbounded; `?1..=3` would state them.

## Language gaps

- All in research/frictions/ioi19p2.md.

## Dream notes

- At bar: `unite_while`, `bfs(through:).take(n)`, the three-way answer.
