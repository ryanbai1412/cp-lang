# Friction Report: ioi08p2

## Dream

```cp
let islands = Functional([I] i => bridge[i].to)
for v in islands.tails_leaf_first():   // O(n), no sort
    ...
for ring in islands.cycles(): total += ring.cyclic_pairs_best(...)
```

## Gaps

- **No leaf-first order over a functional graph's trees.** `Functional` knows
  every island's depth, but folding the trees deepest first is a sort by
  `desc(depth)`, O(n log n) where Kahn's peel is O(n). Severity: low.
- **The best pair round a ring is two running maxima kept by hand.**
  `minus`/`plus` are optional running maxima of `reach - at` and `reach + at`.
  Severity: low.
