# Scout batch 18: statement-first graph, tree, and data-structure wave

Stage-1 statement-first language-ergonomics scout (Wave 2) over ten hard
unported DMOJ problems (17–25 points) targeting trees, graphs, range queries,
segment trees, Fenwick trees, rollback DSU, sparse tables, and max flow.

## Protocol

For every problem, DMOJ API v2 metadata (`https://dmoj.ca/api/v2/problem/<code>`)
was fetched first (noting that the API omits statement bodies and HTML pages
are behind Cloudflare Turnstile), and the idiomatic CP Lang solution at the
top level of this directory was written **statement-first** from the canonical
contest specification before opening any file in `corpus/submissions/`. Only
after all ten top-level `.cp` files were written were the original AC C++
submissions inspected to verify I/O details, build runnable `today/` ports,
run randomized differential tests, and compare algorithms and max-N risks.

The full prose-and-snippet findings live in
[`research/friction-backlog-batch-18-statement-first.md`](../../../research/friction-backlog-batch-18-statement-first.md).

## Problems

- **LazySegTree + interval DP**: `dpw.cp` (AtCoder DP W — Intervals, 20pt, `4133460.cpp`).
- **Fenwick + adjacent-value inversion updates**: `cco21p1.cp` (CCO '21 P1 — Swap Swap Sort, 17pt, `4538969.cpp`).
- **SegTree over min-plus matrices (layered DAG)**: `btoi17p2.cp` (Baltic OI '17 P2 — Toll, 20pt, `4456186.cpp`).
- **Offline coordinate-compressed LazySegTree + prefix-max search**: `ioi05p2.cp` (IOI '05 P2 — Mountain, 20pt, `5032753.cpp`).
- **SparseTable (argmax RMQ) + MaxHeap interval splitting**: `noi10p2.cp` (NOI '10 P2 — Super Piano, 25pt, `5670502.cpp`).
- **Rollback parity-DSU + Divide & Conquer**: `btoi20p3.cp` (Baltic OI '20 P3 — Joker, 25pt, `4656866.cpp`).
- **Directed graph reverse peeling**: `cco21p4.cp` (CCO '21 P4 — Travelling Merchant, 25pt, `6419583.cpp`).
- **Cycle peeling + maximum closure**: `noi09p4.cp` (NOI '09 P4 — Plants vs. Zombies, 25pt, `5144326.cpp`).
- **3-state bottom-up tree DP**: `ccc22s5.cp` (CCC '22 S5 — Good Influencers, 17pt, `4371349.cpp`).
- **Multitest Dijkstra + Kruskal reconstruction tree + binary lifting**: `noi18p1.cp` (NOI '18 P1 — Return, 17pt, `6512702.cpp`).

## Dual LOC comparison (non-blank, non-comment LOC, excluding `@brute`)

Across all ten problems, the original C++ submissions total **1,567 raw LOC**
and **633 library-stripped LOC** (after removing headers, debug macros, `main()`
boilerplate, standalone braces, and generic DS templates such as `LazySegTree`,
`ST`, `Bit`, `kactl::Dinic`, plain `DSU`, `dijkstra`, and `BLifter`, while
keeping problem-specific structures like `Mat`, dynamic segtree, 64-ary bit-trie,
rollback parity-DSU, and problem DP/DFS routines).

- Ideal statement-first CP Lang (`*.cp`): **517 LOC** (**67.0% raw**, **18.3% stripped** reduction).
- Runnable `today/*.cp` (on `proto/cp.py lib-run`): **557 LOC** (**64.5% raw**, **12.0% stripped** reduction).

Where `stdlib/` covered the core structure (`noi10p2`, `cco21p4`, `dpw`,
`cco21p1`, `ioi05p2`), stripped LOC dropped by 24–51%. Where `stdlib/` ran out
(`btoi20p3` rollback parity-DSU, `noi18p1` Kruskal DSU + binary lifting,
`noi09p4` cycle-pruning `topo` + flow sentinels, `ccc22s5` `?Int` lifting),
stripped LOC stayed near parity with C++ — Pinpointing the highest-leverage
stdlib and core gaps in the backlog.

## Verification

Run the differential check against all ten original C++ submissions (compiles
the originals into a temporary directory and executes `today/*.cp` via
`proto/cp.py lib-run`):

```sh
python3 proto/ports/scout_batch_18/check.py
```

All ten `today/` ports pass 30/30 randomized and edge-case differential inputs
(300/300 total) against the compiled `corpus/submissions/*.cpp` binaries, and
`cco21p4`'s statement-first sort-then-peel algorithm was additionally stressed
on 500 small dense graphs against `6419583.cpp` with 0 mismatches.
