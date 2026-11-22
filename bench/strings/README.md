# Strings stdlib benchmarks

Matched CP Lang (`std.strings`) vs hand-written C++17 (`g++ -std=c++17 -O2`, best of 5 runs, outputs verified equal).

Run: `bash bench/strings/run.sh`

| Benchmark | Workload | CP Lang | C++17 | Ratio | Emitted C++ |
| --- | --- | ---: | ---: | ---: | ---: |
| `z_prefix` | `z_function` + `prefix_function` on `n = 1,000,000` | 26.6 ms | 23.9 ms | 1.11x | 4,579 B |
| `rolling_hash` | `RollingHash.new` (`n = 500,000`) + 200k `slice`/`lcp` | 25.0 ms | 42.7 ms | 0.58x | 13,403 B |
| `hash2d` | `Hash2D<R, C>.new` (`500 x 500`) + 500k `rect` | 16.5 ms | 18.7 ms | 0.88x | 15,453 B |
| `suffix_array` | `SuffixArray.new` (`n = 200,000`) + `distinct` + 200k `lcp_between` | 35.7 ms | 42.9 ms | 0.83x | 18,656 B |
| `suffix_automaton` | `SuffixAutomaton<Alpha>.of` (`n = 200,000`) + `occurrences` + `distinct` | 28.1 ms | 39.0 ms | 0.72x | 10,646 B |
| `palindromic_tree` | `PalindromicTree<Alpha>.of` (`n = 1,000,000`) + `occurrences` + `distinct` | 19.8 ms | 20.7 ms | 0.96x | 7,845 B |
| `aho_corasick` | `AhoCorasick<Alpha>.new` (20k x 10) + 500k `step`/`hit` | 16.9 ms | 20.7 ms | 0.81x | 7,998 B |
| `trie` | Binary trie (`n = 25,000`, `L = 25`, `struct Trie(cnt: Int, kid: [0..2]?Int)`) | 35.1 ms | 32.4 ms | 1.08x | 10,535 B |
| `manacher_lyndon` | `manacher` + `lyndon_factorization` + `min_rotation` on `n = 500,000` | 16.0 ms | 14.4 ms | 1.11x | 6,995 B |
| `lyndon_runs` | `lyndon_runs` (Bannai runs theorem) on `n = 100,000` | 44.0 ms | 51.2 ms | 0.86x | 20,177 B |

Summary (10 matched benchmarks, covering every `std.strings` algorithm and structure): **p50 = 0.87x** (target <= 1.10x), **max = 1.11x** (target <= 1.50x).

Notes:
- `rolling_hash` dropped from 1.61x to 0.58x (25.0 vs 42.7 ms) via `unchecked:` behind entry asserts, single-mod `eq_sub`, and borrowing immutable non-scalar captures (`self`) by reference in non-escaping closures (`bsearch_last`).
- `hash2d` dropped from 1.66x (43.4 ms) to 0.88x (16.5 vs 18.7 ms) by flattening `pre1`/`pre2` to 1D with `stride = cols + 1`, precomputing `brc = br * bc % M`, and narrowing prefix/power tables to `List<0..=1_000_000_009>` (32-bit storage).
- `suffix_array` dropped from 1.10x (51.3 ms) to 0.83x (35.7 vs 42.9 ms) by preserving `Cast::Raw` across `settle_indexes`/`freeze_place` and narrowing `rank`/`rmq` to 32-bit.
- `z_prefix` and `manacher_lyndon` both emit zero bounds checks inside `std.strings`; the remaining ~2 ms (1.11x) is non-library top-level global-list checks and `pthread_create` startup (logged in `research/frictions/bench-strings.md`).
