# Strings stdlib benchmarks

Matched CP Lang (`std.strings`) vs hand-written C++17 (`g++ -std=c++17 -O2`, best of 5 runs, outputs verified equal).

Run: `bash bench/strings/run.sh`

| Benchmark | Workload | CP Lang | C++17 | Ratio | Emitted C++ |
| --- | --- | ---: | ---: | ---: | ---: |
| `z_prefix` | `z_function` + `prefix_function` on `n = 1,000,000` | 28.3 ms | 27.2 ms | 1.04x | 4,662 B |
| `rolling_hash` | `RollingHash.new` (`n = 500,000`) + 200k `slice`/`lcp` | 42.8 ms | 46.8 ms | 0.91x | 9,798 B |
| `suffix_array` | `SuffixArray.new` (`n = 200,000`) + `distinct` + 200k `lcp_between` | 46.3 ms | 46.3 ms | 1.00x | 18,127 B |
| `suffix_automaton` | `SuffixAutomaton<Alpha>.of` (`n = 200,000`) + `occurrences` + `distinct` | 47.4 ms | 45.4 ms | 1.04x | 9,437 B |
| `palindromic_tree` | `PalindromicTree<Alpha>.of` (`n = 1,000,000`) + `occurrences` + `distinct` | 21.5 ms | 21.9 ms | 0.98x | 6,722 B |
| `aho_corasick` | `AhoCorasick<Alpha>.new` (20k x 10) + 500k `step`/`hit` | 23.0 ms | 22.9 ms | 1.00x | 6,970 B |
| `trie` | Binary trie (`n = 25,000`, `L = 25`, `struct Trie(cnt: Int, kid: [0..2]?Int)`) | 38.5 ms | 36.2 ms | 1.07x | 10,521 B |
| `manacher_lyndon` | `manacher` + `lyndon_factorization` + `min_rotation` on `n = 500,000` | 16.0 ms | 15.7 ms | 1.02x | 6,988 B |
| `lyndon_runs` | `lyndon_runs` (Bannai runs theorem) on `n = 100,000` | 54.1 ms | 55.1 ms | 0.98x | 19,670 B |

Note: `rolling_hash` dropped from 1.61x to 1.16x with `unchecked:` behind entry asserts in `new`/`raw_slice`/`eq_sub`, and to 0.88x–0.91x by halving 64-bit divisions in `eq_sub` via a single-mod difference check `((p[a+len] - p[b+len]) - (p[a] - p[b]) * pow[len]) % M == 0` (bounded by `M^2 + M < 1.01e18 < 2^63`). `suffix_array` dropped from 1.10x (51.3 ms) to 1.00x (46.3 ms) by preserving `Cast::Raw` across `settle_indexes`/`freeze_place` for nested `unchecked:` assignments (`sa[bkt[c]] = pos`).
