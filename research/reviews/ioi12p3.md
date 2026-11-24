# ioi12p3 review

Score: 3/5
Dream: 3/5

Scored cold against the checklist, the port and the dream (the `## Dream` section of
research/frictions/ioi12p3.md).

The header gives the right idea (texts form a tree, commands name nodes, skew-binary jumps),
and `history` of `Text` nodes makes undo a lookup. But the level-ancestor structure is
hand-rolled as parallel arrays `len`, `last`, `up`, `jump` (lines 15-18) with the jump rule
inline, and the commands decode through `op: Char`. Both are reported.

## Checklist failures

- `len`, `last`, `up`, `jump` are four parallel arrays kept in sync by hand for one structure
  (exemplar rule: a structure with an invariant gets a type); reported as the `JumpTree` gap.
- `var texts` and `Text.nth(texts + 1)` count nodes by hand to mint the next `Text` (R4).
- `history[history.len - 1 - x]` indexes from the end by hand.

## Language gaps

- Tagged-enum command reads, a std level-ancestor structure, the grader task: all in
  research/frictions/ioi12p3.md.

## Dream notes

- Names the right object (`PersistentStack<Char>`, one version per command) and the command
  enum, then stops at `...`. It should show the three arms, e.g.
  `match op: T(c) => versions.push(versions.last.push(c)); U(x) => versions.push(versions[-1 - x]); P(i) => print(versions.last.at(i))`.
