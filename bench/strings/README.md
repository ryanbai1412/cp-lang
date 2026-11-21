# Strings stdlib benchmarks

Matched CP Lang (`std.strings`) vs hand-written C++17 (`g++ -std=c++17 -O2`, best of 5 runs, outputs verified equal).

Run: `bash bench/strings/run.sh`

| Benchmark | Workload | CP Lang | C++17 | Ratio | Emitted C++ |
| --- | --- | ---: | ---: | ---: | ---: |
| `z_prefix` | `z_function` + `prefix_function` on `n = 1,000,000` | 33.8 ms | 30.2 ms | 1.12x | 4,662 B |
| `rolling_hash` | `RollingHash.new` (`n = 500,000`) + 200k `slice`/`lcp` | 53.6 ms | 56.3 ms | 0.95x | 9,798 B |
| `hash2d` | `Hash2D<R, C>.new` (`500 x 500`) + 500k `rect` | 23.9 ms | 21.6 ms | 1.11x | 14,589 B |
| `suffix_array` | `SuffixArray.new` (`n = 200,000`) + `distinct` + 200k `lcp_between` | 49.8 ms | 51.2 ms | 0.97x | 18,046 B |
| `suffix_automaton` | `SuffixAutomaton<Alpha>.of` (`n = 200,000`) + `occurrences` + `distinct` | 50.8 ms | 49.7 ms | 1.02x | 9,437 B |
| `palindromic_tree` | `PalindromicTree<Alpha>.of` (`n = 1,000,000`) + `occurrences` + `distinct` | 22.4 ms | 24.1 ms | 0.93x | 6,713 B |
| `aho_corasick` | `AhoCorasick<Alpha>.new` (20k x 10) + 500k `step`/`hit` | 25.9 ms | 25.5 ms | 1.01x | 7,006 B |
| `trie` | Binary trie (`n = 25,000`, `L = 25`, `struct Trie(cnt: Int, kid: [0..2]?Int)`) | 45.5 ms | 41.5 ms | 1.10x | 10,521 B |
| `manacher_lyndon` | `manacher` + `lyndon_factorization` + `min_rotation` on `n = 500,000` | 18.9 ms | 18.0 ms | 1.05x | 6,988 B |
| `lyndon_runs` | `lyndon_runs` (Bannai runs theorem) on `n = 100,000` | 64.1 ms | 62.8 ms | 1.02x | 19,574 B |

Summary (10 matched benchmarks, covering every `std.strings` algorithm and structure): **p50 = 1.02x** (target <= 1.10x), **max = 1.12x** (target <= 1.50x).

Notes:
- `rolling_hash` dropped from 1.61x to 1.16x with `unchecked:` behind entry asserts in `new`/`raw_slice`/`eq_sub`, and to 0.88x–0.95x by halving 64-bit divisions in `eq_sub` via a single-mod difference check `((p[a+len] - p[b+len]) - (p[a] - p[b]) * pow[len]) % M == 0` (bounded by `M^2 + M < 1.01e18 < 2^63`).
- `hash2d` dropped from 1.66x (43.4 ms) to 1.11x (23.9 ms) by flattening `pre1`/`pre2` from `List<List<Int>>` to flat 1D `List<Int>` with `stride = cols + 1`, precomputing `brc = br * bc % M` (8 -> 2 divisions/cell), and using `unchecked:` behind entry asserts.
- `suffix_array` dropped from 1.10x (51.3 ms) to 0.97x–1.00x by preserving `Cast::Raw` across `settle_indexes`/`freeze_place` for nested `unchecked:` assignments (`sa[bkt[c]] = pos`).
