# Strings stdlib benchmarks

Matched CP Lang (`std.strings`) vs hand-written C++17 (`g++ -std=c++17 -O2`, best of 5 runs, outputs verified equal).

Run: `bash bench/strings/run.sh`

| Benchmark | Workload | CP Lang | C++17 | Ratio | Emitted C++ |
| --- | --- | ---: | ---: | ---: | ---: |
| `z_prefix` | `z_function` + `prefix_function` on `n = 1,000,000` | 29.1 ms | 29.6 ms | 0.98x | 4,662 B |
| `rolling_hash` | `RollingHash.new` (`n = 500,000`) + 200k `slice`/`lcp` | 47.1 ms | 50.3 ms | 0.94x | 9,798 B |
| `suffix_array` | `SuffixArray.new` (`n = 200,000`) + `distinct` + 200k `lcp_between` | 51.3 ms | 46.7 ms | 1.10x | 18,935 B |
| `suffix_automaton` | `SuffixAutomaton<Alpha>.of` (`n = 200,000`) + `occurrences` + `distinct` | 48.3 ms | 46.2 ms | 1.05x | 9,494 B |
| `palindromic_tree` | `PalindromicTree<Alpha>.of` (`n = 1,000,000`) + `occurrences` + `distinct` | 20.9 ms | 22.0 ms | 0.95x | 6,777 B |
| `aho_corasick` | `AhoCorasick<Alpha>.new` (20k x 10) + 500k `step`/`hit` | 23.1 ms | 22.1 ms | 1.05x | 6,970 B |
| `trie` | Binary trie (`n = 25,000`, `L = 25`, `struct Trie(cnt: Int, kid: [0..2]?Int)`) | 38.7 ms | 37.4 ms | 1.03x | 10,521 B |
| `manacher_lyndon` | `manacher` + `lyndon_factorization` + `min_rotation` on `n = 500,000` | 17.3 ms | 16.1 ms | 1.07x | 6,988 B |
| `lyndon_runs` | `lyndon_runs` (Bannai runs theorem) on `n = 100,000` | 58.4 ms | 59.7 ms | 0.98x | 20,294 B |

Note: `rolling_hash` dropped from 1.61x to 1.16x with `unchecked:` behind entry asserts in `new`/`raw_slice`/`eq_sub`, and to 0.88x by halving 64-bit divisions in `eq_sub` via a single-mod difference check `((p[a+len] - p[b+len]) - (p[a] - p[b]) * pow[len]) % M == 0` (bounded by `M^2 + M < 1.01e18 < 2^63`).
