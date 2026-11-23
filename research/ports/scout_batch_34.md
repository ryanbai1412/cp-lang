# Scout Batch 34 — Hard USACO & DMOPC Contest Classics (v2 Spec)

This batch scouts 10 diverse, previously unported hard problems from the strict `usaco*`/`dmopc*` prefix bucket in `corpus/submissions/` against the locked CP Lang v2 specification (`docs/one-page-cheat-sheet.md`, commit `3723dca`).

## Corpus Prefix Note

Inspecting `corpus/submissions/info.json` (836 unique problem codes) yields **0 `usaco*` problems** and **58 `dmopc*` problems** (45 with AC submissions). All 10 selections therefore come from unported hard DMOPC problems, with zero overlap against existing ports in `proto/ports/`, `scout_batch_*`, and `blind_critic_*`.

## Layout

- `<code>.cp` — Idiomatic CP Lang **v2 spec** ports (461 non-blank/non-comment lines total vs 1,253 raw C++ lines and 757 library-stripped C++ lines: **63.2% raw** and **39.1% library-stripped** reduction).
- `today/<code>.cp` — Runnable ports targeting today's `proto/cp.py lib-run` interpreter and `stdlib/` (548 lines total: **56.3% raw** and **27.6% library-stripped** reduction).
- `check.py` — Differential test harness compiling the 10 original AC C++ submissions with `g++ -O2 -std=c++17` and checking all 10 `today/` ports on randomized and edge-case inputs.

## Selected Problems (10)

- `dmopc21c2p6` (`3968585.cpp`) — Offline parallel binary search + descending value sweep with two Fenwick trees (bubble-sort pass queries). Raw 198 / stripped 130 / v2 89 / today 101 LOC.
- `dmopc20c6p6` (`5200785.cpp`) — Interactive water-jug capacity discovery (`@interactor` in v2; inline local grader matching `-DLOCAL` in `today/`). Raw 165 / stripped 99 / v2 80 (58 solver-only) / today 68 LOC.
- `dmopc21c2p5` (`3968384.cpp`) — Rooted tree knapsack DP with bounded degree dimension and non-prime modulus `998_244_352`. Raw 119 / stripped 81 / v2 42 / today 60 LOC.
- `dmopc21c8p5` (`4533533.cpp`) — Sparse divide-and-conquer recursive DP with dense small base table and `@memo(hash)`. Raw 91 / stripped 37 / v2 19 / today 32 LOC.
- `dmopc21c7p4` (`4825739.cpp`) — Offline coordinate-compressed custom 5-field segment-tree monoid + split prefix/suffix binary search. Raw 177 / stripped 93 / v2 77 / today 84 LOC.
- `dmopc21c2p2` (`3946515.cpp`) — Bidirectional permutation composition (`Pos` <-> `Val` bijection with prepend/append swaps and inverse query). Raw 97 / stripped 60 / v2 28 / today 30 LOC.
- `dmopc20c6p3` (`3917634.cpp`) — Retrograde directed-graph game solver using reverse adjacency and degree countdown. Raw 118 / stripped 81 / v2 32 / today 46 LOC.
- `dmopc21c2p4` (`3949962.cpp`) — Run-length compression + 3-state rolling DP (`Neutral`/`Push`/`Pull`). Raw 90 / stripped 53 / v2 25 / today 42 LOC.
- `dmopc20c7p2` (`3686612.cpp`) — Truncated square/octagon tiling outer-perimeter tracing with integer `Pt` map. Raw 107 / stripped 70 / v2 35 / today 44 LOC.
- `dmopc21c8p3` (`4537755.cpp`) — Constructive permutation via capped DP counts and k-th order-statistic extraction. Raw 91 / stripped 53 / v2 34 / today 41 LOC.

## Running Verification

```bash
python3 proto/ports/scout_batch_34/check.py           # 30 cases x 10 ports = 300 tests
python3 proto/ports/scout_batch_34/check.py dmopc21c2p6 60
```

All 10 ports pass 30/30 cases (300/300 total) against the compiled original C++ submissions. For `dmopc20c6p6`, `check.py` compiles `5200785.cpp` with `-DLOCAL` and compares the recovered capacity vector `A c_1 ... c_n` while `today/dmopc20c6p6.cp` also asserts `cap == hidden` and `queries <= 4 * n`.

See `research/archive/frictions/friction-backlog-batch-34-usaco-dmopc.md` for the full friction analysis and multi-feature collision notes.
