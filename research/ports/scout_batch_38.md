# Scout Batch 38 — Hard Olympiad & DP Classics

Ten previously unported hard olympiad and DP submissions from the strict prefix bucket (`dp*`, `btoi*`, `ceoi*`, `egoi*`, `apio*`, `joisc*`, `joi*`), solved idiomatically in CP Lang v2 (`docs/one-page-cheat-sheet.md`, commit `3723dca`) alongside verified runnable ports under `today/`.

## Problems

- `apio09p1.cp` (orig `corpus/submissions/5536569.cpp`): 2D prefix sums, four grid orientations, and prefix/suffix split for three disjoint `k x k` squares.
- `apio09p2.cp` (orig `corpus/submissions/5536701.cpp`): sparse table + functional binary lifting + interval-block map for lexicographically smallest maximum independent set of intervals.
- `btoi04p3.cp` (orig `corpus/submissions/5270736.cpp`): L1 strictly-increasing isotonic regression via Pool Adjacent Violators and mergeable max-heaps.
- `btoi08p6.cp` (orig `corpus/submissions/5325518.cpp`): subset dominance boxes and Pareto skyline sweep.
- `btoi17p4.cp` (orig `corpus/submissions/4523876.cpp`): bounded search tree and submodular cut uncrossing for `(p, q)`-graph partitioning.
- `btoi18p4.cp` (orig `corpus/submissions/4656863.cpp`): circular-arc domination contraction, cyclic indexing, and odd-cycle 2-coloring.
- `dpf.cp` (orig `corpus/submissions/4131645.cpp`): AtCoder DP F longest common subsequence DP and reconstruction.
- `egoi23p2.cp` (orig `corpus/submissions/5678050.cpp`): offline time-tree rollback DFS with dynamic max-duration player leaderboard.
- `egoi23p4.cp` (orig `corpus/submissions/5678052.cpp`): dual bottleneck Floyd-Warshall closures + min/max Kruskal spanning-tree graph reconstruction.
- `egoi23p7.cp` (orig `corpus/submissions/5678057.cpp`): required-edge component contraction and complement-graph DFS tree construction.

## Verification

Run the differential checker (compiles all 10 original C++ submissions and all 10 `today/` ports, then runs 414 small/medium random and edge-case tests with semantic validators for multi-answer problems):

```bash
python3 proto/ports/scout_batch_38/check.py
```

Nine ports compile and run through `proto/cpc` (`proto/cp.py build` + `g++ -O2`), and `today/dpf.cp` runs through `proto/cpi` (`proto/cp.py lib-run`) because `proto/cpc` does not yet index `Str`. During differential testing on `apio09p1`, the harness caught a real out-of-bounds heap read (`ans[n]` when `k == 1`) in the accepted C++ submission `5536569.cpp`; `check.py` therefore validates `k == 1` against an exact brute-force oracle and `k >= 2` against both brute force and `5536569.cpp`.

See `research/friction-backlog-batch-38-olympiads.md` for full dual LOC measurements and v2 friction analysis.
