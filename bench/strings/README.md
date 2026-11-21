# Strings stdlib benchmarks

Matched CP Lang (`std.strings`) vs hand-written C++17 (`g++ -std=c++17 -O2`, best of 5 runs, outputs verified equal).

Run: `bash bench/strings/run.sh`

| Benchmark | Workload | CP Lang | C++17 | Ratio | Emitted C++ |
| --- | --- | ---: | ---: | ---: | ---: |
| `z_prefix` | `z_function` + `prefix_function` on `n = 1,000,000` | 26.6 ms | 25.3 ms | 1.05x | 4,950 B |
| `rolling_hash` | `RollingHash.new` (`n = 500,000`) + 200k `slice`/`lcp` | 41.2 ms | 46.7 ms | 0.88x | 10,094 B |
| `suffix_array` | `SuffixArray.new` (`n = 200,000`) + `distinct` + 200k `lcp_between` | 46.7 ms | 43.5 ms | 1.07x | 19,539 B |
| `suffix_automaton` | `SuffixAutomaton<Alpha>.of` (`n = 200,000`) + `occurrences` + `distinct` | 42.0 ms | 42.5 ms | 0.99x | 9,613 B |
| `aho_corasick` | `AhoCorasick<Alpha>.new` (20k x 10) + 500k `step`/`hit` | 21.0 ms | 20.3 ms | 1.04x | 7,075 B |
| `trie` | Binary trie (`n = 25,000`, `L = 25`, `struct Trie(cnt: Int, kid: [0..2]?Int)`) | 35.4 ms | 33.8 ms | 1.05x | 10,817 B |

Note: `rolling_hash` dropped from 1.61x to 1.16x with `unchecked:` behind entry asserts in `new`/`raw_slice`/`eq_sub`, and to 0.88x by halving 64-bit divisions in `eq_sub` via a single-mod difference check `((p[a+len] - p[b+len]) - (p[a] - p[b]) * pow[len]) % M == 0` (bounded by `M^2 + M < 1.01e18 < 2^63`).
