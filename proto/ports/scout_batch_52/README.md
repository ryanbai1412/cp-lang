# Scout Batch 52 — 15 Hard Unported Olympiad Problems in Locked CP Lang v2.2

This directory contains 15 hard Olympiad problems ported from `corpus/submissions/` into locked 34-keyword CP Lang v2.2, paired with runnable `today/` implementations compiled via the real Rust `cpl build` compiler and differentially verified against the original C++17 submissions under ASan/UBSan.

## Problem Roster

Six IOI problems:

- `ioi14p6io.cp` — IOI '14 P6 (Holiday, `4691224.cpp`): divide-and-conquer DP with sqrt-bucket top-k sums.
- `ioi13p3.cp` — IOI '13 P3 (Wombats, `4722414.cpp`): block-decomposed segment tree over Monge min-plus matrices (`@grader`).
- `ioi14p1.cp` — IOI '14 P1 (Rail, `4691718.cpp`): interactive distance-based station reconstruction (`@grader`).
- `ioi22p5.cp` — IOI '22 P5 (Insects, `4786085.cpp`): randomized binary search on max frequency via set membership (`@grader`).
- `ioi17p4.cp` — IOI '17 P4 (The Big Prize, `5659145.cpp`): memoized prefix-count divide-and-conquer search (`@grader`, `@memo`).
- `ioi20p5.cp` — IOI '20 P5 (Mushrooms, `4736004.cpp`): depth-2 decision-tree search + interleaved bulk queries (`@grader`).

Six NOI problems:

- `noi23p4.cp` — NOI '23 P4 (Trade, `5723063.cpp`): binary-tree subtree Dijkstra with path-weight shift and `Mod<998244353>`.
- `noi19p3.cp` — NOI '19 P3 (Sequence, `5669058.cpp`): 4-state min-cost max-flow simulation over 16 `SortedBag` edge sets.
- `noi21p5.cp` — NOI '21 P5 (Code, `3927923.cpp`): implicit treap maintaining 2x2 `Mod<998244353>` continued-fraction matrices with lazy flip/reverse.
- `noi08p5.cp` — NOI '08 P5 (Clouds, `6067573.cpp`): reflected-coordinate 2D Fenwick tree with periodic wrap clamping.
- `noi02p6.cp` — NOI '02 P6 (Robots, `3754486.cpp`): prime-factorization parity DP modulo `10000`.
- `noi02p2.cp` — NOI '02 P2 (Savage / Ring Interactive, `4793917.cpp`): interactive ring truth/liar/wildcard solver.

Three CTSC / NOIP / IATI problems:

- `ctsc04p4.cp` — CTSC '04 P4 (Station, `4529641.cpp`): tree DP with knapsack merging and all-pairs shortest-path preprocessing.
- `noip20p3.cp` — NOIP '20 P3 (Ball Move, `5445674.cpp`): constructive divide-and-conquer color-partitioning tower sort.
- `iati21a13.cp` — IATI '21 A13 (Tree Activation, `5052303.cpp`): Euler-tour offline activation sweep + 2D range counting.

## Verification

Run the audit and full 750-case differential suite:

```bash
python3 proto/ports/scout_batch_52/test_batch_52.py --cases 50
```

Results on current `main`:

- Real Rust `cpl check` on ideal v2.2 ports: **15/15 PASS**.
- Real Rust `cpl build` on ideal v2.2 ports: **0/15 PASS** (blocked by compiler lowering gaps cataloged in `research/friction-backlog-batch-52-v22-olympiad.md`).
- Real Rust `cpl build` + `g++ -O2 -fsanitize=address,undefined` differential verification of `today/` ports vs original C++17: **15/15 PASS (750/750 cases, 50 per problem)**.

## Dual LOC Summary

Across all 15 problems, ideal CP Lang v2.2 cuts **2,531 raw C++ LOC** down to **871 LOC (-65.6%)** and **1,999 library-stripped C++ LOC** down to **871 LOC (-56.4%)**. See `research/friction-backlog-batch-52-v22-olympiad.md` for per-problem breakdowns and concrete language/compiler findings.
