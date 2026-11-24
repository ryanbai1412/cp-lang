# shoi16p1 — Scout Guards

## A signed DP index needs `Reach(...)`/`.label` at every shift
- **Where:** `port.cp`, `best[c, Reach(-r.label)]`, `r.label >= 0`, `r == Reach.last!`.
- **What felt wrong:** `domain Reach = -k..=k` is the right type for the reach, but the DP
  reads `-r` and `r + 1` constantly, and a domain value supports neither `-`, `+ 1` nor a
  comparison with an `Int`; every read goes through `Reach(...)`, `.label` or `.next!`.
- **General fix:** for a domain of integer labels, `-r` and `r.label` comparisons could be
  allowed when the result provably stays in range (as `r.next` already is).
- **Goal it hurts:** express.
- **Severity:** low.

## `t.up` can't carry a per-vertex array
- **Where:** `port.cp` walks `t.order.rev` over a `[V, Reach]Int` table.
- **What felt wrong:** the dream's `t.up((v, kids) => [Reach] r => ...)` builds each vertex's
  row from its children's rows; `Rooted.up` folds a child value per edge with a `Monoid`, so
  a row DP that needs sums and a min of differences over the same children doesn't fit.
- **General fix:** a `Rooted.up` overload whose step sees `v` and the children's results
  (`(v, kids: yields T) -> T`).
- **Goal it hurts:** express.
- **Severity:** medium.

## Dream

`dream.cp`.
