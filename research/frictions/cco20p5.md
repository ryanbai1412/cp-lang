# Friction Report: cco20p5

## 1. Offline segment tree over time (`TimeSegTree<Q, T>`)

- **Area:** stdlib
- **Goal:** express
- **Severity:** medium
- **Dream:** `proto/ports/scout_batch_36/dream/cco20p5.cp:42,51,55,80-103`
- **Port:** `proto/ports/scout_batch_36/cco20p5.cp:42-50,101-131`

Dynamic-connectivity / add-remove-over-time problems map each element's `[start, end]` lifetime onto `O(log q)` nodes of a segment tree over time and DFS with rollback. Because `stdlib/` has no `TimeSegTree`, the port hand-rolls `seg[o * 2]` and `dfs`.

## 2. Rollback-capable prefix-max `Fenwick` (`RollbackFenwick<Coord, Max>`)

- **Area:** stdlib
- **Goal:** express
- **Severity:** medium
- **Dream:** `proto/ports/scout_batch_36/dream/cco20p5.cp:57-64,101-102`
- **Port:** `proto/ports/scout_batch_36/cco20p5.cp:68-85,127-129`

`std.fenwick.Fenwick` supports additive groups (`+`/`-`) without rollback; prefix-max with checkpoint/rollback forces a hand-written `i -= i & -i` / `j += j & -j` tree and `log: List<Undo>`.
