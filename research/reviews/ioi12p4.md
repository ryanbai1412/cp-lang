# ioi12p4 review

Score: 4/5
Dream: 4/5

Scored cold against the checklist, the port and the dream (the `## Dream` section of
research/frictions/ioi12p4.md).

Reads like the editorial: one `crossings` function per axis, runs of touching blocks, the
run tree via a std `Rooted`, and `w * (n - w)` per edge. The comments state the "no holes means
a tree" argument the code relies on. Gaps (pair-keyed `HashMap`, `(Run, Run)` pairs into a
`Graph`, subtree sizes as a loop, the shape not a type, the grader task) are reported.

## Checklist failures

- `run_of: Map<(Int, Int), 0..1e5>` holds raw run indices that become `Run(...)` only after
  the `Run` domain exists (line 41): run ids cross through `Int` (R6). The domain can only be
  declared once `size` is complete, which is part of the reported tree-building gaps.
- `size.last! += 1` and `last: ?(Int, Int)` build the runs by hand where the dream's
  `runs(by:)` names them.

## Language gaps

- All in research/frictions/ioi12p4.md.

## Dream notes

- Names `runs`, `touching` and `subtree_sizes`; `except the root` is invented syntax for
  "skip the root", and `touching` is undefined, but the shape is the editorial's.
