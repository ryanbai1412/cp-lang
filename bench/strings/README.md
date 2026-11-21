# Strings stdlib benchmarks

Matched CP Lang (`std.strings`) vs hand-written C++17 (`g++ -std=c++17 -O2`, best of 5 runs, outputs verified equal).

Run: `bash bench/strings/run.sh`

| Benchmark | Workload | CP Lang | C++17 | Ratio | Emitted C++ |
| --- | --- | ---: | ---: | ---: | ---: |
| `z_prefix` | `z_function` + `prefix_function` on `n = 1,000,000` | 26.5 ms | 24.4 ms | 1.09x | 4,950 B |
| `rolling_hash` | `RollingHash.new` (`n = 500,000`) + 200k `slice`/`lcp` | 52.5 ms | 42.7 ms | 1.23x | 11,278 B |
| `suffix_array` | `SuffixArray.new` (`n = 200,000`) + `distinct` + 200k `lcp_between` | 46.4 ms | 44.4 ms | 1.05x | 20,409 B |
| `suffix_automaton` | `SuffixAutomaton<Alpha>.of` (`n = 200,000`) + `occurrences` + `distinct` | 41.2 ms | 42.3 ms | 0.97x | 10,483 B |
| `aho_corasick` | `AhoCorasick<Alpha>.new` (20k x 10) + 500k `step`/`hit` | 22.2 ms | 21.4 ms | 1.04x | 7,945 B |

Note: `rolling_hash` is 1.23x (vs 1.61x before stdlib tuning) due to two codegen gaps filed with the codegen lane: `const` integer globals emitting as mutable `ll` instead of `constexpr ll` (forcing hardware `idiv` on `%`) and `cpl::at` bounds checks inside `RollingHash.eq_sub`.
