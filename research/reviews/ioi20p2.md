# ioi20p2 review

Score: 4/5
Dream: 4/5

Scored cold against the checklist, the port and the dream (the `## Dream` section of research/frictions/ioi20p2.md).

Reads like the editorial: trees by `p == 1` named by their first tower, components by `p > 0`,
a star per tree and a ring of at least three tree roots per component, then one check that `p`
matches the layout. The hand class representatives, the grader task and the edge list to a
matrix are reported.

## Checklist failures

- `tree`/`comp` as `V.find(...) ?? v` and `ring[comp[v]].push(v)` (lines 20-27) build classes
  by hand; reported.

## Language gaps

- All in research/frictions/ioi20p2.md.

## Dream notes

- At bar: `V.classes(by:)`, `group_by`, one check line.
