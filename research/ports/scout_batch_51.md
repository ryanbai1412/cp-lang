# Scout Batch 51 — APIO, EGOI, CCO & CCC Classics against CP Lang v2.1

Ten unported, full-score accepted Olympiad submissions from `corpus/submissions/` ported idiomatically to the locked **CP Lang v2.1** spec (`docs/one-page-cheat-sheet.md` at `e7e6794`, `research/language-architecture-v2.1.md`), paired with runnable `today/` implementations verified against the original C++17 submissions.

## Problem Roster

- `apio23p2.cp` (`5593302.cpp`, APIO '23 P2 Sequence): value sweep with a prefix-balance `SegTree` using Tropical Optional `NP(none, none, 0, 0)` as monoid identity, custom `Action` range-chmax `LazySegTree`, `compress()`, and two 2D dominance sweeps. Raw C++ 274 LOC (148 library-stripped) -> 70 v2.1 LOC (135 `today/`).
- `egoi22p3.cp` (`5199301.cpp`, EGOI '22 P3 Social Engineering): `@grader` interactive edge-disjoint return game; spanning-forest DFS, bottom-up pairing of neighbors of vertex 1, and precomputed return paths. Raw C++ 214 LOC (99 library-stripped) -> 54 v2.1 LOC (69 `today/`).
- `egoi22p4.cp` (`5199667.cpp`, EGOI '22 P4 Tourists): `Rooted<City>` for O(1) tree distances, ordered `Map` ODT via `.last_le`/`.last_lt`, and per-city lazy-add treap over a flat `List<Node>` arena relying on v2.1 RHS-before-LHS evaluation and `Copy`-by-value reads. Raw C++ 321 LOC (238 library-stripped) -> 107 v2.1 LOC (149 `today/`).
- `egoi22p7.cp` (`5203797.cpp`, EGOI '22 P7 Toy Design): `@grader` persistent-DSU interactive divide-and-conquer segment tree using `Range.l`/`Range.r` splitting (`r.l..m.r`, `m.r..r.r`) and `Marks<Pin>` for O(1)-clear seen-root tracking. Raw C++ 212 LOC (81 library-stripped) -> 52 v2.1 LOC (99 `today/`).
- `cco18p2.cp` (`4801864.cpp`, CCO '18 P2 Wrong Answer): constructive 100x100 cost-matrix counterexample using default-filled 2D arrays and open-ended row slices `a.row(i)[i + 1..]`. Raw C++ 58 LOC (28 library-stripped) -> 12 v2.1 LOC (18 `today/`).
- `cco03p4.cp` (`5024536.cpp`, CCO '03 P4 Constrained Permutations): rethought from `next_permutation` brute force into an O(n * 2^n) `Subset<V>` bitmask DP. Raw C++ 53 LOC (22 library-stripped) -> 13 v2.1 LOC (23 `today/`).
- `ccc23s2.cp` (`5350805.cpp`, CCC '23 S2 Symmetric Mountains): odd/even center expansion with chained `while let` optional domain stepping and Tropical Optional `min=`. Raw C++ 58 LOC (27 library-stripped) -> 12 v2.1 LOC (18 `today/`).
- `ccc23s1.cp` (`5350804.cpp`, CCC '23 S1 Trianglane): 2xN triangle-strip perimeter using v2.1's `Bool` `Index` (`[Bool, Col]0..=1`, `for r in Bool`, `wet[not r, c]`). Raw C++ 55 LOC (24 library-stripped) -> 11 v2.1 LOC (15 `today/`).
- `ccc96s3.cp` (`5515585.cpp`, CCC '96 S3 Pattern Generator): recursive backtracking with v2.1 block-scoped `defer cur.pop()` inside each branch. Raw C++ 51 LOC (13 library-stripped) -> 19 v2.1 LOC (22 `today/`).
- `ccc20s1.cp` (`4323572.cpp`, CCC '20 S1 Surmising a Sprinter's Speed): `.adjacent`, `F64` casts, `max(gen)!`, and `"{ans:.5f}"` formatting. Raw C++ 51 LOC (15 library-stripped) -> 5 v2.1 LOC (51 `today/`).

## Totals & Verification

Across all 10 problems, raw C++ drops from **1,347 to 355 LOC (-73.6%)** and library-stripped C++ drops from **695 to 355 LOC (-48.9%)**. On the four major Olympiad problems (`apio23p2`, `egoi22p3`, `egoi22p4`, `egoi22p7`), library-stripped C++ drops from **566 to 283 LOC (-50.0%)**.

Run differential verification (326 seeded random and edge-case tests across all 10 problems, compiling copies of `today/*.cp` via `proto/cp.py build` and `g++ -std=c++17 -O2`):

```bash
python3 proto/ports/scout_batch_51/check.py
```

Detailed v2.1 ergonomic findings and remaining friction live in `research/friction-backlog-batch-51-v21-olympiads.md`.
