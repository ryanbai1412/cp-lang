# ecnerwala Corpus Wave #9 (`159 -> 179`)

Wave #9 continues in decreasing recency order across `https://github.com/ecnerwala/contest-programming` after `CF/2138/D` (rebasing onto `929c022`), adding **20 new verified ports** (`159 -> 179` total, including 4 more interactive ports with live interactors, 4D interval DP, Mo's algorithm, Tarjan BCC, and DAG game theory).

## Summary

- **New ports**: 20 (`159 -> 179` total in `proto/ports/ecnerwala/`): `cf2137a`, `cf2137b`, `cf2137c`, `cf2137d`, `cf2137e`, `cf2137f`, `cf2137g`, `cf2135a`, `cf2135b` (interactive), `cf2135c`, `cf2135d1` (interactive), `cf2135e1`, `cf2129a`, `cf2129b`, `cf2129c1` (interactive), `cf2129c3` (interactive), `cf2129d`, `cf2129e`, `cf2127a`, `cf2127b`.
- **Gate verification**: **20/20 passed** `proto/ports/ecnerwala/gate.sh NAME 40` (samples + 40 seeded random cases, ASan/UBSan C++17 + `cpl run` interpreter + differential match against ecnerwala's original C++; live interactive pipes + `cpl run` transcript replay for the 4 interactive ports `cf2135b`, `cf2135d1`, `cf2129c1`, `cf2129c3`).
- **Friction lint**: **0 findings** across all 20 ports (`0 wa_trap`, `0 real_gap`, `0 stale_port`).
- **Emitted C++17 size**: max **12.6 KB** (`cf2135c`, 12,899 B), well under the 64 KB (65,536 B) judge cap.
- **LOC**: **549 CP LOC** vs **1,531 raw C++ LOC** (**-64.1%** raw) and **946 library-stripped C++ LOC** (**-42.0%** stripped).

## Ports

| Port | Upstream | Category / Features Exercised | CP LOC | Orig LOC (Raw / Stripped) | Emitted C++17 | Gate (`40`) |
| --- | --- | --- | ---: | ---: | ---: | --- |
| `cf2137a` | `CF/2137/A` | Bit shift | 3 | 14 / 12 | 2,856 B (2.8 KB) | PASS (1 + 40) |
| `cf2137b` | `CF/2137/B` | Comprehension print | 3 | 16 / 14 | 2,866 B (2.8 KB) | PASS (1 + 40) |
| `cf2137c` | `CF/2137/C` | Parity / divisibility | 9 | 23 / 21 | 3,416 B (3.3 KB) | PASS (1 + 40) |
| `cf2137d` | `CF/2137/D` | Constructive + `all(...)` | 19 | 35 / 33 | 3,993 B (3.9 KB) | PASS (1 + 40) |
| `cf2137e` | `CF/2137/E` | MEX fixed point + `sum(...)` | 16 | 47 / 45 | 4,527 B (4.4 KB) | PASS (1 + 40) |
| `cf2137f` | `CF/2137/F` | Monotonic stack + `bsearch_last` | 21 | 44 / 42 | 8,608 B (8.4 KB) | PASS (1 + 40) |
| `cf2137g` | `CF/2137/G` | DAG game + `[0..n, 0..4]Int` + `Deque` | 50 | 96 / 94 | 11,458 B (11.2 KB) | PASS (1 + 40) |
| `cf2135a` | `CF/2135/A` | Reverse DP on occurrence lists | 14 | 28 / 26 | 4,206 B (4.1 KB) | PASS (1 + 40) |
| `cf2135b` | `CF/2135/B` | Interactive (`flush()` + Manhattan interactor) | 26 | 47 / 45 | 4,709 B (4.6 KB) | PASS (1 + 40) |
| `cf2135c` | `CF/2135/C` | Tarjan BCC DFS + `Mod<998244353>` | 46 | 187 / 68 | 12,899 B (12.6 KB) | PASS (1 + 40) |
| `cf2135d1` | `CF/2135/D1` | Interactive + `ceil_div` word-wrap interactor | 29 | 71 / 69 | 7,041 B (6.9 KB) | PASS (1 + 40) |
| `cf2135e1` | `CF/2135/E1` | `Mod<998244353>.inv()` + difference array | 48 | 194 / 70 | 12,280 B (12.0 KB) | PASS (1 + 40) |
| `cf2129a` | `CF/2129/A` | Optional tuple array (`?(Int, Int)`) | 15 | 27 / 25 | 4,129 B (4.0 KB) | PASS (1 + 40) |
| `cf2129b` | `CF/2129/B` | Inversion count + `count(...)` | 9 | 31 / 29 | 3,785 B (3.7 KB) | PASS (1 + 40) |
| `cf2129c1` | `CF/2129/C1` | Interactive bracket binary search | 30 | 73 / 71 | 5,843 B (5.7 KB) | PASS (1 + 40) |
| `cf2129c3` | `CF/2129/C3` | Interactive superincreasing bracket batches | 55 | 89 / 87 | 7,784 B (7.6 KB) | PASS (1 + 40) |
| `cf2129d` | `CF/2129/D` | 4D interval DP + `Mod.inv()` + `Int(bool)` | 39 | 332 / 52 | 12,753 B (12.5 KB) | PASS (1 + 40) |
| `cf2129e` | `CF/2129/E` | Mo's algorithm + sqrt-bucket order stats | 92 | 127 / 97 | 10,397 B (10.2 KB) | PASS (1 + 40) |
| `cf2127a` | `CF/2127/A` | Optional consistency check | 13 | 23 / 21 | 3,197 B (3.1 KB) | PASS (1 + 40) |
| `cf2127b` | `CF/2127/B` | 1D grid wall escape | 12 | 27 / 25 | 3,547 B (3.5 KB) | PASS (1 + 40) |
| **Total** | | | **549** | **1,531 / 946** | **max 12.6 KB** | **20/20 PASS** |

## Friction & Compiler/Stdlib Findings (with Minimal Repros)

1. **`while let x = opt, cond:` (guarded `while let`) rejected (`E0103`)** (hit in `cf2137f`):
   While `if let` supports chaining `, cond`, `while let` rejects `,` after the binding:
   ```cp
   var stk: List<(Int, Int)> = []
   while let top = stk.last(), top.0 <= 5: // E0103: expected `:` after the `while` condition, found `,`
       stk.pop()
   ```
   Every monotonic stack / lazy-deletion heap loop currently has to write a 3-line `while let` + `if ...: break` pattern instead.

2. **Multi-dimensional (>2D) counted arrays (`tensor<T, 4>`)** (hit in `cf2129d`, and nested fill `[0..n][0..4]Int = 0` in `cf2137g`):
   2D grids `[0..n, 0..4]Int = 0` work cleanly, while `[0..n][0..4]Int = 0` fails with `E0302` (`[0..n][0..4]Int` vs `Int`), and 3D/4D DP tables (like ecnerwala's `tensor<num, 4> dp({N+1, N+1, V+1, V+1})`) require manual 1D stride flattening.

3. **Non-submissions skipped & Wave #10 backlog**:
   - Skipped non-submissions: `AtCoder/AGC078/A/A.cpp` (offline game-theory brute-force script printing to `std::cerr` with no `stdin`), and empty 8-line template stubs `CF/2138/C2`, `CF/2135/D2`, `CF/2129/C2`.
   - Deferred to Wave #10: `CF/2138/E1` & `E2` (416–425L float), `CF/2135/E2` (unconditional `V = 2e7` sieve at startup in C++), `CF/2135/F` (758L mod+segtree+rng), and `CF/2127/C` onward.
