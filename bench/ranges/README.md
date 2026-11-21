# Range Structures Benchmarks

Matched-algorithm CP Lang vs C++17 (`g++ -std=c++17 -O2`) benchmarks for the
range-structure standard library (`std.fenwick`, `std.lazy`, `std.sparse`,
`std.beats`, `std.persistent`, `std.segtree`, `std.timetree`).

Run with `./bench/ranges/run.sh`.

## Measured results (x86_64, `g++ -std=c++17 -O2`, best of 3)

| Benchmark | Problem size | CP Lang | Matched C++ | Ratio | Emitted C++ |
| --- | --- | --- | --- | --- | --- |
| `range_fenwick` | n=200k, 200k range adds + 200k range sums | 27.7 ms | 26.3 ms | 1.05x | 8,286 B |
| `disjoint_sparse` | n=100k build + 500k O(1) sum queries | 19.6 ms | 18.1 ms | 1.08x | 8,132 B |
| `lazy_treap` | n=200k slope-trick inserts + global affine tags + `take(n/2).sum()` | 87.8 ms | 82.9 ms | 1.06x | 11,303 B |
| `beats` | n=50k, 100k mixed `chmin`/`chmax`/`add`/`sum` | 171.1 ms | 150.4 ms | 1.14x | 17,665 B |
| `merge_sort_tree` | n=50k build + 100k O(log^2 n) `count_le` queries | 139.4 ms | 115.2 ms | 1.21x | 9,804 B |
| `time_tree` | n=30k, m=60k edge intervals, q=30k rollback-DSU queries | 72.7 ms | 42.9 ms | 1.70x | 12,012 B |
| `range_kth` | n=50k build + 100k O(log n) median queries | 92.9 ms | 52.9 ms | 1.76x | 10,762 B |
| `dual_segtree` vs `lazy_segtree` | n=100k, 200k range adds + 200k point gets | 71.4 ms | 94.1 ms (`LazySegTree`) | 0.76x (1.32x speedup) | 10,938 B |

`RangeFenwick` (1.05x), `LazyTreap` (1.06x), `DisjointSparseTable` (1.08x), and
`BeatsSegTree` (1.14x) match hand-written C++17 closely with zero `std::function`.
`RangeKth` (1.76x) and `TimeTree` (1.70x) reflect 64-bit `Int` node/DSU indices
versus 32-bit `int` in the C++ references (plus SoA vs AoS cache locality in
`RangeKth` and `List<List<Int>>` bucket indirection in `TimeTree`).
