# Scout Batch 43 — Hard ETSR '23 Classics (`etsr*` prefix bucket)

Ten diverse, previously unported hard problems from the **ETSR '23** series (excluding `etsr23r1p2` from Batch 6 and `etsr23r3p2` from Batch 5), ported against the locked CP Lang **v2 spec** (`docs/one-page-cheat-sheet.md`, `3723dca`) with runnable `today/` implementations differentially verified against the compiled original AC C++ submissions.

*(Note: Batch 43 was originally assigned `dmopc14*`–`dmopc19*`, which contained only two corpus entries, both already ported; redirected to the unclaimed `etsr*` bucket.)*

## Problems Included

- `etsr23r1p3.cp` (`corpus/submissions/5494303.cpp`): All-permutation subsequence check via backward occurrence table + `Subset<Alpha>` bitmask DP (`@multitest`).
- `etsr23r1p4.cp` (`corpus/submissions/5495163.cpp`): Segment Tree Beats for range add, range Euclidean floor-division, range min, and range sum.
- `etsr23r2p2.cp` (`corpus/submissions/5494433.cpp`): Interval DP for minimum `+`/`*` insertions in a digit string to hit target `<= 250` (`@multitest`).
- `etsr23r2p3.cp` (`corpus/submissions/5494453.cpp`): Bitwise `ctz` divisibility characterization on sorted triples (`@multitest`).
- `etsr23r2p4.cp` (`corpus/submissions/5494510.cpp`): Bounded-window XOR scheduling DP via `@memo(hash)` over a sliding `Subset<Win>` (`@multitest`).
- `etsr23r3p1.cp` (`corpus/submissions/5527625.cpp`): Clamped affine + linear-in-initial-x composition via coordinate compression and `LazySegTree` `max_right`/`min_left`.
- `etsr23r3p3.cp` (`corpus/submissions/5521298.cpp`): NOI 2018 Return — Dijkstra + Kruskal Reconstruction Tree + `Rooted.lift_while` replacing a 110-line persistent segment-tree DSU (`@multitest`).
- `etsr23r4p1.cp` (`corpus/submissions/5521420.cpp`): Permutation cycle decomposition (`Perm<V>.cycles()`) + `Subset<Big>` 2-coloring for balanced brackets, with a co-located `@checker`.
- `etsr23r4p2.cp` (`corpus/submissions/5529698.cpp`): Extended Euler totient power tower with runtime modulus `Mod<p>` and shift-clamping `LazySegTree`.
- `etsr23r4p3.cp` (`corpus/submissions/5635626.cpp`): ZJOI 2016 Small Stars — inclusion-exclusion over `Subset<V>` + `Rooted<V>` postorder tree DP in wrapping `U64`.

## Verification

Run the differential harness against all 10 compiled original C++ submissions (30 randomized and edge-case inputs per problem, 300 checks total):

```bash
python3 proto/ports/scout_batch_43/check.py
```

Dual LOC summary across all 10 problems (non-blank, non-comment lines): **1,455** raw C++ LOC -> **798** library-stripped C++ LOC -> **376** v2 CP Lang LOC (**74.2%** raw reduction, **52.9%** library-stripped reduction), and **509** runnable `today/` LOC (**65.0%** raw reduction, **36.2%** library-stripped reduction). Detailed analysis and friction findings live in `research/archive/frictions/friction-backlog-batch-43-etsr.md`.
