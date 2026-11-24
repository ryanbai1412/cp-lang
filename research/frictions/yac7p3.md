# yac7p3 — No More Math Homework

## `Functional` has no per-cycle tree walk
- **Where:** `port.cp`, `label` recomputes `g.root_of(v)`, its cycle index and `g.depth(v)` per
  vertex.
- **What felt wrong:** the dream walks `g.tree_from(c).depths()` from each cycle vertex; today
  each vertex looks its cycle root up and the offsets live in two side arrays `idx`, `base`.
- **General fix:** `Functional.cycles()` items carrying their hanging trees, or
  `tree_from(c) -> yields (v, depth)`.
- **Goal it hurts:** express.
- **Severity:** low.

## Dream

`dream.cp`.
