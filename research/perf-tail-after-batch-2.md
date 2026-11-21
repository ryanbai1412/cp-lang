# Post-Batch #2 Max-N & `ecnerwala` Perf Tail Re-Measurement

Measured at `1d7d526` (`cpl` `0.2.2`, `g++ 11.4.0 -std=c++17 -O2`, 8-core Xeon, 5 alternating runs per binary, 10 ms above-floor cutoff). Includes Safety & Perf Tail Batch #2 (`67802d0`: `cpl::CpHash` for `std::vector<T>`, comprehension `std::move`), `f918ccf` (4 max-N port unblocks and 27 idiom cleanups), Syntax Batch #10 (`5b74c53`), and `ecnerwala` Wave #11 (`1d7d526`, 225 verified ports).

Note on harness path: the repository's max-N benchmark driver is `bench/corpus_p95_bench.py` (`bench/run_bench.py` does not exist in `bench/`). It was invoked with outputs redirected to `/tmp` so `bench/` stays untouched. For `ecnerwala`, a companion read-only driver in `/tmp` built all 225 ports with `cpl build` and timed non-interactive ports against their original C++20/23 sources (`g++ -std=c++20 -O2`, falling back to `zig c++ -std=c++23 -O2` for deducing-`this` originals, matching `proto/ports/ecnerwala/gate.sh`).

---

## 1. Headline Metrics (Full Max-N Suite)

| Metric | Baseline (`321560a`, `n=308`) | Post-Batch #1 (`52c5f5d`, `n=309`) | **Post-Batch #2 (`1d7d526`, `n=342`)** | Above-10ms Floor (`n=309`) | Target |
|---|---:|---:|---:|---:|---:|
| Measured ports | 308 (290 >=10ms) | 309 (309 >=10ms) | **342** (+33) | **309** | >= 200 |
| Runtime median | 0.986x | 0.977x | **0.978x** | **0.953x** | ~1.0x |
| Runtime p90 | 1.976x | 2.136x | **2.054x** | **2.235x** | - |
| Runtime p95 | 3.049x | 2.846x | **2.997x** | **3.207x** | < 1.1x |
| Runtime p99 | 7.987x | 5.866x | **7.704x** | **7.704x** | - |
| Runtime max | 36.796x | 233.419x | **159.441x** | **159.441x** | <= 1.5x-2.0x |
| RSS median | - | 1.051x | **1.034x** | **1.170x** | ~1.0x |
| RSS p95 | 3.504x | 3.766x | **3.527x** | **3.961x** | - |
| RSS max | 37.420x | 37.420x | **37.419x** | **37.419x** | - |
| Max emitted C++17 | 30,223 B (29.5 KB) | 32,499 B (31.7 KB) | **33,839 B (33.0 KB)** (`cco23p2`) | 0 / 654 >= 64 KB | < 64 KB |
| Output agreement | 308/308 (100%) | 309/309 (100%) | **342/342 (100%)** | 6 unmeasured `outputs differ` | 100% |

**Matched-set comparison vs baseline `321560a` (`n = 307` common ports):**
- Median `0.985x -> 0.982x`, p90 `1.976x -> 2.035x`, p95 **`3.049x -> 2.978x`**, p99 **`7.987x -> 6.040x`**, max **`36.796x -> 10.304x`** (**3.57x tail reduction**).
- The jump in all-port max (`159.441x`) and p99 (`7.704x`) compared to Batch #1's matched set is driven by two newly measured ports (`nccc8s3` at `159.44x` and `nccc9s4` at `87.47x`) plus `ioi14p6io` (`10.30x`), all three of which have simple, verified compiler root causes below.

**Output agreement and compile hygiene:**
- **342 / 342 (100%)** measured ports match original `g++ -O2` output exactly on their maximum-size input.
- Only **6** unique unmeasured problems remain skipped for `outputs differ` (`ccc03s4`, `cco24p2`, `coci21c1p5`, `dmopc21c5p4`, `ioi05p2`, `noip22p4`), down from 9 in Batch #1 (three problems — `ccc20s1`, `nccc9s1`, `noip18p2` — have an older port skipped for `outputs differ` while another port of the same problem passes and is measured). Zero new output disagreements were introduced.
- **0 codegen compile failures** across all 654 emitted files. The 4 `g++` link skips (`egoi22p3`, `ioi14p1`, `ioi17p4`, `ioi22p5`) are IOI/EGOI grader-interface files with no `main` (`undefined reference to main`).

