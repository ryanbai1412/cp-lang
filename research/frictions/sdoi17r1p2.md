# Friction Report: sdoi17r1p2

## 1. `LcaTable` is a second object beside the `Rooted` it indexes

- **Where:** `let lca = LcaTable(t)`, then `lca.lca(x, y)`; `t.lca` is O(log n).
- **General fix:** `Rooted` could build its O(1) LCA lazily on first `t.lca` call.
- **Goal it hurts:** express.
- **Severity:** low.

## Dream

`dream.cp`.
