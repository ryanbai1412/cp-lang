# Stage-6 Max-N Judge & Constant-Factor Audit #1

Small random tests validate semantics, not judge survival. Earlier testing in [`ports-report.md`](ports-report.md) and [`tools/bench/`](../tools/bench/) benchmarked ports using each port's own random generator, which already flagged `ioi04p4` as ~26× slower while leaving adversarial worst-case shapes (max-N brooms/chains, full `M = 150` meet-in-the-middle, anti-hash collisions, 8 MB stack limits) untested. This audit closes that gap: it runs 30 diverse, buildable ports from `proto/ports/` (covering DP, graphs, trees, segment trees/sweeps, strings, sliding windows/hashing, and 2D grids) at their real contest limits, compiling both the CP Lang emitted C++17 (`proto/cpc`) and the original `corpus/` C++ with judge flags (`g++ -std=c++17 -O2`, no `-DLOCAL`).

Every single port (30/30) built, compiled cleanly, and matched the original C++ output token-for-token at max-N. Across all 30 ports, CP Lang's **median runtime ratio is 0.92×** (faster than original C++ on 17/30 ports, and >15% faster on 14/30), and every emitted C++17 file stayed under the **64 KB** judge limit (25.3 KB to 36.7 KB, median 28.2 KB).

However, the tail exposes severe constant-factor cliffs and memory bloat that would cause TLE, MLE, or SIGSEGV on real judges: **10 ports exceed 1.25× runtime** (peaking at **31.86×** on `ioi04p4` — 5.42 s vs 0.17 s), **16 ports exceed 1.50× peak RSS** (median **1.55×**, peaking at **5.77×** on `ioi04p4` and **5.60× / 198.0 MB** on `noi01p5`, an instant MLE on 64/128 MB judges), `CpHash` has **no runtime salt** (44.5× slowdown under splitmix64 inversion at `N = 50,000`), and recursive `@memo` on a 100k-node chain segfaults under Linux's default 8 MB stack.

## Measurement setup & reproducibility

All measurements were collected on an 8-core Intel Xeon VM with `g++ 11.4.0` (`-std=c++17 -O2`), taking the median wall-clock time and maximum peak RSS (`ru_maxrss` via `/usr/bin/time`) across 3 timed runs per binary (2 runs when a binary exceeds 1.5 s), with `ulimit -s unlimited` for the main 30-port sweep and `ulimit -s 8192` for the explicit stack-overflow probe. The reusable harness lives in [`bench/max_n_audit.py`](../bench/max_n_audit.py) and copies each `.cp` file into a temporary directory before running `proto/cp.py build` so tracked sources are never modified.

Reproduce the full audit and adversarial probes in one command:

```sh
python3 bench/max_n_audit.py --runs 3 --adversarial
```

## Summary of regressions (> 1.25× runtime or > 1.50× RSS)

Eighteen of the 30 ports tripped at least one regression threshold (10 runtime > 1.25×, 16 RSS > 1.50×, 8 both). The 12 ports that passed both thresholds cleanly (`btoi18p6`, `ioi11p4`, `apio09p3`, `ccc21s4`, `sac22cc3p4`, `ccc21s5`, `ccc20s3`, `noi14p4`, `coci20c6p3`, `ccc22s4`, `dmopc21c9p4`, plus `dps` on memory at 0.17× RSS) are omitted from the regression list below.

| Port | Max-N input | Runtime (ours vs orig) | Peak RSS (ours vs orig) | Primary cause |
| :--- | :--- | :--- | :--- | :--- |
| `ioi04p4` | 600×600, N=200 | 5423 vs 170 ms (**31.86×**) | 27.4 vs 4.8 MB (**5.77×**) | `@memo` over range types uses `unordered_map` |
| `noi01p5` | N=6, M=150 (3.375M) | 924 vs 200 ms (**4.62×**) | 198.0 vs 35.4 MB (**5.60×**) | `CpCounter` uses `unordered_map` + double lookup + 2× 3.375M `vector<ll>` |
| `dpp` | N=1e5 broom tree | 96.3 vs 24.6 ms (**3.92×**) | 36.7 vs 13.2 MB (**2.77×**) | `@memo(V, Bool)` uses `unordered_map`; eager `RootedTree::sp` |
| `cco18p1` | N=1000 | 28.4 vs 8.9 ms (**3.20×**) | 11.0 vs 7.2 MB (**1.52×**) | 16B `Opt<ll>` memo table + top-down recursion + size check per call |
| `cco20p1` | N=1e5 intervals | 98.6 vs 33.2 ms (**2.97×**) | 20.4 vs 7.3 MB (**2.81×**) | `CpSteps` uses `std::map<ll, ll>` (200k RB-tree nodes) |
| `dpv` | N=1e5 star+chain | 57.1 vs 29.9 ms (**1.91×**) | 34.7 vs 14.2 MB (**2.44×**) | Eager `RootedTree::sp` + 4× adj copies + `cp_emod` in hot loop |
| `noi16p4` | N=5e5, M=2e5 | 855 vs 541 ms (**1.58×**) | 70.0 vs 29.1 MB (**2.41×**) | `AddMax` uses 64-bit `vector<ll>` + 2× `lower_bound` per `cover()` |
| `ceoi18p1` | N=M=2000, C=50 | 425 vs 289 ms (**1.47×**) | 5.1 vs 5.1 MB (1.00×) | 16B `Opt<ll>` doubles 4000× `before = best` memcpy bandwidth |
| `ioi10p3` | 3000×3000 grid | 1679 vs 1147 ms (**1.46×**) | 107.1 vs 106.3 MB (1.01×) | `vector<bool>` proxy writes + 24× fresh 72 MB `CpPrefix` allocs |
| `dps` | \|K\|=1e4, D=100 | 55.9 vs 44.3 ms (**1.26×**) | 3.5 vs 20.4 MB (0.17×) | Double `%` in circular domain wrap + 10k `next_` vector reallocs |
| `ccc21s2` | 5e5×10, K=1e6 | 29.3 vs 37.8 ms (0.78×) | 10.9 vs 3.9 MB (**2.81×**) | `in` materializes all 1M `E_Stroke` entries vs streaming |
| `coi06p2` | N=1e5, M=5e5, Q=3e5 | 157 vs 145 ms (1.08×) | 64.4 vs 24.0 MB (**2.68×**) | `RootedTree` copies 1M edges to `pair<int, ll>` + `plain` + `sp` + `jump` |
| `noi15p2` | N=1e5, Q=1e5 | 177 vs 183 ms (0.96×) | 29.9 vs 12.0 MB (**2.49×**) | Unused eager `RootedTree::sp` + 64-bit `AssignSum` + `chains()` allocs |
| `btoi17p3` | N=1e5, M=5e4 | 62.2 vs 75.6 ms (0.82×) | 37.4 vs 19.8 MB (**1.90×**) | `RootedTree` 4× adj copies + eager `sp` + buffered `plans` |
| `ccc18s5` | N=M=P=Q=1e5 | 35.7 vs 45.1 ms (0.79×) | 18.3 vs 9.7 MB (**1.90×**) | Global `flights`/`portals` stay live + 24B `Tup2` vs 16B `array<int,4>` |
| `ioi11p2` | N=2e5, K=1e6 | 231 vs 222 ms (1.04×) | 58.8 vs 32.4 MB (**1.82×**) | `RootedTree` + 200k `unordered_map` shells (56B each) + 48B nodes |
| `dpo` | N=21 (2^21 states) | 61.9 vs 119.3 ms (0.52×) | 19.2 vs 11.4 MB (**1.69×**) | `Mod<1000000007>` stores 8B `ll` instead of 4B `uint32_t` |
| `noip09p3` | N=1e5, M=5e5 | 124 vs 155 ms (0.80×) | 29.3 vs 17.7 MB (**1.66×**) | Global `roads` (6 MB) kept live + 2× `vector<vector<int>>` |
| `cco24p1` | N=5e5, M=1e6 | 459 vs 552 ms (0.83×) | 108.4 vs 68.4 MB (**1.58×**) | Global `roads` (12 MB) + padded 16B `Tup1{int, ll}` adj vs 8B `pair<int,int>` |

