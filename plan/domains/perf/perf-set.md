# Perf set: what builds, and why the rest does not

Measured 2026-11-22 with `python3 bench/v03_perf_bench.py --suite dryrun,corpus,bench --runs 5 --min-ms 10 --jobs 1 --json-out <file>` (release `cpl`, g++ 11.4 `-O2`; emitted C++ at `-std=c++17`, `orig.cpp` at `-std=gnu++20`). Re-run it to refresh; every skip carries its reason in the JSON `reason` field.

| suite | pairs | measured or ok | not building / not comparable |
|---|---|---|---|
| dryrun (`research/dryrun/v03`) | 11 | 11 | 0 (p06_labyrinth fixed by the `(char)` cast commit) |
| bench (`bench/*/`) | 54 | 38 timed >= 10 ms (40 ok) | 14 (below; 6 fixed in `bench/` by perf) |
| corpus (`corpus/problems/*`) | 307 | 190 ok, 70 timed >= 10 ms (rerun on main 88a62fca8) | 117 (the 187 below, minus those that build since) |

Pairs that build but compare different algorithms are listed in `bench/unmatched.txt`; the bench times them and leaves them out of p50/max (`cf2101c`, `cf2115a`).

## bench: 14 pairs that cannot build yet

Fixed by perf in `bench/`: `containers/bitset` (copy via `shifted(0)`; `Bitset` has no `.clone()`), `containers/flatmap` (`FlatMap.new(0, cap:)`), `strings/trie` (`var` fields), `strings/palindromic_tree` and `strings/suffix_automaton` (`occ_counts`), `containers/monodeque` (`Window(min)`, which keeps a monotone deque for a selective op).

