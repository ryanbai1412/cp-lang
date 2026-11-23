# Scout Batch 19: Interactive, Multi-Answer Constructive (`checker`), and Unusual I/O

Stage-1 scouting batch targeting 10 previously unported problems from `corpus/` (zero overlap with existing ports, `scout_batch_1`–`scout_batch_18`, `blind_critic_1`, or `blind_critic_2`) across three under-exercised contest surfaces: interactive protocols (stdio and IOI graders), multi-answer constructive outputs requiring custom checkers, and unusual I/O formats (sentinel-terminated multi-test streams, mixed cell alphabets, recursive grammar strings, and verbatim line-oriented input).

Full ergonomic friction analysis and unified core-syntax proposals live in `research/archive/frictions/friction-backlog-batch-19-interactive-checker.md`.

## Layout

- Top-level `*.cp` (10 files): idiomatic ideal CP Lang sketches with co-located `interactor:`, `checker:`, `gen`, `assume`, and `spec` blocks.
- `today/*.cp` (9 files): buildable `proto/cpc` ports verified with `g++ -std=c++17 -O2 -fsanitize=address,undefined` against the original C++ submissions and independent checkers/interactors. (`noimps22p1` has no `today/` port because `proto/cpc` lacks `read_line()` and mutable/sliceable `Str` operations.)
- `verify.py`: runs all 9 `today/` builds (on temporary copies so tracked `.cp` files are never stamped) and executes every interactive, constructive-checker, and differential test under ASan+UBSan.

Run verification from repo root:

```bash
python3 proto/ports/scout_batch_19/verify.py
```

## Problems Scouted

1. **`egoi23p3`** (EGOI '23 P3, *Find the Box*, 17 pts, `corpus/submissions/5678051.cpp`): stdio `?`/`!` interactive with a 2-query limit and hidden box state. Raw C++ 194 LOC / stripped 166 LOC -> ideal 49 LOC solution + 12 LOC `interactor:`; `today/` 94 LOC (verified on 30 random grids).
2. **`ioi07p1`** (IOI '07 P1, *Aliens*, 12 pts, `corpus/submissions/5111683.cpp`): stdio `examine`/`solution` interactive over `2e9` coordinates with a 300-query limit and coupled public/hidden 5x5 checkerboard geometry. Raw C++ 124 LOC / stripped 96 LOC -> ideal 28 LOC solution + 20 LOC `interactor:`; `today/` 35 LOC (verified on 30 random boards).
3. **`ioi16p5`** (IOI '16 P5, *Unscrambling a Messy Bug*, 20 pts, `corpus/submissions/4692325.cpp`): two-phase IOI grader (`add_element` / `compile_set` / `check_element`) with separate write/read budgets and power-of-2 `n`. Raw C++ 105 LOC / stripped 82 LOC -> ideal 30 LOC solution + 30 LOC `interactor:`; `today/` 58 LOC stdio-adapted (verified on 30 cases up to `n=64`).
4. **`ioi14p3`** (IOI '14 P3, *Game*, 20 pts, `corpus/submissions/4683972.cpp`): reversed-control adaptive IOI grader where the judge calls `hasEdge(u, v)` `n*(n-1)/2` times. Raw C++ 56 LOC / stripped 35 LOC -> ideal 20 LOC solution + 18 LOC `interactor:`; `today/` 40 LOC stdio-adapted (verified on 20 adversarial edge orderings).
5. **`ccc23s3`** (CCC '23 S3, *Palindromic Poster*, 10 pts, `corpus/submissions/5350806.cpp`): multi-answer `N x M` char grid with `IMPOSSIBLE` sentinel and parity constraints. Raw C++ 180 LOC / stripped 154 LOC -> ideal 36 LOC solution + 13 LOC `checker:`; `today/` 70 LOC (verified exhaustively for all `n, m in 2..6` and all `r, c`).
6. **`dmopc21c5p4`** (DMOPC '21 C5 P4, *Permutations & Primes*, 17 pts, `corpus/submissions/4223525.cpp`): multi-test constructive permutation with prime adjacent differences, `-1` sentinel, and `sum(N) <= 1e6` budget. Raw C++ 264 LOC / stripped 161 LOC -> ideal 20 LOC solution + 12 LOC `checker:`; `today/` 48 LOC (verified on 106 cases up to `n=10000`).
7. **`coci21c6p3`** (COCI '21 Contest 6 #3, *Naboj*, 10 pts, `corpus/submissions/4530006.cpp`): multi-answer constructive directed-graph orientation sequence (`k` ops or `-1`) verified by an independent op-simulation checker. Raw C++ 94 LOC / stripped 70 LOC -> ideal 38 LOC solution + 15 LOC `checker:`; `today/` 54 LOC (verified on 80 random directed graphs).
8. **`ccc04s5`** (CCC '04 S5, *Super Plumber*, 10 pts, `corpus/submissions/4806811.cpp`): unusual `0 0` sentinel-terminated multi-test stream plus mixed `.` / `*` / `'1'..'9'` grid alphabet and reachability-preserving `gen`. Raw C++ 83 LOC / stripped 59 LOC -> ideal 25 LOC solution + 17 LOC `gen`; `today/` 40 LOC (verified on 40 random multi-test grids).
9. **`ceoi22p2`** (CEOI '22 P2, *Homework*, 15 pts, `corpus/submissions/4727898.cpp`): recursive `min(...)`/`max(...)`/`?` expression-tree string I/O with grammar-constructive `gen` and permutation `spec`. Raw C++ 100 LOC / stripped 76 LOC -> ideal 28 LOC solution + 28 LOC `gen`+`spec`; `today/` 30 LOC stack-safe evaluator (verified on 40 trees up to 1000 leaves).
10. **`noimps22p1`** (NOI '22 Multi-Provincial P1, *Preprocessor*, 10 pts, `corpus/submissions/4533698.cpp`): verbatim line-oriented I/O (`read_line()`, preserving spaces and empty lines, `#define`/`#undef` directives, cycle-free macro expansion). Raw C++ 106 LOC / stripped 82 LOC -> ideal 38 LOC solution + 24 LOC `gen`; not buildable in `proto/cpc` today.

## Totals

Across all 10 problems, raw C++ totals 1,306 LOC and library/template-stripped C++ totals 981 LOC. Ideal CP Lang solution code totals 312 LOC (76.1% reduction vs raw, 68.2% reduction vs stripped), plus 189 LOC of co-located `interactor:`, `checker:`, `gen`, and `spec` harnesses (averaging 18.9 LOC per problem). The 9 buildable `today/` ports total 469 LOC vs 1,200 raw / 899 stripped C++ on those 9 problems (60.9% raw / 47.8% stripped reduction).
