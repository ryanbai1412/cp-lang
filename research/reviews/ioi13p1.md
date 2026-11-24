# ioi13p1 review

Score: 3/5
Dream: 3/5

Scored cold against the checklist, the port and the dream (the `## Dream` section of
research/frictions/ioi13p1.md).

The closing combination (`r[0] + r[1] + l`, `r[1] + r[2] + 2 * l` over the radii sorted down)
reads as the header's centre-star argument. The rest is traversal bookkeeping: a hand stack
walk `spread`, `farthest`, and three walks per tree coordinated through `reach` and `seen`.
The missing forest eccentricities are reported, with the reason a per-tree `dijkstra` is too
slow.

## Checklist failures

- `spread` (lines 31-38) is a hand DFS with a `came_from` field to avoid turning back, and the
  `for root in V: if not seen[root]` loop is component discovery by hand (R3); reported.
- `type Trail` reads `t: 1..=1e4` but the `// Bounds:` line names only `n`, `m`, `l`; add
  `trails.t: 1..=1e4` (R1).

## Language gaps

- Forest eccentricities and the grader task: in research/frictions/ioi13p1.md.

## Dream notes

- The dream keeps an `Int.MAX` sentinel (`radius = [Tree] _ => Int.MAX`, then `min=`), puts two
  statements on one line with `;`, and stops at `...`. It should read
  `let radius = [Tree] t => comp.members(t).min(v => ecc[v])` and finish with the two
  closing terms, and the port be judged against that.
