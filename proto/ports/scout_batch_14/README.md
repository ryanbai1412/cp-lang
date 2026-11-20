# Scout batch 14: advanced DP

Stage-1 language-ergonomics scout over ten hard dynamic programming
submissions from `corpus/submissions/`, spanning digit/carry DP, interval DP
(including Knuth optimization), SOS and broken-profile bitmask DP, ternary
prime-mask DP, expected-value/probability DP, and WQS/event-sweep DP with Li
Chao trees.

The findings are written up in
[`research/friction-backlog-batch-14.md`](../../../research/friction-backlog-batch-14.md)
under item numbers `P1` through `P14`.

## Layout

Each problem appears as an idiomatic CP Lang sketch at the top level of this
directory with `// WISH Pn` markers on lines the current prototype compiler
rejects. The eight integer and modular problems also have buildable,
differentially verified ports in `today/`:

- **Digit / carry DP**: `noip21p2.cp` (NOIP 2021 Sequence, `5445677.cpp`, builds in `today/`).
- **Ternary prime-mask + inclusion-exclusion DP**: `noi15p3.cp` (NOI 2015 Sushi Dinner, `3782724.cpp`, builds in `today/`).
- **Interval DP**: `csps21p2.cp` (CSP-S 2021 Bracket Sequences, `4819734.cpp`, builds in `today/`) and `dmopc18c1p6.cp` (DMOPC 2018 C1 P6 Sorting, Knuth-optimized, `4155900.cpp`, builds in `today/`).
- **SOS & broken-profile bitmask DP**: `noip00p4.cp` (NOIP 2000 Square Number Picking, SOS DP, `5067920.cpp`, builds in `today/`) and `noi23p2.cp` (NOI 2023 D1T2 Guiyang Tree, `5723059.cpp`, builds in `today/`).
- **Probability / expected-value DP**: `noi05p4.cp` (NOI 2005 CongCong and KeKe, `5032733.cpp`, sketch only — blocked on `Float`, `P4`) and `ccc20s5.cp` (CCC 2020 S5 Josh's Double Bacon Deluxe, `5032745.cpp`, sketch only — blocked on `Float`, `P4`).
- **Convex hull / Li Chao / WQS DP**: `ioi16p6io.cp` (IOI 2016 Aliens, `4730558.cpp`, builds in `today/`) and `noi19p1.cp` (NOI 2019 Route, `3790564.cpp`, builds in `today/`).

## Size comparison (non-blank, non-comment LOC)

Across all ten problems, the original C++ totals **1,243 raw LOC** (and **845
stripped LOC** after removing headers, macros, `using`/`typedef`, and standalone
brace lines). The ten ideal CP Lang sketches total **327 LOC** — a **73.7% raw**
and **61.3% stripped** reduction.

On the eight buildable problems (`1,075` raw / `731` stripped C++ LOC), the
working `today/` ports take **520 LOC** while their ideal sketches take **267
LOC**. Nearly all of the 253-line gap between `today/` and ideal comes from two
missing capabilities: userland `struct` + `LiChao` imports (`ioi16p6io`: 115 ->
36 LOC; `noi19p1`: 106 -> 30 LOC) and ternary `Index` + `ModR` (`noi15p3`: 91 ->
44 LOC).

## Verification

Run the differential check against the original C++ submissions (compiles from
temporary copies so tracked `.cp` files are never modified):

```sh
python3 proto/ports/scout_batch_14/today/stress.py
```

All eight `today/` ports build with `proto/cp.py build` + `g++ -O2 -std=c++17`
and pass 213/213 randomized and sample differential cases against the originals.
`noi05p4.cp` and `ccc20s5.cp` are not runnable today because `Float` (`P4`) is
not yet implemented in `proto/cpc`.
