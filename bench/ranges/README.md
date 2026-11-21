# Range Structures Benchmarks

Matched-algorithm CP Lang vs C++17 (`g++ -std=c++17 -O2`) benchmarks for the
range-structure standard library (`std.fenwick`, `std.lazy`, `std.sparse`,
`std.beats`, `std.persistent`).

Run with `./bench/ranges/run.sh`.

## Measured results (x86_64, `g++ -std=c++17 -O2`, best of 3)

| Benchmark | Problem size | CP Lang | Matched C++ | Ratio | Emitted C++ |
| --- | --- | --- | --- | --- | --- |
| `range_fenwick` | n=200k, 200k range adds + 200k range sums | 26.7 ms | 25.1 ms | 1.06x | 8,359 B |
| `disjoint_sparse` | n=100k build + 500k O(1) sum queries | 18.9 ms | 17.2 ms | 1.09x | 7,901 B |
| `dual_segtree` vs `lazy_segtree` | n=100k, 200k range adds + 200k point gets | 71.4 ms | 94.1 ms (`LazySegTree`) | 0.76x (1.32x speedup) | 10,938 B |
| `beats` | n=50k, 100k mixed `chmin`/`chmax`/`add`/`sum` | 47.0 ms | — | — | 17,067 B |
| `range_kth` | n=50k build + 100k O(log n) median queries | 76.4 ms | — | — | 11,721 B |

Both `RangeFenwick` (1.06x) and `DisjointSparseTable` (1.09x, 0 `std::function`,
flat single-allocation table + return-tuple move) hit the 1.1x target against
matched hand-written C++17.
