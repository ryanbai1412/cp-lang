# Scout Batch 36 — Hard CCC & CCO Contest Classics (`ccc*` / `cco*`)

Ten diverse, previously unported hard Canadian Computing Competition (CCC) and
Canadian Computing Olympiad (CCO) problems ported idiomatically against the
locked CP Lang v2 specification (`docs/one-page-cheat-sheet.md`, commit
`3723dca`), alongside runnable `today/` implementations verified differentially
against the original accepted C++ submissions.

## Problems

- `ccc02s4hard.cp` (`4157744.cpp`) — CCC '02 S4 Hard: bridge-crossing DP with monotone stack and `LazySegTree` range-assign + point-update (`Entry` dual-field monoid).
- `ccc19s5hard.cp` (`5119931.cpp`) — CCC '19 S5 Hard: Triangle: The Data Structure — 2D triangular sparse-table DP growing side length by `3/2` over flat triangular storage.
- `cco18p6.cp` (`4575245.cpp`) — CCO '18 P6: Flop Sorting — divide-and-conquer in-place merge sort via median cut and three-reversal block rotation.
- `cco19p4.cp` (`3617807.cpp`) — CCO '19 P4: Card Scoring — real-valued DP (`F64`, `sqrt`) with per-rank Li Chao trees.
- `cco20p5.cp` (`4562195.cpp`) — CCO '20 P5: Interval Collection — offline segment tree over time, dual mirrored rollback Fenwick trees, and lazy-deletion min-heaps.
- `cco22p2.cp` (`4801882.cpp`) — CCO '22 P2: Rainy Markets — greedy allocation with a run-length-encoded capacity stack (`truncate`/`pop`) and difference-array shift reconstruction.
- `cco22p4.cp` (`4801884.cpp`) — CCO '22 P4: rooted binary-tree parking DP with `Mod<1_000_000_007>`, `Comb`, and `@memo`.
- `cco22p5.cp` (`4801885.cpp`) — CCO '22 P5: two-graph connectivity threshold via small-to-large DSU, rollback log, and inclusion-exclusion `HashMap`.
- `cco23p2.cp` (`5579928.cpp`) — CCO '23 P2: Real Mountains — modular sweep (`Mod<1_000_003>`) with ordered `Set` endpoints and offline `first_gt` prefix/suffix queries.
- `cco23p4.cp` (`5579929.cpp`) — CCO '23 P4: Binary String — forbidden-substring avoidance across all 14 patterns (`|t| <= 3`) via complement symmetry and greedy zero-gap splitting.

## Dual LOC Summary

Across all 10 problems:
- Raw original C++: **1,586 LOC**
- Library-stripped original C++ (template/I/O and standard DS boilerplate removed): **1,038 LOC**
- Ideal CP Lang v2 (`scout_batch_36/*.cp`): **615 LOC** (**-61.2%** vs raw C++, **-40.8%** vs library-stripped C++)
- Runnable `today/` ports (`scout_batch_36/today/*.cp`): **1,107 LOC** (**-30.2%** vs raw C++)

## Running Verification

```bash
python3 proto/ports/scout_batch_36/verify.py
```

Nine ports compile through `proto/cp.py build` + `g++ -std=c++17 -O2 -fsanitize=address,undefined` and pass 1,040 differential test cases with zero ASan/UBSan diagnostics; `today/cco19p4.cp` (floating-point `Float` + `sqrt`) runs through `proto/cp.py lib-run` and matches `3617807.cpp` within `1e-5` across 45 cases (1,085 total cases).

See `research/archive/frictions/friction-backlog-batch-36-ccc-cco.md` for detailed friction findings and concrete code examples.
