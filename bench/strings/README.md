# Strings stdlib benchmarks

Matched CP Lang (`std.strings`) vs hand-written C++17 (`g++ -std=c++17 -O2`, best of 5 runs, outputs verified equal).

Run: `bash bench/strings/run.sh`

| Benchmark | Workload | CP Lang | C++17 | Ratio | Emitted C++ |
| --- | --- | ---: | ---: | ---: | ---: |
| `z_prefix` | `z_function` + `prefix_function` on `n = 1,000,000` | 28.6 ms | 27.7 ms | 1.03x | 4,958 B |
| `rolling_hash` | `RollingHash.new` (`n = 500,000`) + 200k `slice`/`lcp` | 45.5 ms | 48.2 ms | 0.94x | 10,094 B |
| `suffix_array` | `SuffixArray.new` (`n = 200,000`) + `distinct` + 200k `lcp_between` | 49.3 ms | 47.3 ms | 1.04x | 19,678 B |
| `suffix_automaton` | `SuffixAutomaton<Alpha>.of` (`n = 200,000`) + `occurrences` + `distinct` | 43.2 ms | 46.2 ms | 0.93x | 9,654 B |
| `palindromic_tree` | `PalindromicTree<Alpha>.of` (`n = 1,000,000`) + `occurrences` + `distinct` | 21.9 ms | 22.5 ms | 0.97x | 6,937 B |
| `aho_corasick` | `AhoCorasick<Alpha>.new` (20k x 10) + 500k `step`/`hit` | 23.6 ms | 24.1 ms | 0.98x | 7,075 B |
| `trie` | Binary trie (`n = 25,000`, `L = 25`, `struct Trie(cnt: Int, kid: [0..2]?Int)`) | 39.1 ms | 36.8 ms | 1.06x | 10,817 B |
| `manacher_lyndon` | `manacher` + `lyndon_factorization` + `min_rotation` on `n = 500,000` | 17.8 ms | 17.7 ms | 1.00x | 7,284 B |

Note: `rolling_hash` dropped from 1.61x to 1.16x with `unchecked:` behind entry asserts in `new`/`raw_slice`/`eq_sub`, and to 0.88x by halving 64-bit divisions in `eq_sub` via a single-mod difference check `((p[a+len] - p[b+len]) - (p[a] - p[b]) * pow[len]) % M == 0` (bounded by `M^2 + M < 1.01e18 < 2^63`).
