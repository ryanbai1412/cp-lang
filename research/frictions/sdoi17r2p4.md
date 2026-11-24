# Friction report: sdoi17r2p4

SDOI 2017 Genius Hacker: Dijkstra over roads, with the turns at each town
carried by prefix chains over its roads sorted by trie preorder.

## Road and chain nodes share an index space by offsetting

- **Where:** `spec.cp`: `domain Node = [9 * m]`, `let at = (r: Road) => Node(r.idx)`
  and the chain nodes `Node(m + used + i)`.
- **Still open (2026-11-23):** `domain Node = Road + Aux` checks, but
  `Node(Road(0))` / `Node(Aux(1))` is E0302 (expected `Int`, found `Road`).
  Repro: `domain A = [n]; domain B = [2 * n]; domain N = A + B; let x = N(A(0))`.
  Already in `plan/domains/compiler/worklog.md` (cf2022e1, sum domains).
- **What felt wrong:** roads and chain nodes are two index spaces the
  contestant keeps apart, but one graph needs one node type, so the chain
  nodes are offset past the roads by hand; nothing stops `Node(i)` for a
  chain index landing on a road.
- **Goal it hurts:** hard to get wrong (wrong index space).
- **Ideal code:** `domain Node = Road + Aux`, `Node(r)`, `Node(Aux(used + i))`.
- **General fix (core):** sum domains (spec R02.9.6) with injections from
  each part, stored as one dense range.
- **Severity:** medium.

## Measurements

Largest test (10: n = m - 1 = 5e4, k = 2e4, a path at max cost and LCP):
`cpl run` 1.5 s, 457 MB; the accepted C++ (`corpus/submissions/5515890.cpp`)
agrees on all 10 tests and 74 generated inputs.

## An O(1) `LcaTable` costs 6 KB of C++
- **Where:** `port.cp`, `trie.lca(...)` (O(log k)) where the dream builds `LcaTable(trie)`.
- **What felt wrong:** `LcaTable` takes the generated C++ from 32 076 to 38 004 bytes, over the
  32 768-byte target (`corpus/differential.py` flags `over-32kb`), so the port keeps the
  binary-lifting `Rooted.lca`, which is fast enough here.
- **General fix:** `LcaTable` built on the Euler tour `Rooted` already stores, with a smaller
  sparse table in the runtime.
- **Goal it hurts:** fast to run (the O(1) table is the natural choice).
- **Severity:** low.

## Dream

`dream.cp`.
