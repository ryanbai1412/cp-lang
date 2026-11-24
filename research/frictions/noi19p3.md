# Friction Report: noi19p3

The thirteen hand-rolled lazy-deletion heaps (parallel `h_val`/`h_idx` lists, `h_push`,
`h_pop`, sentinel `(-1e18, -1)` tops and `st[i] != s` skipping) are one `IndexedHeap<I, Int>`
per (from, to) side pair, so a heap holds exactly the indices on its side and `peek` is the
best move; the side is an enum, the per-side counts and capacities are `[Side]Int`, the six
orders are `[A, B, Both].permutations()`, an augmenting route is `?(gain, path)`, and
`@multitest` with typed `k: 1..=n`, `l: 1..=k` reads each case.

## 1. An indexed heap costs a remove per move
- **Where:** `shift` removes the index from its old side's three heaps.
- **What felt wrong:** the C++ (and the v0.2 port) only push and skip stale tops; the exact
  heap pays a `sift_down` with a `keys[v]` indirection per compare, 33% of the run.
- **Ideal code:** the same port.
- **General fix:** store the key beside the index in `IndexedHeap`'s heap array so compares
  read one array.
- **Severity:** low.

## Measurements
Stress: 30/30 agree with the v0.2 port and with AC. Max (T = 5, n = 2e5 each, k = 1e5, l = 1):
port 2.16 s, v0.2 port 1.60 s, C++ 1.03 s, same output. The accepted C++ is over the 1 s limit
on that input too, so tests.toml has no `--max` case.
