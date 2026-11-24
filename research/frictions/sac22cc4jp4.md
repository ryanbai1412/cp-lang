# sac22cc4jp4 — Obligatory Permutation Problem

## A permutation is a `Functional` graph to get its cycles
- **Where:** `port.cp`, `Functional(p).cycles()`.
- **What felt wrong:** `Perm<V>` is a type alias of `[V]V`, so the cycle walk has to go through the
  functional-graph structure; `p.cycles()` on a permutation would say it directly. The rotation by
  `k - 1` along a cycle is index arithmetic; `cycle.ring.step` works on domains, not on a list.
- **General fix:** `cycles()` and `pow(k)` on `Perm<V>` (O(n) by cycles).
- **Goal it hurts:** express.
- **Severity:** low.

## Dream

```cp
let n = read<1..=1e5>()
let k = read<1..=1e18>()
domain V = 1..=n
let p = read<Perm<V>>()
print(p.pow(k - 1).inverse())
```