---

## Root-cause analysis & concrete fixes

### 1. `@memo` falls back to `unordered_map` on bounded range and `Bool` parameters

This is the single largest performance cliff in the audit. In `ioi04p4`, the CP Lang solution writes `@memo fun waste(a: 1..=w, b: 1..=h) -> Int`, and in `dpp` it writes `@memo fun ways(v: V, black: Bool) -> Mod<P>`. Both parameter spaces are small, dense, statically bounded Cartesian products (`600 × 600 = 360,000` states in `ioi04p4`; `100,000 × 2 = 200,000` states in `dpp`). Yet `proto/cpc` only recognizes named `domain` parameters for dense array memoization; any range type (`1..=w`) or `Bool` forces the entire function onto a chained hash table:

```cpp
// Emitted for ioi04p4 (31.86x slower, 5.77x RSS)
unordered_map<array<ll, 2>, ll, CpArrHash> _memo_waste;

ll waste(ll a, ll b) {
    array<ll, 2> _key{(ll)a, (ll)b};
    auto _it = _memo_waste.find(_key);
    if (_it != _memo_waste.end()) return _it->second;
    ll _r = waste_body(a, b);
    _memo_waste.emplace(_key, _r);
    return _r;
}
```

In `ioi04p4`, the cutting-plate DP executes roughly `600 × 600 × 300 ≈ 1.08 × 10^8` lookups. In the original C++, each lookup is a 2-cycle L1/L2 hit in a 1.4 MB `int dp[601][601]` array (170 ms total). In the emitted C++, each lookup hashes a 16-byte `array<ll, 2>` through two splitmix64 rounds and chases pointers across 360,000 heap-allocated hash nodes (~23 MB), burning **5.42 seconds** (instant TLE on a 1–2 s judge). On top of that, `wanted` is emitted as `std::set<Tup0>` and queried with `wanted.count(Tup0{a, b})` on every cache miss (360,000 red-black tree traversals). In `dpp`, the exact same fallback emits `unordered_map<array<ll, 2>, Mod<1000000007>, CpArrHash>` for `(V, Bool)`, turning 200k array reads into 200k heap allocations and contributing heavily to the **3.92×** slowdown.

**Concrete fix (for both `proto/cpc` and `crates/cpl-codegen`):** Lower `@memo` to a flat `vector<Opt<R>>` (or a flat value vector + packed validity bitset/sentinel) whenever every parameter has a finite cardinality: named domains (`len = D_len`), `Bool` (`len = 2`), `Enum` (`len = variant_count`), or bounded integer ranges `lo..hi` / `lo..=hi` whose runtime extent `hi - lo` is known before the first call (with a safety threshold, e.g., product `<= 2×10^7`, before falling back to a hash table). Allocate the flat table once at entry or lazily via a single `if (_memo.empty())` guard rather than re-multiplying dimensions on every recursive call (see `cco18p1` below).

---

### 2. `CpCounter` and `Map` use `std::unordered_map` with heap-node bloat, double lookups, and unsalted `CpHash`

`noi01p5` (meet-in-the-middle over `150^3 = 3,375,000` half-sums) is an **instant MLE** on any 64 MB or 128 MB judge: **198.0 MB RSS vs 35.4 MB** (**5.60×**) and **924 ms vs 200 ms** (**4.62×**). Look at the emitted C++:

```cpp
// Emitted for noi01p5 (4.62x slower, 5.60x RSS -> 198.0 MB)
CpCounter<unordered_map<ll, ll, CpHash>, ll> seen;
...
seen = ([&]{
    auto _c7 = CpCounter<unordered_map<ll, ll, CpHash>, ll>();
    for (const auto& _x6 : sums(Rng{0, (ll)(mid)}))
        _c7.set(_x6, _c7.get(_x6) + 1);   // TWO hash lookups per insert!
    return _c7;
}());
```

Four distinct overheads pile up here:

1. **Chained hash-node memory explosion:** libstdc++'s `std::unordered_map<ll, ll>` allocates a 32-byte heap node plus malloc metadata plus an 8-byte bucket pointer (~48–56 bytes per distinct entry). Across ~2.7M distinct sums, that alone consumes ~140–155 MB, whereas the original C++ uses a static open-addressed flat table `int v[1 << 22][2]` (32 MB, zero mallocs, L2-friendly linear probing).
2. **Double lookup on every increment:** `_c7.set(k, _c7.get(k) + 1)` searches the hash table twice for every single one of the 3,375,000 left-half sums — once in `get()` and again in `set()`!
3. **Materializing two 3.375M-element `vector<ll>`s:** `sums()` builds a `vector<ll>` of 3.375M elements (27 MB, peaking higher during capacity doubling) for the left half and another 27 MB `vector<ll>` for the right half, instead of streaming into the counter or folding directly.
4. **Unsalted `CpHash` is trivially hackable (anti-hash TLE):** Both `CpHash` and `CpArrHash` in `proto/cpc/runtime.hpp` use fixed splitmix64 constants with zero runtime entropy:

```cpp
struct CpHash {
    size_t operator()(ll x) const {
        uint64_t z = (uint64_t)x + 0x9e3779b97f4a7c15ULL;
        z = (z ^ (z >> 30)) * 0xbf58476d1ce4e5b9ULL;
        z = (z ^ (z >> 27)) * 0x94d049bb133111ebULL;
        return (size_t)(z ^ (z >> 31));
    }
};
```

Because splitmix64 is a bijection with a known inverse, our adversarial probe in `bench/max_n_audit.py` inverted 50,000 multiples of libstdc++'s bucket count (`85,229`) and drove **all 50,000 keys into bucket 0**: runtime jumped from **22.0 ms** (random keys) to **977.7 ms** (**44.5× slower**, scaling quadratically to >4 s TLE at `N = 10^5`). The same vulnerability affects `ioi11p2`, `noi01p5`, and every hash-backed `Map`/`Set`/`Counter`.

**Concrete fixes:**
- Salt `CpHash` and `CpArrHash` with a per-process random seed (`static const uint64_t CP_HASH_SALT = chrono::steady_clock::now().time_since_epoch().count() ^ (uintptr_t)&CP_HASH_SALT;`) XORed before mixing.
- Replace `std::unordered_map` inside `Map`, `Set`, and `CpCounter` with a compact open-addressed flat hash table (e.g., Robin Hood or linear-probing power-of-two table storing packed `{key, val}` with a sentinel/metadata byte, ~16–20 bytes per slot vs ~56 bytes per `unordered_map` node). That cuts `noi01p5` RSS from ~198 MB down to ~45–60 MB and removes ~2.7M `malloc` calls.
- Add `inc(k, delta)` (single-lookup upsert) on `CpCounter` instead of emitting `c.set(k, c.get(k) + 1)`.
- In `ioi11p2` (`vector<unordered_map<ll, ll, CpHash>> reach(200000)`), note that empty `std::unordered_map` shells are 56 bytes each (11.2 MB before inserting a single key); a flat hash table struct is 24 bytes empty (pointer + cap + size).

---

### 3. `RootedTree` eagerly builds the LCA sparse table and allocates 4× redundant adjacency lists

Six of the 16 memory regressions — `coi06p2` (2.68× RSS), `dpp` (2.77×), `noi15p2` (2.49×), `dpv` (2.44×), `btoi17p3` (1.90×), and `ioi11p2` (1.82×) — trace directly to `RootedTree` in `proto/cpc/runtime.hpp` and its call sites. Look at what `proto/cpc` emits to construct an unweighted tree in `dpv`, `dpp`, `coi06p2`, `noi15p2`, and `btoi17p3`:

```cpp
// Emitted for dpv, dpp, noi15p2, coi06p2, btoi17p3
tree = ([&]{
    const auto& _g = ([&]{
        vector<vector<int>> _g(D1_V_len);
        for (const auto& _e : edges) { _g[_e.u].push_back(_e.v); _g[_e.v].push_back(_e.u); }
        return _g;
    }());
    vector<vector<pair<int, ll>>> _adj(D1_V_len);
    for (size_t _u = 0; _u < _g.size(); ++_u)
        for (const auto& _a : _g[_u]) _adj[_u].emplace_back((int)(_a), 1);
    return RootedTree(_adj, 0);
}());
```

And inside `RootedTree`'s constructor (`proto/cpc/runtime.hpp:488-503`):

```cpp
sp.assign(1, vector<int>(n));
for (int i = 0; i < n; ++i) sp[0][i] = par[order[i]] < 0 ? 0 : tin[par[order[i]]];
for (int k = 1; (1 << k) <= n; ++k) {
    sp.emplace_back(n - (1 << k) + 1);
    for (int i = 0; i + (1 << k) <= n; ++i)
        sp[k][i] = min(sp[k - 1][i], sp[k - 1][i + (1 << (k - 1))]);
}
...
RootedTree(const vector<vector<pair<int, ll>>>& arcs, int root) : RootedTree(plain(arcs), root) { ... }
```

This does an astonishing amount of wasted work on every tree problem:

1. **Four separate `vector<vector<...>>` graphs for one unweighted tree:** For `coi06p2` (`N = 100,000, M = 500,000`), it builds `map_` (`vector<vector<int>>`), widens all 1,000,000 directed edges into `vector<vector<pair<int, ll>>> _adj` (16 bytes/edge + vector doubling ≈ 24–32 MB of temporary heap!), calls `plain(_adj)` to narrow all 1,000,000 edges *back* into a third `vector<vector<int>>`, and then allocates `kids` (100,000 more heap vectors)!
2. **Eager `sp` sparse table even when `lca()` is never called:** `jump` and `hpos`/`top` are already lazy in `RootedTree`, yet `sp` (`N × log2(N) × 4` bytes ≈ 7 MB at `N = 10^5`, 15 MB at `N = 2×10^5`) is built unconditionally in the constructor! Neither `dpp`, `dpv`, nor `noi15p2` ever calls `lca()`. On `dpv` (29.9 ms original), building `sp` and triple-copying the graph accounts for the bulk of the **1.91×** runtime slowdown and **2.44×** RSS.
3. **Eager `down` (`vector<ll>`) on unweighted trees** (another `8N` bytes duplicating `dep`) and **per-query heap allocation in `chains()`** (`vector<P> out` allocated on every one of the 100k HLD queries in `noi15p2`).