**Key wins since Batch #1 (`a6f6c9c`):**
- **`cco07p2` regression fixed (`67802d0`):** `cpl::CpHash` now hashes `std::vector<T>` and fixed arrays; `cco07p2` compiles cleanly and measures **`1.413x`** (`115.7 -> 163.5 ms`, faster than `1.575x` pre-Batch #1).
- **`dmopc20c6p3` dropped `48.52x` (`11,428.9 ms`) -> `0.742x` (`153.9 ms`)** (**74.3x faster**, now 26% faster than original C++), after `f918ccf` changed `out = out + " "` to `out += " "`. Note that `cpl-codegen` itself still lowers `s = s + x` to an `O(|s|)` copy rather than `s += x` — worth fixing in codegen so no user hits the quadratic trap.
- **33 additional ports now measured (`309 -> 342`):** `f918ccf` and `e9cb505` unblocked `noi18p4` (`3.30x`), `ioi13p3io` (`0.58x`, RSS `35.21x`), `ioi08p3`, `sdoi17r1p6`, and legacy ports. Prior Batch #1 wins held steady: `dmopc21c8p5` `2.27x` (was `36.80x`), `shoi16p2` `3.30x` (was `21.33x`), `ioi07p3` `1.99x` (was `3.49x`), `dmopc21c2p2` `1.25x` (was `2.05x`).

---

## 2. Top 5 Remaining Tail Outliers (Exact Generated C++ Root Causes for Batch #3)

The top 3 outliers alone explain why max is `159.44x` instead of `7.70x`. Two of them (`#2 nccc9s4` and `#3 ioi14p6io`) share the **exact same single-line codegen defect**, confirmed below by direct A/B edits to the emitted C++.

### #1. `nccc8s3` — `159.441x` (`66.0 -> 10,519.9 ms`, RSS `0.60x`, `8,531 B`)
Port: `proto/ports/scout_batch_6/today/nccc8s3.cp`.

Source line 10 validates the input precondition with an `O(N * K)` nested quantifier before running the `O(N)` greedy stack algorithm:
```cp
assert all(any(a[i] == v for i in I) for v in V)
```
Emitted release C++17 leaves the assertion active unconditionally:
```cpp
if (!([&]() -> bool {
    bool t2 = true;
    for (int v = 0; v < V_len; v++) {
        if (!cpl::has(a, v)) {
            t2 = false;
            break;
        }
    }
    return t2;
}())) cpl::fail("assertion failed: all(v in a for v in V) (line 10)");
```
On the `n = 2e5, k = 2e5` derived input, `cpl::has(a, v)` performs a linear scan per `v`, burning ~10.5 seconds before line 11 executes.

**Batch #3 fix:** Gate user `assert` checks under `#ifdef LOCAL` in release `cpl build` (matching C++ `assert()` semantics and preserving full checking under `cpl run` and `-DLOCAL` debug builds).

### #2. `nccc9s4` — `87.473x` (`20.9 -> 1,827.3 ms`, RSS `1.52x`, `18,753 B`)
Port: `proto/ports/scout_batch_41/nccc9s4.cp`.

Coordinate compression looks up each element via `vals.at(a[i])` (`Compressed.at` in `stdlib/prelude.cp`, which calls `bsearch_first` on `self.vals`). Inside the emitted `Compressed_..._at` function, codegen copies `self.vals` by value into a temporary `std::vector<ll>` before binary-searching:
```cpp
int Compressed_D1_D_1_Int_2_at(const Compressed_D1_D_1_Int_2& self, ll val) {
    std::vector<ll> t0 = self.vals;
    ...
```
That copies the entire `10^5`-element vector on every lookup — `O(N^2)` bytes copied (`~80 GB` of `memcpy` traffic) for what should be an `O(log N)` read.

**Verified A/B proof:** Editing that single emitted line from `std::vector<ll> t0 = self.vals;` to `const std::vector<ll>& t0 = self.vals;` and recompiling with `g++ -std=c++17 -O2` dropped wall time on the exact benchmark input from **`1.75 s` to `0.01 s`** (**175x speedup**, identical output MD5 `29040db85ea8429ee54f9cbb3dceb556`).

### #3. `ioi14p6io` — `10.304x` (`1,720.7 -> 17,730.5 ms`, RSS `1.42x`, `12,890 B`)
Port: `proto/ports/scout_batch_52/today/ioi14p6io.cp`.

**Exact same codegen bug as `#2`.** Inside the hot divide-and-conquer DP query helper `topk_query(i, k)`, `let sp = g_sorted_pref[b]` emits a by-value `std::vector<ll>` deep copy instead of a const reference:
```cpp
ll topk_query(ll i, ll k) {
    ...
    std::vector<ll> sp = cpl::at(g_sorted_pref, b);
    for (const std::tuple<ll, ll>& pair : cpl::at(g_gaps, b)) {
```
Every DP transition copies a `~10^3`-element prefix vector, spending ~16 seconds in allocator and `memcpy` overhead.

**Verified A/B proof:** Editing that single emitted line to `const std::vector<ll>& sp = cpl::at(g_sorted_pref, b);` cut wall time on the exact `hand.in` benchmark input from **`17.73 s` (`10.30x`) to `1.87 s` (`1.10x`)** (**9.5x speedup**, identical output MD5 `fd8cf4d0854cc486fbea62f96fb6692a`, vs `1.70 s` original).

**Batch #3 fix for `#2` and `#3` together:** In `cpl-codegen`, emit `const T&` for non-mutated `let` bindings and compiler temporaries (`t0`) initialized from an lvalue container place (`self.vals`, `cpl::at(vec, idx)`, struct field, or local/global variable) whenever the source container is not mutated while the binding is live (or trivially when the enclosing scope never writes to that container). This single pass eliminates both `#2` (`87.47x -> ~1x`) and `#3` (`10.30x -> 1.10x`) and prevents the same cliff in any port that writes `let row = adj[u]`.

### #4. `cco17p3` — `7.704x` (`382.9 -> 2,950.4 ms`, RSS `9.99x` / `232.5 -> 2,322.5 MB`, `10,536 B`)
Port: `proto/ports/scout_batch_7/cco17p3.cp`.

Two compounding codegen overheads hit both runtime (`7.70x`) and peak RSS (`9.99x`, `2.32 GB`):
1. `Node.kid: [0..2]?Int` emits a heap-allocated `std::vector<std::optional<ll>>` inside every trie node (`fresh()` pushes `Node{std::vector<std::optional<ll>>{...}, 0, std::optional<ll>()}`), paying a 24-byte vector header + 32-byte heap block + malloc header per node instead of `int[2]` (`8 B`).
2. The generator `fn along(root: Int, x: Int) -> yields Int` lowers to `std::vector<ll> along(ll root, ll x)` instead of a push callback, allocating a fresh `std::vector<ll>` for every outer and inner trie walk:
```cpp
std::vector<ll> t8 = along(xs, r);
for (ll col : t8) {
    if (cpl::at(nodes, col).ys.has_value()) {
        std::vector<ll> t9 = along(ys, c);
        ...
```

**Batch #3 fix:** Lower compile-time-constant `[0..K]T` struct fields (and small fixed-size list literals like `[INF, INF]`) to `std::array<T, K>`, niche-pack `?Int`/`?Index` into a single sentinel word instead of 16-byte `std::optional<ll>`, and inline/push-lower `for x in gen(...)` yields directly into a callback loop with zero heap allocation.

### #5. `noi04p3` — `6.117x` (`110.7 -> 677.3 ms`, RSS `1.08x`, `23,041 B`)
Port: `proto/ports/blind_critic_5/noi04p3.cp`.

Inside `for bni in 0..(1 << n)` (1,024 iterations), three scratch arrays are reallocated and populated via `.push(...)` on every mask iteration:
```cpp
std::vector<std::optional<ll>> dp = std::vector<std::optional<ll>>();
std::vector<ll> nxt_dir = std::vector<ll>();
std::vector<ll> nxt_enc = std::vector<ll>();
```
Commit `65686e3` only hoists non-escaping lists declared at the top level of a function body, not lists declared inside a loop body. In addition, `dp` uses 16-byte `std::optional<ll>` (2x cache footprint) and pays `cpl::at` bounds checks and `cpl::shamt` shift checks on every inner `m * states` transition.

**Batch #3 fix:** Extend `65686e3` buffer reuse to non-escaping loop-local lists (hoist declaration before the loop, `.clear()` per iteration) and niche-pack `?Int`.

---

## 3. Next 5 Runtime Outliers (#6-#10) & Top RSS Offenders

For full Batch #3 visibility beyond the top 5:

6. **`yac6p6`** (`6.040x`, `76.0 -> 459.0 ms`, RSS `1.56x`): 3D `cpl::Grid<std::optional<ll>, 3>` (16 B/cell vs 4 B `int32` in original) plus unhoisted local `Grid` allocations in `small_dp`.
7. **`cco05p6`** (`4.995x`, `26.6 -> 133.0 ms`, RSS `0.98x`): `10^8`-iteration tight stack loop pays `cpl::at(stack, i)` bounds check on every step plus 32-byte tuple value copies.
8. **`sac22cc2p4`** (`4.372x`, `162.9 -> 712.0 ms`, RSS `2.47x`): `for (dr, dc) in [(0, 1), (0, -1), (1, 0), (-1, 0)]` allocates a fresh heap `std::vector` on each of `10^6` Dijkstra pops + 16-byte `std::optional<ll>` grid.
9. **`ioi09p6`** (`4.120x`, `32.1 -> 132.3 ms`, RSS `1.52x`): `around(r, c)` returns a fresh heap `std::vector` on every BFS pop (~19M allocations) + fresh `800x800` `Grid` per binary-search step.
10. **`ioi09p4`** (`3.920x`, `74.9 -> 293.4 ms`, RSS `2.08x`): flat `@memo` splits `50 MB` value table and `6.25 MB` `_seen` table across two cache lines plus per-call `__memo_ready` check and `cpl::have`.

**Top 5 RSS outliers:**
- `dmopc21c2p2`: **`37.42x`** (`3.5 -> 131.0 MB`, `1.25x` time) — `read_all()` holds the entire input buffer in memory for the program's lifetime instead of streaming tokens.
- `ioi13p3io`: **`35.21x`** (`58.0 -> 2,042.2 MB`, `0.58x` time) — 64-bit `ll` Monge matrices in segment tree vs `int32` in original.
- `cco17p3`: **`9.99x`** (`232.5 -> 2,322.5 MB`, `7.70x` time) — heap `std::vector<std::optional<ll>>` per trie node (`#4` above).
- `cco22p4`: **`9.57x`** (`3.5 -> 33.5 MB`, `2.74x` time) — `read_all()` / 64-bit `Int` table width.
- `ccc14s4`: **`8.90x`** (`3.8 -> 33.4 MB`, `1.17x` time) — 64-bit 2D difference grid vs `int32`.

---

## 4. `ecnerwala` Corpus Benchmark (All 225 Verified Ports)

All **225** ports in `proto/ports/ecnerwala/` were compiled with `cpl build` (`0.2.2`) to measure emitted C++17 size, and all **212** non-interactive ports were tested against their accepted `ecnerwala` C++20/23 originals (13 ports are interactive and verified via live pipes in `gate.sh`).

### Emitted C++17 Size (`225 / 225` built, `0 >= 64 KB`)
- **Pass rate:** **225 / 225 (100%)** under the 64 KB (`65,536 B`) limit.
- **Distribution:** min **`2,777 B` (`2.7 KB`, `cf2178a`)**, median **`7,453 B` (`7.3 KB`)**, p95 **`19,514 B` (`19.1 KB`)**, max **`65,469 B` (`63.9 KB`, `cf2255f`, 67 B headroom)**.

### Runtime, RSS & Output Agreement vs Original C++ (`-O2`)

Because `gen/<name>.py` generators are sized for fast differential fuzzing, we measured both:
1. **Full non-interactive sweep (`211 / 212` measured; 3 alternating runs):** every port whose generator produces standard single-test or multitest input (1 port, `cf2255c`, is a Run-Twice problem whose generator starts with `"first"` rather than `T`, so the multitest stacker skipped it; it builds at `8,451 B` and passes `gate.sh 40`). **211 / 211 (100%, 0 mismatches)** exact output agreement.
2. **Heavy compute-bound subset (`n = 19` above 10 ms floor; 5 alternating runs):** scaled `sz` and stacked `T` batches so each binary runs compute-bound (`10 ms - 450 ms`), spanning NTT/poly, sparse tables, treaps, dual segtrees, centroid decomposition, DP, and ordered maps. **19 / 19 (100%)** output agreement.

| `ecnerwala` Slice | Ports | Time Median | Time p90 | Time p95 | Time p99 | Time Max | RSS Median | RSS p95 | RSS Max |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Full non-interactive sweep (3-run) | 211 | **0.982x** | **1.146x** | **1.231x** | **1.624x** | **3.157x** | **0.893x** | **1.625x** | **4.510x** |
| **Heavy compute-bound >= 10 ms (5-run)** | **19** | **1.032x** | **1.679x** | **4.872x** | **4.872x** | **4.872x** | **1.049x** | **4.510x** | **4.510x** |

**Key takeaway on `ecnerwala`:** **18 of 19** heavy compute-bound `ecnerwala` ports run within **`<= 1.68x`** of `ecnerwala`'s hand-tuned C++20/23, with median **`1.032x`** and multiple heavy ports faster than the original (`ucup5s06a` `0.94x`, `ucup5s05f` `0.81x`, `cf2101e` `0.96x`, `cf2096f` `0.57x`, `cf2262e` `0.57x`, `cf2089c1` `0.52x`, `cf2127f` `0.17x`).

**Top 10 slowest heavy `ecnerwala` ports (5-run median, >= 10 ms floor):**
1. **`cf2268c`** — **`4.87x`** (`9.0 -> 43.8 ms`, RSS `1.00x`, `14,139 B`, `g++` orig): inner loop emits `std::tuple<std::vector<ll>, std::vector<ll>> cur = {std::vector<ll>{INF, INF}, std::vector<ll>{-INF, -INF}}` (two 2-element heap allocations on every element across 18 bit iterations) plus `SparseTable` `std::function` dispatch and `bounds = cpl::sorted(bounds)` copy. Exact same fixed-small-array codegen pattern as `cco17p3` (`#4`) and `sac22cc2p4` (`#8`)!
2. **`ucup5s03f`** — **`1.68x`** (`33.4 -> 56.1 ms`, RSS `0.96x`, `16,180 B`, `g++` orig).
3. **`cf2261h`** — **`1.50x`** (`287.2 -> 431.6 ms`, RSS **`4.51x`**, `11,639 B`, `zig-c++` orig).
4. **`cf2268e`** — **`1.39x`** (`25.7 -> 35.7 ms`, RSS `0.97x`, `39,819 B`, `g++` orig).
5. **`cf2124f2`** — **`1.38x`** (`29.6 -> 40.9 ms`, RSS `2.15x`, `12,833 B`, `g++` orig).
6. **`ucup5s01a`** — **`1.38x`** (`197.7 -> 271.9 ms`, RSS `1.45x`, `15,305 B`, `g++` orig).
7. **`cf2115f1`** — **`1.24x`** (`18.7 -> 23.2 ms`, RSS `1.90x`, `19,763 B`, `zig-c++` orig — implicit treap + mergeable segtree).
8. **`cf2089c2`** — **`1.24x`** (`49.1 -> 60.8 ms`, RSS `0.93x`, `16,210 B`, `g++` orig).
9. **`ucup5s03a`** — **`1.16x`** (`52.7 -> 60.9 ms`, RSS `1.05x`, `42,591 B`, `g++` orig).
10. **`cf2101f`** — **`1.03x`** (`35.4 -> 36.6 ms`, RSS `1.62x`, `17,385 B`, `zig-c++` orig).

---

## 5. Ranked Priority Plan for Perf Tail Batch #3

Ordered by verified impact per compiler change:

1. **`const T&` for non-mutated lvalue container `let` bindings and temporaries (`t0`):** Directly proven to fix `#2 nccc9s4` (`87.47x -> ~1x`, **175x speedup**) and `#3 ioi14p6io` (`10.30x -> 1.10x`, **9.5x speedup**), and eliminates silent `O(N)` copies on `Compressed.at` and `let row = g[u]` corpus-wide.
2. **Gate user `assert` under `#ifdef LOCAL` in release `cpl build`:** Directly fixes `#1 nccc8s3` (`159.44x -> ~1x`). Together with `#1` above, max drops from **`159.44x` to `7.70x`** and p99 from **`7.70x` to `~5.0x`**.
3. **Stack `std::array<T, K>` for constant-size `[0..K]T` fields and small list literals in `for x in [...]` / structs:** Fixes `#4 cco17p3` RSS & time, `#8 sac22cc2p4`, and `ecnerwala` `#1 cf2268c` (`4.87x`).
4. **Push-callback lowering for `yields` generators (`for x in gen(...)`):** Eliminates per-call `std::vector` heap allocation in `#4 cco17p3` (`along()`) and `#9 ioi09p6` (`around()`).
5. **Hoist loop-local non-escaping lists (and `Grid`s) before loops with `.clear()` reuse:** Extends `65686e3` to fix `#5 noi04p3` (`6.12x`) and `#6 yac6p6` (`6.04x`).
6. **Niche-packed `?Int` / `?Index` (`i64::MIN` sentinel instead of 16-byte `std::optional<ll>`) + `s = s + x` -> `s += x` rewrite:** Halves DP/trie memory bandwidth across `#4 cco17p3`, `#5 noi04p3`, `#6 yac6p6`, `#8 sac22cc2p4`, `#15 cco21p5`, and prevents any future `dmopc20c6p3`-style quadratic string concat.
7. **Streaming stdin reader instead of `read_all()` buffer retention + `SegTree`/`SparseTable` templated ops instead of `std::function`:** Fixes the `37.42x` RSS outlier (`dmopc21c2p2`, `cco22p4`) and cuts indirect-call overhead in `btoi17p2`, `cco21p5`, and `cf2268c`.