| pairs | why | owner |
|---|---|---|
| flow/bounded, flow/closure, flow/dinic, flow/general, flow/globalcut, flow/gomory, flow/matching, flow/pushrelabel, flow/wgeneral | `spec/std/flow.cp` uses `V.K` (domain size), which no std module declares on v0.3 (`r.len` is the spec's name, `spec/core/04-indices.md`) | stdlib |
| math/ntt | builds, but times out: generic `convolve<T>` is the O(n·m) loop for `Mod<p>`; the NTT path (`convolve_prime`) is not reachable from `Mod` lists | stdlib |
| ranges/range_fenwick | `std.fenwick` has no `RangeFenwick` (range add, range sum) | stdlib |
| seq/scan | `std.seq` has no `suffix_scan` (`suffix_fold` is cut-indexed, not a drop-in) | stdlib |
| seq/scan_in_place | `std.seq` has no `scan_in_place` / `suffix_scan_in_place` | stdlib |
| seq/slice_nth_element | `nth_element` is now `select`, but `select` on a list slice fails: `List<Int>` slice is not `IndexedMut` | stdlib + compiler |

## corpus: 187 pairs not measured

Ports are idioms-owned (`corpus/`); perf does not edit them. Requests are in `plan/domains/idioms/worklog.md` and `plan/domains/stdlib/worklog.md`.

| count | why | owner |
|---|---|---|
| 54 | port calls a std API v0.3 does not have (stdlib or port drift) | idioms (port drift) / stdlib (missing API) |
| 40 | other type/check error in the port | idioms |
| 29 | orig.cpp does not build with g++ 11 -std=gnu++20 (C++20/23 headers: <format>, std::span, concepts) | perf (harness): `--cxx23 'python3 -m ziglang c++'` (zig 0.15.1, clang + libc++) builds 22 of 29, both sides of each pair on that compiler; 8 measured, the rest skip for no input, a mismatch, or a front-end crash; `wtf26a` needs `std::ranges::iota` (missing in that libc++) |
| 21 | port uses a 0.2 call shape (min/max/any/all arity) | idioms |
| 13 | output differs from orig.cpp on the sampled input (idioms: port or reference wrong) | idioms |
| 13 | no input: neither tests/ nor a generator | idioms |
| 10 | port indexes with a ?V (V.first etc.) that v0.3 makes optional | idioms |
| 7 | port uses renamed/0.2 syntax | idioms |

### Pairs by reason

- **port calls a std API v0.3 does not have (stdlib or port drift)** (54): `agc077a`, `cf1943b`, `cf2035f`, `cf2057f`, `cf2062e2`, `cf2066e`, `cf2069e`, `cf2089b2`, `cf2089c2`, `cf2096f`, `cf2101d`, `cf2101e`, `cf2124g`, `cf2129c3`, `cf2135f`, `cf2137f`, `cf2138b`, `cf2147c`, `cf2147f`, `cf2150d`, `cf2152c`, `cf2152d`, `cf2152f`, `cf2152g`, `cf2159d1`, `cf2159d2`, `cf2165f`, `cf2178d`, `cf2178e`, `cf2178g`, `cf2180f2`, `cf2183f`, `cf2183h`, `cf2187f1`, `cf2187f2`, `cf2201e`, `cf2211g`, `cf2222c`, `cf2229d`, `cf2229f`, `cf2237a`, `cf2237b`, `cf2245f`, `cf2255e1`, `cf2255e2`, `cf2261e`, `cf2262a2`, `cf2262b`, `cf2262e`, `cf2268d`, `ioi11p2`, `ucup4s24f`, `ucup5s05e`, `ucup5s05f`
- **other type/check error in the port** (40): `cf1284e`, `cf2057e2`, `cf2062e1`, `cf2062h`, `cf2066d2`, `cf2084c`, `cf2084e`, `cf2084f`, `cf2084g1`, `cf2084h`, `cf2096g`, `cf2096h`, `cf2101f`, `cf2115f1`, `cf2124d`, `cf2124f2`, `cf2127f`, `cf2129d`, `cf2137g`, `cf2147h`, `cf2150f`, `cf2159e`, `cf2165e`, `cf2180h2`, `cf2183e`, `cf2187c`, `cf2187d`, `cf2201d`, `cf2207f`, `cf2222f`, `cf2237e`, `cf2237h`, `cf2245c`, `cf2255b`, `cf2261g`, `ucup4s25e`, `ucup5s02h`, `ucup5s05b`, `wtf26b`, `wtf26d`
- **orig.cpp does not build with g++ 11 -std=gnu++20 (C++20/23 headers: <format>, std::span, concepts)** (29): `agc077c`, `agc079a`, `cf1909g`, `cf2069f`, `cf2084g2`, `cf2127e`, `cf2129f1`, `cf2135c`, `cf2138a`, `cf2138d`, `cf2152h1`, `cf2157f`, `cf2178f`, `cf2178i`, `cf2207c`, `cf2207d`, `cf2207h1`, `cf2207h3`, `cf2219c`, `cf2222g`, `cf2255f`, `cf2261h`, `cf2262c`, `cf2268e`, `ucup5s01a`, `ucup5s03a`, `ucup5s03f`, `ucup5s06a`, `wtf26a`
- **port uses a 0.2 call shape (min/max/any/all arity)** (21): `cf2115e`, `cf2127c`, `cf2127d`, `cf2135b`, `cf2138c1`, `cf2147g`, `cf2159f`, `cf2180f1`, `cf2183d1`, `cf2187a`, `cf2207e1`, `cf2211c1`, `cf2211c2`, `cf2222a`, `cf2229b`, `cf2229e`, `cf2261b`, `cf2262d`, `ioi10p3`, `ucup4s25c`, `ucup5s06g`
- **output differs from orig.cpp on the sampled input (idioms: port or reference wrong)** (13): `cf2062g`, `cf2066b`, `cf2129b`, `cf2138e1`, `cf2138e2`, `cf2159c`, `cf2165c`, `cf2187b`, `cf2222b`, `cf2255a`, `cf2261c`, `cf2261d2`, `cf2262a1`
- **no input: neither tests/ nor a generator** (13): `cf2129c1`, `cf2135d1`, `cf2152e`, `cf2159a`, `cf2183g`, `cf2201f2`, `cf2211e`, `cf2211f`, `cf2222e`, `cf2237g`, `ucup4s25h`, `ucup5s02e`, `ucup5s02f`
- **port indexes with a ?V (V.first etc.) that v0.3 makes optional** (10): `cf1984e`, `cf2089e`, `cf2115b`, `cf2124i`, `cf2129f2`, `cf2152h2`, `cf2183d2`, `cf2229i`, `cf2237i1`, `cf2237i2`
- **port uses renamed/0.2 syntax** (7): `cf2022c`, `cf2066f`, `cf2150e1`, `cf2150e2`, `cf2229g`, `cf2268c`, `ucup4s25b`
