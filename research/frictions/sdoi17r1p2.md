# Friction Report: sdoi17r1p2

## 1. An enum with two domain payloads misreads in the C++ build (`codegen`)

- **Where:** `enum Op = ... Path(x: V, y: V)`: with n = 1e5 the judge build traps "no arm of
  this `match` fits the value" on `2 30000 30000`; `cpl run` reads it right.
- **Workaround:** payloads are `1..=n`, converted with `V(x)` at each use.
- **General fix:** pack enum payloads by their declared widths (Core, enum packing).
- **Goal it hurts:** hard to get wrong (a domain payload is the obvious spelling).
- **Severity:** high. **Status:** sent to Core.

## 2. `LcaTable` is a second object beside the `Rooted` it indexes

- **Where:** `let lca = LcaTable(t)`, then `lca.lca(x, y)`; `t.lca` is O(log n).
- **General fix:** `Rooted` could build its O(1) LCA lazily on first `t.lca` call.
- **Goal it hurts:** express.
- **Severity:** low.

## Dream

`dream.cp`.