**Concrete fixes:**
- Call the unweighted `RootedTree(const vector<vector<int>>& adj, int root)` overload directly when edges are unweighted (eliminating the `_adj` → `plain(_adj)` double copy completely).
- Make `sp` `mutable` and build it lazily on the first `lca()` call, matching `jump` and `layout()`.
- Store `kids` in a flat CSR (`kid_head[N + 1]`, `kid_to[N - 1]`) instead of `vector<vector<int>>` (100,000 heap vectors → 2 flat vectors, cutting per-tree overhead by ~6–10 MB and improving cache locality in tree DP loops).
- Accept a callback (or reuse a scratch buffer) in `chains()` so HLD queries perform zero heap allocations.

---

### 4. Non-niche-packed `Opt<ll>` (`sizeof == 16` instead of `8`) doubles DP bandwidth and RSS

`ceoi18p1` is a pure controlled experiment on `Opt<ll>`. Both `ceoi18p1_orig.cpp` and `ceoi18p1.cp` use the **exact same algorithm**, including copying the entire 100,001-element DP array on every one of the 4,000 deals (`memcpy(pdp, dp, sizeof(dp))` in original vs `before = best;` in CP Lang). Yet CP Lang is **1.47× slower** (424.5 ms vs 288.5 ms). Why?

In `proto/cpc/runtime.hpp:66-90`, `Opt<int>` (4 bytes, `-1` sentinel) and `Opt<bool>` (1 byte, `-1` sentinel) are niche-packed, **but `Opt<ll>` falls through to the generic template**:

```cpp
template <class T> struct Opt {
    T v{}; bool h = false;   // sizeof(Opt<ll>) == 16 (7 bytes padding)!
    ...
};
```

In `ceoi18p1`, `before` and `best` are `vector<Opt<ll>>` of length 100,001: **1.6 MB** each instead of **800 KB** (`long dp[100005]` with `-1e18` sentinel). Copying 1.6 MB 4,000 times moves **6.4 GB** of memory instead of **3.2 GB**, plus the inner loop runs `cp_step` and `opt_max` branches on every state:

```cpp
// Emitted for ceoi18p1 (1.47x slower)
before = best;   // 1.6 MB x 4,000 = 6.4 GB
for (int c = 0; c < D3_Spare_len; ++c) {
    auto _o7 = before[c]; if (_o7.has()) {
        const auto& b = _o7.get();
        auto _o8 = cp_step(c, d.cores, false, D3_Spare_len); if (_o8.has()) {
            opt_max(best[_o8.get()], cp_add(b, d.gain, 24));
        }
    }
}
```

