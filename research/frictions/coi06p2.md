# coi06p2 friction

The old port built the DFS tree, the low-links and the subtree test by hand, with a
`-1` parent sentinel and raw `Int` towns. The port now takes the tree from
`Graph.dfs_tree`, answers "under" with `Rooted.is_anc` and finds C's child toward x
with `Rooted.jump`; only the low-link fold stays a loop.

## Dream

```cp
let low = tree.up(min, lift: v => tree.pos(v) min map.out(v).filter(a => not tree_road[a.e]).map(a => tree.pos(a.to)).min())
```

## The low-link fold is a loop

- **Where:** `low` is filled by a reverse-preorder loop that pushes each town's
  value into its parent. `Rooted.up` folds a monoid over children, but `min` over a
  preorder position has no identity to name (there is no `Monoid` for "smallest
  position"), so the one-line fold above does not check.
- **Workaround:** `for v in tree.at.rev:` with `low[p] min= low[v]`.
- **General fix:** a `Monoid` for `min`/`max` over an ordered domain index (identity
  `None` in a `?Pos`), so `tree.up(min, ...)` reads like the dream.
- **Status:** open.

## The checked-in port failed `cpl check`

- The old port's `let w = adj[v][tried[v]]` then `kids[v].push(w)` is E0603 (a plain
  `Town` read out of a nested list is treated as lent). Sent to Core with the repro
  `var xs: List<List<Int>> = [[1]]` / `let x = xs[0][0]` / `ys.push(x)`. The rewrite
  no longer reads nested lists, so it does not depend on the fix.

Measured: `cpl stress` agrees with the accepted C++ on 30 cases (size 60) and on the
`--max` case (N = 1e5, E = 5e5, Q = 3e5): 0.21 s against the C++'s 0.22 s.
