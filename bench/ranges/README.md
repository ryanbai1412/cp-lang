# Range Structures Benchmarks

Matched-algorithm CP Lang vs C++17 (`g++ -std=c++17 -O2`) benchmarks for the
range-structure standard library (`std.fenwick`, `std.lazy`, `std.sparse`,
`std.beats`, `std.persistent`, `std.segtree`, `std.timetree`).

Run with `./bench/ranges/run.sh`.

## Measured results (x86_64, `g++ -std=c++17 -O2`, best of 3)

| Benchmark | Problem size | CP Lang | Matched C++ | Ratio | Emitted C++ |
| --- | --- | --- | --- | --- | --- |
| `disjoint_sparse` | n=100k build + 500k O(1) sum queries | 17.5 ms | 18.2 ms | 0.96x | 7,408 B |
| `lazy_treap` | n=200k slope-trick inserts + global affine tags + `take(n/2).sum()` | 83.5 ms | 82.2 ms | 1.02x | 12,332 B |
| `range_fenwick` | n=200k, 200k range adds + 200k range sums | 26.0 ms | 24.4 ms | 1.06x | 8,138 B |
| `merge_sort_tree` | n=50k build + 100k O(log^2 n) `count_le` queries | 119.1 ms | 112.9 ms | 1.06x | 9,940 B |
| `beats` | n=50k, 100k mixed `chmin`/`chmax`/`add`/`sum` | 164.5 ms | 148.0 ms | 1.11x | 17,536 B |
| `range_kth` | n=50k build + 100k O(log n) median queries | 62.9 ms | 54.1 ms | 1.16x | 12,840 B |
| `time_tree` | n=30k, m=60k edge intervals, q=30k rollback-DSU queries | 50.8 ms | 41.7 ms | 1.22x | 13,680 B |
| `dual_segtree` vs `lazy_segtree` | n=100k, 200k range adds + 200k point gets | 71.4 ms | 94.1 ms (`LazySegTree`) | 0.76x (1.32x speedup) | 10,938 B |

All seven matched range benchmarks are at p50 1.06x (max 1.22x) against
hand-written C++17 with zero `std::function`: `MergeSortTree` uses a single flat
`data` array with `[beg, fin)` offsets, `RangeKth` uses 32-bit ranged-int node
indices (`0..=2_000_000_000`) and iterative path copy, and `TimeTree` uses a
flat linked-list pool (`head`/`next`/`ev_id`) with iterative interval insertion.
