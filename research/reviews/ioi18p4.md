# ioi18p4 review

Score: 4/5
Dream: 4/5

Scored cold against the checklist, the port and the dream (the `## Dream` section of research/frictions/ioi18p4.md).

Reads like the editorial: one complete switch tree, pass t reaching leaf `bitrev(t)`, the
leftmost unused leaves looping to the root, all-unused subtrees collapsing to an edge. The
judge-in-solution, the hand bit reversal and the preorder placeholder are reported.

## Checklist failures

- The header's `O(n)` (line 10) misses `Leaf.map(bitrev)` costing `depth` per leaf; it is
  O(n log n) (R1).
- `span.l..span.mid`, `Int(span.r)` (lines 48, 54-55) where decision §101 names
  `.start`/`.end` (R8).

## Language gaps

- `@grader`, `bit_rev`, and preorder allocation: in research/frictions/ioi18p4.md.

## Dream notes

- At bar: `SwitchTree` with `build(collapse:)`.
