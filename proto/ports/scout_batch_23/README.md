# Scout Batch 23: Hard Combinatorics, Game Theory, Expected Value & Polynomials/NTT

Ten unported hard problems from `corpus/submissions/` spanning modular combinatorics (`Mod<P>`, `Comb<P>`, `ModR`, non-standard prime modulus `1024523`, SOS/bitmask inclusion-exclusion), probability / expected value DP (both modular and floating-point), formal power series / polynomial interpolation, and retrograde / minimax game theory.

Every top-level `.cp` file was written **statement-first** (from `dmoj.ca/api/v2/problem/<code>` metadata and published contest specifications) before opening the corresponding `corpus/submissions/*.cpp` file. Each problem also has a runnable counterpart in `today/` verified differentially against the compiled corpus C++ submission on randomized and edge-case inputs.

## Verification

Run the full differential test suite (20 randomized + edge cases per problem, 200 cases total):

```bash
python3 proto/ports/scout_batch_23/check.py
```

Run a single problem with more cases:

```bash
python3 proto/ports/scout_batch_23/check.py noi18p2 50
```

All 10 ports in `today/` pass 20/20 differential cases on `proto/cp.py lib-run`.

## Problems

- **noi18p2** (`NOI '18 P2 - Inverse`, 25pt, `corpus/submissions/6512891.cpp`): Ballot/reflection-principle counting of 321-avoiding permutations strictly larger than `q` modulo 998244353, multi-test. Raw C++ 240 LOC (49 core) vs 28 ideal CP / 40 today.
- **ioi08p3** (`IOI '08 P3 - Fish`, 35pt, `corpus/submissions/5032789.cpp`): Runtime composite modulus `ModR(M)` + product `SegTree` over gem ranks ordered by max length. Raw C++ 153 LOC (66 core) vs 45 ideal CP / 70 today. Note: `5032789.cpp` has `1 << (__lg(n - 1) + 2)` UB when `F == 1` or `K == 1`; `today/ioi08p3.cp` handles `F = 1, K = 1` cleanly.
- **noi09p5** (`NOI '09 P5 - Pipe Marbles`, `corpus/submissions/5108108.cpp`): Combinatorial pair-counting DP (`sum a_s^2`) modulo non-standard prime `1024523`, rolled in `O(n^2)` memory. Raw C++ 252 LOC (49 core) vs 28 ideal CP / 30 today.
- **noimps22p4** (`NOI '22 MPTS P4 - Card`, 25pt, `corpus/submissions/4547602.cpp`): Small/large prime split, SOS (zeta) transform over `2^13` masks, and per-query submask inclusion-exclusion modulo 998244353. Raw C++ 177 LOC (114 core) vs 73 ideal CP / 115 today. Note: `4547602.cpp` allocates a 9.5 MB VLA on the stack (segfaults under 8 MB default stack).
- **noi19p5** (`NOI '19 P5 - Landlords`, 25pt, `corpus/submissions/5652798.cpp`): Modular expected value tracked as a degree-2 polynomial over `Mod<998244353>` via 3-point interpolation per shuffle. Raw C++ 250 LOC (57 core) vs 49 ideal CP / 61 today. Note: `5652798.cpp` crashes with `assert(y == 1)` when `n <= 2`; `today/noi19p5.cp` handles `n = 1, 2`.
- **dpi** (`AtCoder EDPC I - Coins`, `corpus/submissions/4131648.cpp`): Floating-point probability DP / polynomial product `prod ((1 - p_i) + p_i x)` for strictly more heads than tails (`10^-9` precision). Raw C++ 71 LOC (24 core) vs 15 ideal CP / 19 today.
- **noi07p1** (`NOI '07 P1 - Social Network`, 15pt, `corpus/submissions/5032737.cpp`): All-pairs shortest paths with exact path counts and floating-point betweenness centrality to 3 decimal places. Raw C++ 151 LOC (91 core) vs 42 ideal CP / 42 today.
- **egoi22p2** (`EGOI '22 P2 - Lego Wall`, 15pt, `corpus/submissions/5199888.cpp`): Formal power series inversion `S(x) = 1 - T(x)^(-1)` modulo `10^9 + 7` for solid brick walls (`w * h <= 250000`). Raw C++ 184 LOC (50 core) vs 19 ideal CP / 19 today. (Initial statement-first recall used `1x2`/`1x3` bricks instead of `1x1`/`1x2`, corrected upon inspecting `5199888.cpp`.)
- **ioi17p3** (`IOI '17 P3 - Toy Train`, 30pt, `corpus/submissions/3921067.cpp`): Büchi / nested retrograde attractor fixed point on a directed graph. Raw C++ 148 LOC (76 core) vs 64 ideal CP / 80 today.
- **coci21c1p2** (`COCI '21 Contest 1 #2 - Kamencici`, 10pt, `corpus/submissions/4080842.cpp`): 3D zero-sum interval game DP (`win(l, r, my_red)`). Raw C++ 92 LOC (45 core) vs 15 ideal CP / 23 today. Note: `4080842.cpp` allocates a 43 MB VLA on the stack at max `n = 350`.

## Dual LOC Summary

Across all 10 problems, the corpus C++ totals **1,718 raw LOC** and **621 library-stripped core LOC** (excluding I/O templates, debug macros, `Mint`, `Comb`, `ST`, and sieve setup). The ideal CP Lang ports total **378 LOC** (**78.0% raw reduction**, **39.1% library-stripped reduction**), and the runnable `today/` ports total **499 LOC** (**71.0% raw reduction**, **19.6% library-stripped reduction**). The 121-line gap between ideal and `today/` directly measures the language and stdlib friction cataloged in `research/friction-backlog-batch-23-math-games.md`.