The same 16-byte `Opt<ll>` hits `cco18p1` (**3.20× slower**, **1.52× RSS** — 11.0 MB vs 7.2 MB): `_memo_best` is `vector<Opt<ll>>` of size `1001 × 1001 × 16 B = 16 MB` (versus original's 4 MB `int dp[1001][1001]`), and every one of the ~3M recursive calls executes a redundant resize check before indexing:

```cpp
// Emitted for cco18p1 (3.20x slower, 1.52x RSS)
ll best(int x, int y) {
    if (_memo_best.size() != (size_t)(D1_Game_len + 1) * (size_t)(D1_Game_len + 1))
        _memo_best.assign((size_t)(D1_Game_len + 1) * (size_t)(D1_Game_len + 1), {});
    auto& _m = _memo_best[(x) * (D1_Game_len + 1) + y];
    if (!_m.has()) _m = best_body(x, y);
    return _m.get();
}
```

**Important note for the Rust compiler (`crates/cpl-codegen`):** `crates/cpl-codegen/src/names.rs:176` currently emits `Ty::Opt(inner)` as `std::optional<T>`, which has the **exact same 16-byte layout** for `std::optional<int64_t>`! Niche-packing `?Int` using `LLONG_MIN` (matching `Opt<int>`/`OptI32`) should be implemented directly in `crates/cpl-codegen` as well as `proto/cpc/runtime.hpp`, and `cco18p1`-style `@memo` tables should allocate once at top-level initialization instead of checking `.size() != ...` inside the recursive call.

---

### 5. `CpSteps` uses `std::map<ll, ll>` instead of a lazy-sorted `vector`

In `cco20p1` (`N = 100,000` intervals), CP Lang is **2.97× slower** (98.6 ms vs 33.2 ms) and uses **2.81× RSS** (20.4 MB vs 7.3 MB). The entire regression lives in one line of `proto/cpc/runtime.hpp:780`:

```cpp
struct CpSteps {
    map<ll, ll> d;   // 48B heap node per endpoint -> 200,000 RB-tree allocations!
    void add(Rng r, ll x) { if (r.lo < r.hi && x != 0) { d[r.lo] += x; d[r.hi] -= x; } }
```

The original `cco20p1_orig.cpp` pushes 200,000 `(pos, delta)` pairs into a flat `vector<pair<long, int>>` (3.2 MB) and calls `std::sort` once. `CpSteps` instead inserts 200,000 nodes into a `std::map<ll, ll>`, paying ~10–13 MB of RB-tree node overhead and pointer-chasing cache misses during both insertion and sweep.

**Concrete fix:** Back `CpSteps` with a `vector<pair<ll, ll>> ev` plus a `bool dirty` flag; `add()` does two `emplace_back`s, and `runs()` sorts and coalesces equal keys once if `dirty` is true (or drops straight into a single scan). That reduces `CpSteps` to ~3.2 MB and matches `cco20p1_orig.cpp`'s 33 ms runtime.

---

### 6. `vector<bool>` proxy overhead + 24× per-iteration 72 MB `CpPrefix` allocations (`ioi10p3`)

In `ioi10p3` (3000×3000 grid, 1001×1001 median window, 24 binary search steps), CP Lang is **1.46× slower** (1678.6 ms vs 1146.8 ms). Look at `some_median_at_most(ll x)`:

```cpp
// Emitted for ioi10p3 (1.46x slower)
bool some_median_at_most(ll x) {
    const CpPrefix low = CpPrefix(([&]() {
        vector<bool> _tab2((size_t)D1_Row_len * (size_t)D2_Col_len);
        for (int i = 0; i < D1_Row_len; ++i)
            for (int j = 0; j < D2_Col_len; ++j)
                _tab2[(i) * D2_Col_len + j] = (((ll)rank_[(i) * D2_Col_len + (j)] <= x));
        return _tab2;
    }()), D1_Row_len, D2_Col_len);
    ...
}
```

Three penalties stack up across the 24 binary search iterations (`24 × 9,000,000 = 2.16 × 10^8` cells):
1. **`vector<bool>` bit-proxy reads and writes:** `proto/cpc` lowers `[Row, Col]Bool` to `std::vector<bool>`, forcing bit-masking shifts on every write in `_tab2` and every read in `CpPrefix`'s constructor (and violating the explicit project guardrail against `vector<bool>` proxies — note that `crates/cpl-codegen` already emits `std::vector<char>` instead).
2. **Double-pass materialization:** `_tab2` is allocated and filled in one 9M-cell pass, then read back in a second 9M-cell pass inside `CpPrefix`, whereas the original fuses `(arr[i][j] <= x)` directly into the prefix-sum recurrence in a single pass.
3. **24× 72 MB `mmap`/`munmap` page-fault churn:** `CpPrefix` stores `vector<ll> p` (`3001 × 3001 × 8 B = 72 MB`, even though the sum of booleans is at most `9×10^6`, fitting in 32-bit `int` = 36 MB), and destroys/reallocates that 72 MB buffer on every single binary-search call (**1.73 GB** of OS page faults!).

**Concrete fixes:**
- Emit `vector<uint8_t>` (or `vector<char>`, matching `crates/cpl-codegen`) instead of `vector<bool>` everywhere in `proto/cpc`.
- Fuse `Prefix.of(tab)` when `tab` is a fresh comprehension into a single pass without materializing the intermediate grid.
- Parameterize `CpPrefix<T>` by element width (`int` when max sum fits in `int32_t`, such as summing `Bool` over `<= 2×10^9` cells) — cutting both memory and bandwidth in half — and allow buffer reuse across loop/binary-search calls.

---

### 7. 64-bit over-widening (`Mod<P>`, `AddMax`/`AssignSum`, padded tuples) and repeated `lower_bound`

Across the 30 ports, unconditional 64-bit storage (`ll`) where 32-bit (`int`/`uint32_t`) is proven sufficient accounts for a systematic **1.5×–2.4× RSS tax**:

- **`Mod<M>` stores `ll v` (8 bytes) instead of `uint32_t v` (4 bytes):** In `dpo` (`2^21 = 2,097,152` states), `vector<Mod<1000000007>>` uses 16 MB vs 8 MB in original `int dp[1 << 21]`, driving `dpo`'s **1.69× RSS** (19.2 MB vs 11.4 MB) even though CP Lang ran nearly 2× *faster* (0.52×). Similarly, `coci20c6p3`'s `2001 × 2001` Pascal table uses 32 MB instead of 16 MB. Using `uint32_t` (with `(uint64_t)v * o.v % M` instead of `__int128`) whenever `M <= 2147483647` halves `Mod<P>` table RSS and speeds up multiplication too.
- **`AddMax` and `AssignSum` hardcode `vector<ll>` (8 bytes):** In `noi16p4` (`N = 500,000`, `2N = 1,000,000` compressed coordinates), `AddMax` allocates `v(4N)` and `lz(4N)` as `vector<ll>` = **64 MB** (vs original's 32 MB `int` segment tree), directly causing **2.41× RSS** (70.0 MB vs 29.1 MB) and heavy L3 thrashing. Worse, `noi16p4.cp`'s `cover()` runs two `lower_bound` binary searches over the 1M-element `coord` vector on every segment add and remove (**2,000,000 binary searches** over an 8 MB `vector<ll>`!), explaining the **1.58×** runtime slowdown (855 ms vs 541 ms). The same 64-bit `AssignSum` contributes 6.4 MB (vs 3.2 MB) to `noi15p2` (2.49× RSS).
- **Struct padding in generated `Tup*` records:** In `cco24p1` (`N = 5e5, M = 1e6`), `adj` stores `Tup1 { int f0; ll f1; }` (**16 bytes** due to 4 bytes padding) vs original's `pair<int, int>` (**8 bytes**). Across 2,000,000 directed adjacency entries (plus `vector` capacity doubling), that single type widening adds ~30 MB of RSS (`cco24p1`: 108.4 MB vs 68.4 MB, **1.58×**). In `ccc18s5`, `Tup2 { ll cost; E_Link link_; }` is 24 bytes vs original's 16-byte `array<int, 4>`.

**Concrete fixes:**
- Specialize `Mod<M>` storage to `uint32_t` when `M <= INT32_MAX` (using `uint64_t` multiply) and `uint64_t` only for larger moduli.
- Template `AddMax<T>` and `AssignSum<T>` on value width (`int` when the declared range proves sums/maxima fit in `int32_t`).
- Narrow tuple fields (`I32`/`int`) when value range analysis proves the field fits in 32 bits (e.g., edge tolls `<= 10^9` in `cco24p1`, costs in `ccc18s5`).
- For coordinate-compressed segment problems like `noi16p4`, pre-map endpoints once into domain indices rather than repeating `coord.rank()` inside the two-pointer loop (or provide a stdlib `compress(xs)` returning both the domain and the mapped array).

---

### 8. Un-elided Euclidean/wrapping modulo, `cp_win` bounds checks, and `SegTree`'s `std::function`

Three inner-loop runtime primitives add measurable constant-factor overhead when they appear in tight loops:

1. **Double `%` in circular domain wrapping (`dps`, 1.26× slower — 55.9 ms vs 44.3 ms):** In `dps`, advancing a circular remainder domain `Rem` by a digit `x` emits two signed 64-bit hardware divisions on every one of the 10,000,000 inner-loop iterations:
   ```cpp
   // Emitted for dps (1.26x slower)
   next_[(int)(((((ll)(r) + ((ll)(x))) % (ll)(D3_Rem_len)) + (ll)(D3_Rem_len)) % (ll)(D3_Rem_len))] += below[r];
   ```
   Alongside that, `next_ = vector<Mod<1000000007>>(...)` allocates and frees a heap vector 10,000 times, and `below = next_` copies instead of swapping. Because `0 <= r < d` and `0 <= x <= 9`, a single `int nr = r + x; if (nr >= d) nr -= d;` (or even one `%` when `a >= 0`) eliminates 20 million `idiv` instructions.
2. **Unconditional `cp_emod` (`dpv` + microbenchmark):** On our 5×10^7-op microbenchmark with a runtime positive modulus, `cp_emod` runs **1.13× slower** than native `%` (341.1 ms vs 302.1 ms) because of the zero check and negative-remainder adjustment branch. `crates/cpl-codegen` already solves this cleanly via `ranges.rs` (lowering proven-non-negative `/` and `%` straight to native operators and using branchless bitmask adjustment otherwise); `proto/cpc` should mirror that optimization.
3. **`CpView` / `cp_win` bounds checks (`lends` slices):** On our 2×10^8-op loop microbenchmark (`sum_checked` vs `sum_raw`), calling `cp_win(i, v.lo, v.hi, line)` per access runs **1.31× slower** (102.0 ms vs 78.1 ms) because the `[[noreturn]] cp_fail(...)` branch inside the loop prevents GCC `-O2` from auto-vectorizing or unrolling the loop. When iterating `for i in view.indices` (or any range proven inside `[v.lo, v.hi)`), eliding `cp_win` inside the loop — or hoisting a single `lo >= v.lo && hi <= v.hi` check before the loop — recovers full SIMD vectorization.
4. **Type-erased `std::function` in `SegTree`:** `SegTree<T>` (`proto/cpc/runtime.hpp:306-315`) stores `std::function<T(const T&, const T&)> op` and `std::function<T(int)> src`. Every `set`/`query`/`refresh` makes `O(log N)` indirect calls through `std::function`, blocking inlining. Parameterizing `SegTree<T, Op, Src>` by functor template types (using CTAD or a helper `make_segtree`) makes `op` inline completely to 1–2 instructions.

---

### 9. Whole-input materialization & global variable lifetimes from `in` declarations

Four ports (`ccc21s2` 2.81× RSS, `noip09p3` 1.66×, `cco24p1` 1.58×, `ccc18s5` 1.90×, plus `btoi17p3` 1.90×) use more memory solely because every top-level `in` array (`strokes`, `roads`, `flights`/`portals`, `plans`) is emitted as a **file-scope global `vector`** that stays allocated for the entire process lifetime — even after being transformed into an adjacency list or consumed in one streaming pass.

- In `ccc21s2` (`K = 1,000,000`), `strokes` occupies 8 MB while `row_odd`/`col_odd` take < 1 MB (original streams `K` strokes directly: 3.9 MB vs 10.9 MB).
- In `cco24p1` (`M = 1,000,000`), `roads` holds 12 MB of dead input data throughout Dijkstra.
- In `noip09p3` (`M = 500,000`), `roads` holds 6 MB of dead input data throughout both graph traversals.

**Concrete fix:** Emit a liveness-based `.clear(); .shrink_to_fit();` (or `vector<T>().swap(v)`) right after the last read of large input containers when another large structure is allocated afterward — and for single-pass read-only inputs (`ccc21s2`), fuse reading into the loop when no `CP_GEN` or `where` clause requires buffering.

---

### 10. Stack-overflow vulnerability on recursive `@memo` over deep trees

Our adversarial chain probe ran `dpp` on a 100,000-node path graph (`1-2-3-...-100000`) under Linux's default 8 MB stack (`ulimit -s 8192`): **both CP Lang's recursive `@memo` and the original recursive DFS segfaulted with SIGSEGV (exit code 139)**. With `ulimit -s unlimited`, both succeeded, though CP Lang used **47.7 MB RSS** vs 19.1 MB (2.50×) and **85.3 ms** vs 22.1 ms (3.86×).

Because CP Lang already computes an iterative DFS preorder (`RootedTree::order`), there is an opportunity that C++ lacks: a `@memo` function whose recursion follows `tree.kids[v]` (or a compiler diagnostic / bottom-up lowering over `tree.order.rbegin()..rend()`) can evaluate in `O(1)` call stack space and survive strict 8 MB stack judges unconditionally.

---

## Emitted C++17 file size & readability

All 30 emitted C++17 files are comfortably under the **64 KB** judge ceiling:

- **Range:** 25.3 KB (`dpo`, 565 lines) to 36.7 KB (`dmopc21c9p4`, 844 lines); median **28.2 KB** (627 lines).
- **Breakdown:** Roughly **23–25 KB (~500–525 lines)** of *every* emitted file is the un-treeshaken core of `proto/cpc/runtime.hpp`, while the actual translated user program is only **1.5–12 KB** (~40–160 lines).

Two concrete issues stand out for judge submissions and readability:

1. **Un-treeshaken core runtime:** `proto/cpc/runtime.hpp` wraps 16 tail helpers (`DenseSet`, `CpPrefix`, `CpSteps`, `CpHash`, etc.) in `// @part ... // @end`, but leaves `RootedTree` (90 lines), `SegTree`/`AssignSum`/`AddMax` (70 lines), `cp_dijkstra`/`cp_scc`/`CpDsu` (65 lines), `StrHash` (20 lines), `Mod` (18 lines), and all `CP_GEN` random-generator helpers (~50 lines) always emitted. Wrapping those in `@part` and stripping `if (CP_GEN) ...` dead branches in release (`cpl build`) mode will shrink typical emitted files from **~28 KB down to ~4–10 KB** and cut compile time as well.
2. **Code readability:** In `proto/cpc`, emitted `main()` is cluttered with `if (CP_GEN) { out_int(...); }` on every single read statement, deeply nested IIFE lambdas (`([&]() { ... }())`), numbered temps (`_k1`, `_o7`, `_x12`), and no original `.cp` source comment header (whereas `crates/cpl-codegen` already emits the original `.cp` source as a top comment header and cleaner straight-line statements). Stripping `CP_GEN` from submission builds and hoisting single-use lambdas will make the emitted C++ immediately readable to contest judges.

---

## What already works well (and why 14 ports beat original C++)

Before tuning anything, it is worth noting why CP Lang already achieved a **0.92× median runtime** and beat accepted contest C++ by 15%–73% on nearly half the corpus:

- **Custom buffered `read(2)` / `write(2)` I/O (`cp_ibuf`/`cp_obuf`):** Dominates input-heavy benchmarks (`ccc22s4` 0.53×, `apio09p3` 0.57×, `ioi11p4` 0.63×, `dmopc21c9p4` 0.64×, `ccc21s2` 0.78×, `ccc18s5` 0.79×, `noip09p3` 0.80×, `btoi17p3` 0.82×, `cco24p1` 0.83×) against `std::cin` originals.
- **Cleaner algorithmic primitives in stdlib:** `ccc21s4` ran **3.7× faster** (0.27× — 77.7 ms vs 290.2 ms) because CP Lang's iterative `SegTree.of(...)` beat the original's `std::multiset<int>` RB-tree updates; `ccc20s3` ran **3.1× faster** (0.32× — 30.0 ms vs 93.4 ms) because `StrHash` mod `2^61 - 1` + hash set beat the original's double-mod `std::set`; `noi14p4` ran **3.3× faster** (0.30× — 91.3 ms vs 302.2 ms, and **0.34× RSS**) with tight domain-indexed KMP loops; `dpo` ran **1.9× faster** (0.52× — 61.9 ms vs 119.3 ms) using bit-iteration (`_m3 &= _m3 - 1`, `__builtin_ctz`) instead of scanning all 21 bits per mask.

---

## Prioritized optimization roadmap

**P0 — Judge-killers (TLE, MLE, anti-hash, stack overflow)**
1. **Dense array lowering for `@memo`** over bounded ranges (`lo..hi`, `lo..=hi`), `Bool`, and `Enum` (fixes `ioi04p4` 31.86× → ~1× and `dpp` 3.92× → ~1×).
2. **Salted open-addressed flat hash table** replacing `std::unordered_map` in `Map`/`Set`/`CpCounter`, plus single-lookup `CpCounter::inc` (fixes `noi01p5` 198 MB MLE / 4.62× slowdown and the 44.5× `CpHash` anti-hash exploit).
3. **Lazy `RootedTree::sp` + direct unweighted constructor + flat CSR `kids`** (fixes 6 tree RSS regressions of 1.8×–2.7× and `dpv`'s 1.91× slowdown).
4. **Niche-packed 8-byte `Opt<ll>` (`LLONG_MIN` sentinel)** in both `proto/cpc/runtime.hpp` and `crates/cpl-codegen` (fixes `ceoi18p1` 1.47× slowdown and `cco18p1` 1.52× RSS).

**P1 — High-leverage constant-factor & RSS fixes**
5. **Lazy-sorted `vector` for `CpSteps`** instead of `std::map` (fixes `cco20p1` 2.97× runtime / 2.81× RSS).
6. **Replace `vector<bool>` with `vector<uint8_t>` in `proto/cpc` + fuse & narrow `CpPrefix<T>`** (fixes `ioi10p3` 1.46× slowdown and 1.7 GB heap churn).
7. **4-byte `uint32_t` `Mod<M>` for `M <= INT32_MAX` and parameterized `AddMax<T>`/`AssignSum<T>`** (fixes `dpo` 1.69× RSS, `noi16p4` 2.41× RSS, `noi15p2` 2.49× RSS).
8. **Elide double `%` on circular domain wrap, elide `cp_emod` on proven-non-negative operands, hoist `cp_win` out of loops, and template `SegTree`'s `op`/`src`** (fixes `dps` 1.26×, `dpv`, and the 1.31× `cp_win` / 1.13× `cp_emod` microbenchmark penalties).

**P2 — Submission polish & safety**
9. **Tree-shake the remaining `runtime.hpp` structs and strip `CP_GEN` in release builds** (shrinks emitted C++ from ~28 KB to ~4–10 KB and greatly improves readability).
10. **Free dead global input arrays after last use** (cuts RSS on `ccc21s2`, `cco24p1`, `noip09p3`, `ccc18s5`) and **support iterative bottom-up `@memo` lowering or stack-depth warnings** for tree DPs like `dpp`.

---

## Left untested

- **Interactive port (`aac5p2`):** Excluded from the batch max-N sweep because it requires the interactive judge (`proto/ports/aac5p2_judge.py`) rather than stdin/stdout differential execution.
- **Fenwick-specific port:** No dedicated Fenwick tree port currently exists in `proto/ports/`; range-structure coverage was measured via `SegTree` (`ccc21s4`, `ccc21s5`), `AddMax` (`noi16p4`), `AssignSum` (`noi15p2`), and `CpSteps` (`cco20p1`).
- **Rust `cpl build` backend (`crates/cpl-codegen`) end-to-end at max-N:** The Rust compiler does not yet implement the full `proto/cpc` surface (e.g., `@memo`, `RootedTree`, `SegTree`, `Mod<P>`, `Steps`), so all 30 max-N measurements were compiled through `proto/cpc`, while `crates/cpl-codegen` was audited by static inspection (`names.rs`, `runtime.rs`, `ranges.rs`) to carry forward every relevant finding.
- **ASan/UBSan at max-N:** Not run as part of this timing/RSS sweep (sanitizers distort both wall-clock time and RSS); small-N sanitizer testing remains covered by `proto/run_all.sh` plus standalone ASan/UBSan stress tests for `CpHashMap`, `AddMax`, `AssignSum`, `Mod<M>`, `Opt<Mod<M>>`, `CpSteps`, and `RootedTree`.

---

## Follow-up: Prototype P0/P1 codegen & runtime fixes (after-results)

Pure codegen and runtime optimizations were implemented in `proto/cpc/compiler.py` and `proto/cpc/runtime.hpp` with zero syntax changes and zero edits to `crates/`:

1. **Runtime-salted `CpHash` / `CpArrHash` + compact open-addressed `CpHashMap` + single-lookup `CpCounter::inc`:** Mixed a per-process runtime salt (`steady_clock ^ pid ^ stack address` via `splitmix64`) into `CpHash` and `CpArrHash`, replaced `std::unordered_map` in `HashMap`, `CpCounter`, and sparse `@memo` with a single-allocation linear-probing `CpHashMap` (16 B/slot sentinel-packed specialization for `ll` keys with `LLONG_MIN` side-slot and backward-shift deletion), and added `CpCounter::inc` + `reserve_hint` in `counts(...)`.
2. **Dense flat-array `@memo` for bounded ranges, `Bool`, and unit `Enum`:** Extended dense `@memo` lowering beyond named domains to `Bool`, unit enums, and bounded `TInt` ranges (`lo..hi`, `lo..=hi`) with exact lower-bound offsetting and product-cap fallback to `CpHashMap`, moving table initialization into a cold `noinline` helper and inlining the wrapper into the memo body.
3. **Lazy `RootedTree::sp`, CSR `kids`, and fused `Rooted(Graph.undirected(edges), root)`:** Made the LCA sparse table lazy (built on first `lca()`), replaced `vector<vector<int>> kids` with flat `KidsCSR`, replaced `chains()`'s heap `vector` with stack-allocated `ChainList`, eliminated the weighted `plain()` copy, and fused `Rooted(Graph.undirected(edges), root)` into a single CSR pass.
4. **Compact value types & hot-path primitives:** Narrowed `Mod<M>` and `Opt<Mod<M>>` to 4 bytes (`uint32_t`) when `M <= INT32_MAX` (with `__uint128_t` preserved for 64-bit moduli), added branch-minimized `opt_min`/`opt_max` for 8-byte `Opt<ll>`, switched `AssignSum` and `AddMax` to 2N Euler-tour indexing (halving segment-tree memory), replaced `CpSteps`'s `std::map` with a lazy-sorted flat event vector, lowered `Bool` arrays to `vector<uint8_t>`, fused and narrowed `CpPrefix<int>` for small-sum grids, and added proven-positive `cp_ediv_pos`/`cp_emod_pos` and `cp_cyc_nonneg` fast paths.

### Before vs. after (`python3 bench/max_n_audit.py --runs 3 --adversarial`)

Across all 30 max-N ports (30/30 token-for-token output agreement, all emitted C++17 files 30.7–48.1 KB, median 34.0 KB):

- **Median runtime ratio:** `0.92×` → **`0.70×`** (**25/30** ports faster than accepted contest C++, up from 17/30).
- **Median peak RSS ratio:** `1.55×` → **`1.00×`** (exact parity with accepted contest C++).
- **Ports exceeding 1.25× runtime:** `10` → **`4`** (worst cliff `ioi04p4` dropped from **31.86×** to **1.61×**).
- **Ports exceeding 1.50× RSS:** `16` → **`8`** (zero 128 MB MLEs; `noi01p5` dropped by **105 MB**).

| Port | Before time (ratio) | After time (ratio) | Before RSS (ratio) | After RSS (ratio) |
| :--- | :--- | :--- | :--- | :--- |
| `ioi04p4` | 5423 ms (31.86×) | **282.7 ms (1.61×)** | 27.4 MB (5.77×) | **6.0 MB (1.26×)** |
| `noi01p5` | 924 ms (4.62×) | **377.4 ms (1.66×)** | 198.0 MB (5.60×) | **93.1 MB (2.63×)** |
| `dpp` | 96.3 ms (3.92×) | **17.0 ms (0.61×)** | 36.7 MB (2.77×) | **11.2 MB (0.85×)** |
| `cco20p1` | 98.6 ms (2.97×) | **28.9 ms (0.88×)** | 20.4 MB (2.81×) | **13.1 MB (1.80×)** |
| `dpv` | 57.1 ms (1.91×) | **19.4 ms (0.52×)** | 34.7 MB (2.44×) | **9.8 MB (0.69×)** |
| `ceoi18p1` | 425 ms (1.47×) | **312.4 ms (1.03×)** | 5.1 MB (1.00×) | **5.1 MB (1.00×)** |
| `ioi10p3` | 1679 ms (1.46×) | **997.1 ms (0.86×)** | 107.1 MB (1.01×) | **71.8 MB (0.67×)** |
| `dps` | 55.9 ms (1.26×) | **13.8 ms (0.31×)** | 3.5 MB (0.17×) | **3.5 MB (0.17×)** |
| `coi06p2` | 157 ms (1.08×) | **123.2 ms (0.83×)** | 64.4 MB (2.68×) | **35.9 MB (1.50×)** |
| `noi15p2` | 177 ms (0.96×) | **149.1 ms (0.79×)** | 29.9 MB (2.49×) | **14.4 MB (1.20×)** |
| `btoi17p3` | 62.2 ms (0.82×) | **48.9 ms (0.57×)** | 37.4 MB (1.90×) | **22.8 MB (1.15×)** |
| `ioi11p2` | 231 ms (1.04×) | **153.0 ms (0.69×)** | 58.8 MB (1.82×) | **46.8 MB (1.44×)** |
| `dpo` | 61.9 ms (0.52×) | **41.8 ms (0.34×)** | 19.2 MB (1.69×) | **11.2 MB (0.99×)** |
| `noi16p4` | 855 ms (1.58×) | **895.9 ms (1.53×)** | 70.0 MB (2.41×) | **49.2 MB (1.69×)** |
| `cco18p1` | 28.4 ms (3.20×) | **31.4 ms (3.52×)** | 11.0 MB (1.52×) | **11.0 MB (1.52×)** |

**Adversarial probes (after):**

- **Anti-hash (`unordered_map<ll, ll, CpHash>`, N = 50,000):** `bucket0_size_killer` dropped from **50,000** to **0**; slowdown dropped from **44.5×** (977.7 ms) to **2.64×** (62.1 ms vs 23.5 ms, driven by parsing 20-digit 64-bit random keys vs 10-digit keys in `std::cin`).
- **Anti-hash (`CpHashMap<ll, ll, CpHash>`, power-of-2 killer, N = 50,000):** 26.0 ms vs 18.3 ms (**1.42×**), zero collision blowup.
- **Deep chain `dpp` (`N = 100,000`, unlimited stack):** 16.9 ms vs 23.4 ms (**0.72×**, down from 85.3 ms / 3.86×), 17.0 MB vs 19.2 MB (**0.89×**, down from 47.7 MB / 2.50×). Default 8 MB stack still overflows for both CP Lang and original C++ (recursive DFS; bottom-up `@memo` lowering remains P2).
- **Differential & sanitizer regression check:** Full `bash proto/run_all.sh` passed with 0 failures across all 91 ports (200 randomized tests each) plus `bad`, `lend`, `ergo`, `gen`, and ASan/UBSan `redteam` suites (including new `memo_bounded_range` and `hashmap_sentinel` redteam tests).

