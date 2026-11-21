# Max-N Corpus Performance Re-Measurement (Post-Perf Tail Batch #1)

Measured on `main` at `52c5f5d` (incorporating Perf Tail Batch #1: `63d3ab6` salted hash `Map`/`Set` + single-lookup `if k in m: m[k]`, `a928d81` `unchecked:` in `SegTree`/`LazySegTree`, and `65686e3` fn-local `List` buffer reuse) against the pre-Batch-1 baseline (`389b56f`).

Methodology: `python3 bench/corpus_p95_bench.py --runs 5 --jobs 8` comparing `cpl build` emitted C++17 against accepted original C++ (`g++ -std=c++17 -O2`) on identical max-N inputs, reporting 5-run median wall time, peak RSS, emitted byte size, and exact output agreement.

## Summary Metrics

**Full measured corpus (`n = 309`, up from `308`; all `309/309` above the 10 ms noise floor, up from `290/308`):**

- **Runtime ratio (CP Lang / original `g++ -O2`)**:
  - Median: **0.977x** (was `0.986x` all / `0.984x` above-floor)
  - p90: **2.136x** (was `1.976x` all / `2.050x` above-floor)
  - p95: **2.846x** (was `3.049x` all / `3.081x` above-floor)
  - p99: **5.866x** (was `7.987x` all / `9.983x` above-floor)
  - Max: **233.419x** (was `36.796x` — driven by two *newly measured* ports, `nccc8s3` at `233.42x` and `dmopc20c6p3` at `48.52x`, not a Batch #1 regression; see below)
  - Ports > 1.5x: **56 / 309** (was `56 / 308`); ports > 2.0x: **34 / 309** (was `30 / 308`)
- **Peak RSS ratio**:
  - Median: **1.051x** (was `1.060x`)
  - p90: **2.611x** (was `2.588x`)
  - p95: **3.766x** (was `3.504x` all / `3.766x` above-floor)
  - p99: **8.933x** (was `8.933x`)
  - Max: **37.420x** (`dmopc21c2p2`, 3.6 MB → 134.1 MB; unchanged)
- **Max emitted C++17 size**: **32,499 B (31.7 KB)** across `571` emitted files (`proto/ports/scout_batch_36/today/cco23p2.cp`, was `30,223 B` / `29.5 KB`), **0 / 571** over the 64 KB judge limit.
- **Output agreement**: **309 / 309 (100%)** measured ports produce byte-exact identical output to original C++ at max-N. Among skipped ports, the exact same **9** pre-existing unique problems (`ccc03s4`, `ccc20s1`, `cco24p2`, `coci21c1p5`, `dmopc21c5p4`, `ioi05p2`, `nccc9s1`, `noip18p2`, `noip22p4`) report `outputs differ`; **0 new output disagreements** were introduced by Batch #1.

**Matched-set comparison (`n = 294` ports measured in both `389b56f` and `52c5f5d`):**

- Runtime ratio: median `0.984x -> 0.978x`, p90 `1.917x -> 2.042x`, p95 **`2.965x -> 2.771x`** (`-0.194x`), p99 **`6.119x -> 4.591x`** (`-1.528x`), max **`36.796x -> 8.107x`** (**-28.69x / 4.54x drop in worst-case tail!**).
- RSS ratio: median `1.035x -> 1.034x`, p95 `3.489x -> 3.488x`, max `37.420x -> 37.420x` (unchanged, as expected since Batch #1 targeted compute/alloc overhead rather than 64-bit array width or `read_all()`).

## Biggest Batch #1 Wins

- **`dmopc21c8p5`**: **`36.80x -> 2.23x`** (`5,695.4 ms -> 365.6 ms`, **15.6x speedup**) — `63d3ab6` replaced `std::map<ll, ll>` with `std::unordered_map<ll, ll, cpl::CpHash>` and fused `if x in cache: return cache[x]` into a single `cache.find(x)` lookup.
- **`shoi16p2`**: **`21.33x -> 3.09x`** (`575.7 ms -> 65.9 ms`, **8.7x speedup**) — `65686e3` hoisted local `xs`, `ys`, and `codes` inside `square()` to `static std::vector<ll>` cleared per call.
- **`ioi07p3`**: **`3.49x -> 1.91x`** (`412.1 ms -> 248.1 ms`, **1.66x speedup**) — `a928d81` dropped `cpl::at` bounds checks inside `LazySegTree`.
- **`dmopc21c4p2`**: **`4.89x -> 3.37x`** (`1,262.2 ms -> 859.2 ms`, **1.47x speedup**) — `63d3ab6` switched `kept: Set<Int>` to `std::unordered_set<ll, cpl::CpHash>`.
- **`btoi18p1`**: **`1.66x -> 0.60x`** (`136.9 ms -> 55.3 ms`, **2.48x speedup**) — moved well below parity.
- **Dropped below 1.5x**: `noi15p2` (`1.76x -> 1.31x`), `wossoly21s3` (`1.74x -> 1.27x`), `dmopc19c4p2` (`1.68x -> 1.34x`), `ioi00p5` (`1.56x -> 1.42x`).
- **Newly unblocked from timeout**: `dmopc20c6p3` previously timed out (> 15 s) and now completes in `11.43 s` (still dominated by an `O(N^2)` string concat bug; see #2 below).

## Batch #1 Codegen Regression Found (Fix First in Batch #2)

- **`cco07p2`** (`proto/ports/blind_critic_2/cco07p2.cp`, previously `1.575x` at `122.7 -> 193.2 ms`): now **fails `g++` compilation** in `<bits/hashtable.h>` because `63d3ab6` lowered `Set<List<Int>>` to `std::unordered_set<std::vector<long long>, cpl::CpHash>`, and `cpl::CpHash` has no `operator()` overload for `std::vector<T>` (or fixed arrays).
  - **Fix**: Add a generic container overload to `cpl::CpHash` hashing `v.size()` and combining `(*this)(elem)` over `v`, or restrict hash-table lowering to key types supported by `cpl::CpHash`.

## Top 15 Remaining Slowest Ports (Ratio > 1.5x)

Every root cause below was verified directly against the post-Batch-1 emitted C++17 (`52c5f5d`).

### 1. `nccc8s3` — **233.42x** (`47.1 ms -> 10,984.7 ms`, RSS `1.21x`, `7,650 B`)
- **Port**: `proto/ports/scout_batch_6/today/nccc8s3.cp` (newly measured; was skipped in `389b56f` for `where`).
- **Root cause**: Line 10 has a specification precondition `assert all(any(a[i] == v for i in I) for v in V)`. `cpl build` emits this `assert` unconditionally in release C++ as a nested `200,000 x 200,000` (`4e10`-iteration) loop before the `O(N)` greedy stack algorithm even starts.
- **Batch #2 lever**: Strip `assert` statements in release `cpl build` (keep active in `cpl run`, `cpl test`, and debug builds — matching C++ `NDEBUG` / Rust `debug_assert!`), or distinguish runtime `require`/`check` from debug `assert`.

### 2. `dmopc20c6p3` — **48.52x** (`235.6 ms -> 11,428.9 ms`, RSS `1.48x`, `14,928 B`)
- **Port**: `proto/ports/scout_batch_34/today/dmopc20c6p3.cp` (newly measured; previously timed out > 15 s).
- **Root cause**:
  1. The output loop runs `if u.to_idx > 0: out = out + " "` for `n = 500,000`, which emits `out = out + std::string(" ");` — copying the entire growing `std::string` on every iteration (`O(N^2)`, ~1.25e11 bytes copied).
  2. Secondary: `Graph.adj(u)` allocates and returns a fresh `std::vector<std::tuple<int, int>>` on every vertex visit instead of iterating a zero-copy CSR slice.
- **Batch #2 lever**: Lower `s = s + rhs` on `Str` (when `s` does not appear in `rhs`) to in-place `s += rhs`; lower `for arc in g.adj(u)` directly over the CSR offset range without materializing a `std::vector`.

### 3. `cco17p3` — **8.11x** (`388.1 ms -> 3,146.5 ms`, RSS **9.99x** / `2.38 GB`, `10,040 B`; was `7.99x`)
- **Port**: `proto/ports/scout_batch_7/cco17p3.cp`.
- **Root cause**:
  1. `struct Node` contains `kid: [0..2]?Int`, which lowers to `std::vector<std::optional<ll>>` — a 24 B vector header plus a separate 32 B heap allocation for *every single trie node* across millions of nodes (causing both `9.99x` RSS and severe pointer-chasing cache misses).
  2. Generator `along(root, x)` materializes a heap `std::vector<ll>` per call (`2e5 x log` times) rather than streaming via push callback.
  3. 14 `cpl::at` checks + `cpl::modp`/`divp` by 2 inside the trie walk.
- **Batch #2 lever**: Lower compile-time-constant-size `[0..K]T` (especially in struct fields, `K <= 64`) to inline `std::array<T, K>` (zero heap allocation per struct); inline single-consumer `for x in gen(...)` generator loops as push callbacks.

### 4. `noi04p3` — **5.87x** (`119.2 ms -> 699.2 ms`, RSS `1.08x`, `22,527 B`; was `5.98x`)
- **Port**: `proto/ports/blind_critic_5/noi04p3.cp`.
- **Root cause**:
  1. Inside the `1,024`-iteration `for bni in 0..num_masks` loop inside `solve()`, `dp`, `nxt_dir`, and `nxt_enc` are freshly allocated `std::vector`s filled with `20,200` `push_back`s each (`65686e3` only hoists fn-local lists, not loop-local lists inside `solve()`).
  2. `dp` is `std::vector<std::optional<ll>>` (16 B per cell vs 4 B `int32` in original).
  3. Innermost query loop executes `cpl::shamt(x1_2, 64)` and `cpl::at(queries, ind)` on every check.
- **Batch #2 lever**: Extend `65686e3` buffer reuse to non-escaping **loop-local** lists; niche-pack `?Int` to 8 B `cpl::OptInt` inside `vector`/`Grid`; elide `cpl::shamt` when shift amount is bounded in `0..64`.

### 5. `yac6p6` — **5.14x** (`76.0 ms -> 390.7 ms`, RSS `1.56x`, `20,754 B`; was `4.76x`)
- **Port**: `proto/ports/scout_batch_39/today/yac6p6.cp`.
- **Root cause**:
  1. `dp` and `dp2` are 3D `cpl::Grid<std::optional<ll>, 3>` (16 B per cell vs 4 B `int32` in original — 4x cache footprint plus `.has_value()` branching on every transition).
  2. `small_dp(...)` allocates a fresh 3D `cpl::Grid<std::optional<ll>, 3>` on every call (`65686e3` only reuses `std::vector`, not `cpl::Grid`).
- **Batch #2 lever**: Niche-pack `?Int` (`cpl::OptInt`, 8 B, `INT64_MIN` sentinel) inside `cpl::Grid` and `std::vector`; extend `65686e3` buffer reuse to non-escaping local `cpl::Grid`s.

### 6. `cco05p6` — **4.55x** (`30.7 ms -> 139.7 ms`, RSS `1.05x`, `5,762 B`; was `4.19x`)
- **Port**: `proto/ports/scout_batch_33/cco05p6.cp`.
- **Root cause**: Tight `O(N * Q) = 1e8` scan over `stack` runs `cpl::at(stack, i)` bounds check on every iteration of `for i in 0..stack.len`, copies 32-byte `std::tuple<ll, ll, ll, ll>` (`frame[stack[i]]`) by value (vs 16 B `int32` struct), writes `std::optional<ll> hit` on every match, and runs `2x cpl::at` per element in the shift loop.
- **Batch #2 lever**: Bounds-check elision for `for i in 0..v.len` indexing `v[i]` when `v` is not resized in the loop (eliminates `cpl::at` in both the `1e8` scan and the shift loop) + bind non-mutated `let f = frame[...]` by `const auto&`.

### 7. `sac22cc2p4` — **4.26x** (`169.4 ms -> 722.2 ms`, RSS `2.47x`, `13,576 B`; was `4.09x`)
- **Port**: `proto/ports/blind_critic_2/sac22cc2p4.cp`.
- **Root cause**:
  1. Inside the `10^6`-node Dijkstra `while let` loop, `for (dr, dc) in [(0, 1), (1, 0), (0, -1), (-1, 0)]` constructs a **fresh heap-allocated `std::vector<std::tuple<ll, ll>>` on every single pop** (~1,000,000 `malloc`/`free` calls).
  2. `dist` is `cpl::Grid<std::optional<ll>, 2>` (16 B vs 4 B `int32`), and `MinHeap` stores 24 B `tuple<ll, int, int>` (original also uses 0-1 BFS `deque`).
- **Batch #2 lever**: Lower constant/small list literals in `for x in [...]` directly to `static constexpr` / stack `std::array` (zero heap allocation inside loops!).

### 8. `ioi09p6` — **3.88x** (`39.1 ms -> 151.3 ms`, RSS `1.52x`, `14,348 B`; was `4.68x`)
- **Port**: `proto/ports/blind_critic_3/ioi09p6.cp`.
- **Root cause**:
  1. `around(r, c)` allocates and returns a fresh `std::vector<std::tuple<int, int>>` on every BFS pop across ~30 binary-search iterations (~19M `malloc`/`free` calls; `65686e3` cannot hoist `out` because `around` returns it).
  2. `check(t)` allocates a fresh `800 x 800` `cpl::Grid<char, 2> seen` on each binary-search step.
- **Batch #2 lever**: Inline small list-returning helpers (`around`) at `for x in fn(...)` call sites (or use push-generator lowering / small-buffer return), and hoist non-escaping local `cpl::Grid`s in functions like `check()`.

### 9. `ioi09p4` — **3.75x** (`85.2 ms -> 319.9 ms`, RSS `2.08x`, `17,199 B`; was `3.99x`)
- **Port**: `proto/ports/scout_batch_28/ioi09p4.cp`.
- **Root cause**: `@memo` 4D DP (`50^4 = 6.25M` states) hits flat mode (`__memo1_flat = true`), but splits storage into two separate vectors (`__memo1_vals` 50 MB `ll` + `__memo1_seen` 6.25 MB `char` = 2 cache-line loads per lookup vs original's single 25 MB `int32` table with `-1` sentinel), checks `if (__memo1_ready && __memo1_flat)` on every recursive call, and wraps inner cut sums in `std::optional<ll>` + `cpl::have(cpl::next_pos(r, Row_len))`.
- **Batch #2 lever**: Store `@memo` flat tables as a single niche-packed `cpl::OptInt` vector (single cache line, no `_seen` array), initialize once in `solve()` before first call, and elide `next_pos` bounds checks for `r in r1..r2` (`r < r2 < Row_len`).

### 10. `dmopc21c4p2` — **3.37x** (`255.3 ms -> 859.2 ms`, RSS `3.48x`, `10,279 B`; **down from `4.89x`**)
- **Port**: `proto/ports/scout_batch_44/today/dmopc21c4p2.cp`.
- **Root cause**: Batch #1 (`63d3ab6`) moved `kept` to `std::unordered_set`, dropping wall time from `1,262 ms` to `859 ms`. Remaining gap: `Sieve.factor(x)` and `divisors(x)` each allocate and return a new `std::vector` per input element (`2e6` heap allocations; unhoisted because returned), `Sieve.factor` runs `cpl::ediv`/`cpl::emod` in its inner loop despite `rest > 1` and `p >= 2`, and the port factors each number rather than using the original's simple multiple-marking boolean sieve.
- **Batch #2 lever**: Emit native `/` and `%` inside `Sieve.factor` once `rest >= 0 && p > 0` is proven; support push-callback / caller-scratch-buffer lowering for `for pe in sieve.factor(x)` and `for d in divisors(x)`.

### 11. `noi10p3` — **3.23x** (`385.0 ms -> 1,245.1 ms`, RSS `3.03x`, `15,504 B`; newly selected port replacing `scout_batch_27/noi10p3.cp` which ran at `0.33x`)
- **Port**: `proto/ports/scout_batch_31/today/noi10p3.cp` (now compiles after non-0-based array domains landed).
- **Root cause**:
  1. `dist` is 3D `cpl::Grid<std::optional<ll>, 3>` (16 B/cell x 2M states = 32 MB vs 8 MB `int32` -> 3.03x RSS and cache misses), and `MinHeap` stores 24 B `tuple<ll, tuple<int, int, int>>`.
  2. Algorithm difference: `scout_batch_31/today/noi10p3.cp` models an 8-direction turn-at-junction graph (`8x` states = 2M), whereas original `5672180.cpp` (and `scout_batch_27/noi10p3.cp`, `0.33x`) uses the standard 4-neighbor dual face graph (`250k` states).
- **Batch #2 lever**: Niche-pack `?Int` to 8 B `OptInt` in `cpl::Grid` (halves `dist` from 32 MB to 16 MB).

### 12. `btoi17p2` — **3.17x** (`76.1 ms -> 241.0 ms`, RSS **6.08x**, `18,632 B`; was `3.05x`)
- **Port**: `proto/ports/scout_batch_18/today/btoi17p2.cp`.
- **Root cause**:
  1. `SegTree<Gap, [Slot, Slot]?Int>` stores a heap-allocated `5x5` `cpl::Grid<std::optional<ll>, 2>` in every tree node (`100k+` separate heap buffers, driving `6.08x` RSS).
  2. `Monoid_Arr2_Opt_Int_2.op` is a **`std::function<Grid(Grid, Grid)>`** taking both matrices **by value** — every segment-tree combine pays an indirect call, two heap-buffer copies, and a third heap allocation.
- **Batch #2 lever**: Monomorphize `SegTree`/`LazySegTree` over concrete `Monoid`/`Action` callables (zero `std::function`, pass `const T&` instead of by-value copies!) + niche-pack `?Int` in `Grid` + small inline storage for tiny matrices.

### 13. `shoi16p2` — **3.09x** (`21.3 ms -> 65.9 ms`, RSS `0.89x`, `9,393 B`; **down from `21.33x`**)
- **Port**: `proto/ports/scout_batch_45/today/shoi16p2.cp`.
- **Root cause**: `65686e3` hoisted `xs`, `ys`, and `codes` inside `square()` to `static std::vector<ll>` (`575.7 ms -> 65.9 ms`, **8.7x speedup**). What remains across ~6M calls to `square()`: 12 `cpl::at` bounds checks on the 4-element `xs`/`ys`/`codes` vectors, `push_back` capacity checks + `std::sort(codes.begin(), codes.end())`, and `cpl::divp`/`modp` by 2 on signed `ex`/`ey`.
- **Batch #2 lever**: Lower fixed-push-count small lists (4 elements) to stack `std::array<ll, 4>` with provable bounds elision (removes `cpl::at` and `push_back` overhead).

### 14. `coci21c1p2` — **3.05x** (`28.6 ms -> 87.2 ms`, RSS **6.85x** / `148.1 MB`, `6,002 B`; was `3.08x`)
- **Port**: `proto/ports/scout_batch_23/today/coci21c1p2.cp`.
- **Root cause**: `memo` is `std::vector<ll>` of `351 x 351 x 151 = 18.6M` entries (`148.8 MB`) vs original's `char dp[351][351][151]` (`18.6 MB`) — **8x memory traffic and L3 thrashing** — plus 8 `cpl::at` bounds checks per recursive `win(l, r, my_red)` call.
- **Batch #2 lever**: Niche-packed `?Bool` (`cpl::OptBool`, 1 byte) already exists for 3-state DP tables (shrinking `memo` 8x when ports use `[...]?Bool` instead of `-1/0/1` `List<Int>`); add range-check elision on proven-in-bounds array accesses.

### 15. `cco21p5` — **2.85x** (`84.4 ms -> 240.7 ms`, RSS `2.34x`, `19,198 B`; **down from `3.17x`**)
- **Port**: `proto/ports/scout_batch_30/cco21p5.cp`.
- **Root cause**: `a928d81` removed `cpl::at` inside `LazySegTree` (`291.7 -> 240.7 ms`, `-17%`). Remaining gap:
  1. `Action_Opt_Int_Int_2` / `Monoid_Opt_Int_3` store **3 `std::function` objects** (`compose`, `apply`, `op`) invoked indirectly at every tree node during `push`/`pull`/`fold`/`apply`.
  2. Tree elements are `std::optional<ll>` (16 B vs 8 B sentinel `ll`, doubling tree cache footprint).
- **Batch #2 lever**: Monomorphize `LazySegTree`/`SegTree` `Monoid`/`Action` callables (inline `compose`/`apply`/`op` directly under `g++ -O2`) + niche-pack `?Int` to 8 B `cpl::OptInt` inside `std::vector`.

## Recommended Perf Tail Batch #2 Priority Order

Ranked by direct impact on corpus p95 (`2.846x -> < 1.5x`), worst-case max (`233.4x -> < 3x`), and RSS p95 (`3.766x`):

1. **Fix `cpl::CpHash` for `std::vector<T>` / fixed arrays** — unblocks the one Batch #1 codegen regression (`cco07p2`).
2. **Release `assert` elision + `s = s + rhs` -> `s += rhs` on `Str`** — drops worst-case max from `233.42x` (`nccc8s3`) and `48.52x` (`dmopc20c6p3`) straight to ~1x.
3. **Niche-pack `?Int` (`cpl::OptInt`, 8 B) inside `cpl::Grid` and `std::vector` + single-array `@memo`** — halves memory/bandwidth on `noi04p3`, `yac6p6`, `sac22cc2p4`, `ioi09p4`, `noi10p3`, `btoi17p2`, `cco21p5`, and `dmopc21c8p5` (8 of the top 15!).
4. **Monomorphize `SegTree`/`LazySegTree` `Monoid`/`Action` callables (no `std::function`, pass `const T&`)** — directly attacks `btoi17p2` (`3.17x`), `cco21p5` (`2.85x`), `ioi07p3` (`1.91x`), and every segment-tree port.
5. **Stack/inline lowering for constant/small arrays and loop-literal lists**:
   - `for x in [(0,1), ...]` -> `static constexpr` / stack `std::array` (fixes `sac22cc2p4` `4.26x` 1M `malloc`s).
   - Small compile-time-constant `[0..K]T` struct fields -> inline `std::array<T, K>` (fixes `cco17p3` `8.11x` time & `9.99x` RSS).
   - Extend `65686e3` buffer reuse to loop-local `List`s and fn-local `cpl::Grid`s (fixes `noi04p3`, `yac6p6`, `ioi09p6`).
6. **Zero-copy `for x in g.adj(u)` / generator push-inlining + `for i in 0..v.len` bounds-check elision** — removes heap-alloc-per-call in `dmopc20c6p3`, `cco17p3`, `ioi09p6`, `dmopc21c4p2` and tight-loop `cpl::at` in `cco05p6`, `shoi16p2`, `coci21c1p2`.
7. **Streaming stdin (replace `cpl::read_all()` buffer retention) for RSS p95** — fixes the `37.42x` RSS outlier (`dmopc21c2p2`) and `5-9x` RSS on `cco22p4`, `ccc14s4`, `ccc19s2`, `ioi09p3`, `dmopc21c5p1`.
