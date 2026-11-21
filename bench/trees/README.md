# `std.tree` against matched C++

`tree_bench.cp` and `tree_bench.cpp` run the same algorithms: an iterative
DFS rooting with heavy-first preorder, `reroot` heights with prefix and
suffix sibling folds, chain-jumping `lca` + `dist` for every query, and an
iterative centroid decomposition read back through `ancestors`. Both print
the same three checksums. Commands: `bench/README.md`.

## Results

n = q = 10^6, `g++ -std=c++17 -O2`, best of three runs, outputs identical.
Seed 1 attaches each vertex to a uniform earlier one (depth about log n);
seed 2 to one of the previous ten (long chains).

| input | C++ | CP Lang | ratio | C++ RSS | CP Lang RSS |
|---|---:|---:|---:|---:|---:|
| seed 1 | 2.67 s | 3.99 s | 1.49x | 520 MB | 1110 MB |
| seed 2 | 3.54 s | 5.32 s | 1.50x | 706 MB | 1621 MB |

With the breadth-first build (decisions.md §86) seed 1 is 3.45 s and 677 MB
against 2.70 s and 520 MB (1.28x).

This misses the 1.1x target. Rooting, rerooting and the queries are within
about 1.5x; the centroid decomposition is most of the gap.

## Where the gap is

Measured by editing the emitted C++ of the centroid phase alone (seed 1):

| change | time | RSS |
|---|---:|---:|
| emitted as is | 2.95 s | 998 MB |
| move the built arrays into the returned struct | 2.72 s | 611 MB |
| and store the centroid distance in 32 bits | 2.15 s | 427 MB |

Both are core features, not library changes:
- A struct literal built from local arrays copies each array instead of
  moving it (every `Rooted` and `Centroids` constructor pays a full copy).
- `Int` is 64-bit and fixed widths (`I32`) are not implemented yet, so the
  per-ancestor distance cannot be narrowed from the library.

`Rooted.new` alone (build and read depth and size, seed 1) is 0.65 s and
269 MB against 0.49 s and 158 MB; moving the arrays into the struct by hand
brings it to 187 MB, and the rest is 64-bit `dep`, `far` and `sz`.

`ancestors` already yields instead of building a list (decisions.md §45.9).

## Building a `Rooted`

`root_bench.cp` and `root_bench.cpp` root a tree from its edge list and run one
bottom-up subtree sum. The C++ builds CSR adjacency, a BFS order and everything
`Rooted` keeps (parent, depth, size, heavy child, chain top, heavy-first
preorder and its inverse). Input: `python3 gen.py 1000000 0 1 | head -n -1`.

n = 10^6, best of three, outputs identical (seed 2 within 0.03 s):

| version | time | RSS |
|---|---:|---:|
| C++ | 0.27 s | 70 MB |
| depth-first stack, one child list per vertex | 0.63 s | 175 MB |
| breadth-first, flat child ranges (decisions.md §86) | 0.45 s | 178 MB |
| the same, edges read as `[E](u: V, v: V)` over `domain E` | 0.30 s | 138 MB |
| `Tree<V>` edges, `Graph.undirected` building CSR `Adj` (decisions.md §110) | 0.32 s (C++ 0.39 s, loaded box) | 79 MB |

The edge list as `Tree<V>` (`[0..n - 1]`) names no edge domain, so
`Graph.undirected` builds `Adj`, the same compressed rows as `Graph` holding the
neighbours themselves, with 32-bit offsets; it used to build one `List` per
vertex (0.45 s, 178 MB). The last row was measured later than the others on a
busier machine, so compare it with the C++ run beside it: 0.82x the time and
79 MB against 70 MB (`research/frictions/rooted-build.md` has the remaining
storage gaps).

## `LinkCut`

`lct_bench.cp` and `lct_bench.cpp` run the same splay-based link-cut tree
(path sums, re-rooting by a reversal flag, an explicit push stack) on
`python3 lct_gen.py 200000 1000000 1`: n = 2 * 10^5 vertices and 10^6
operations, a third each of `set`, cut-and-relink and `fold_path`.

Both check what `std.tree` checks: `link` traps when the vertices are already
connected (one `root_of`, as KACTL's `assert(!connected(u, v))`) and `cut`
traps when the edge is missing.

| C++ | CP Lang | ratio | C++ RSS | CP Lang RSS |
|---:|---:|---:|---:|---:|
| 1.17 s | 1.18 s | 1.01x | 9 MB | 47 MB |

Best of seven, outputs identical. The generated splay, rotate and pull match
the C++ line for line (optional links are `-1`-niche ints, the monoid op is a
direct call); without the `link` check the C++ runs in 0.94 s, and so does
the generated C++ with that one call deleted. The RSS gap is CP Lang reading
the 21 MB input in one buffer.

`lazy_lct_bench.{cp,cpp}` replace `set` by `apply_path(u, v, x)` (add `x` along
the path, both with a subtree count per splay node) on `python3 lct_gen.py
200000 1000000 SEED lazy`. Best of three, outputs identical:

| seed | C++ | CP Lang | C++ RSS | CP Lang RSS |
|---:|---:|---:|---:|---:|
| 1 | 2.11 s | 1.82 s | 11 MB | 21 MB |
| 2 | 2.12 s | 2.05 s | 11 MB | 18 MB |

The C++ reads with `scanf`, as in `lct_bench.cpp`; the generated splay and
push match it.

## `Rooted.virtual`

`virtual_bench.cp` and `virtual_bench.cpp` build the tree as `root_bench` does,
then per query build the virtual tree of k random keys (sort by preorder,
dedupe, add adjacent LCAs, sort and dedupe, link with a stack) and sum all
pairwise key distances in one bottom-up pass. Input:
`python3 virtual_gen.py n q k seed`. Best of seven, outputs identical:

| input | C++ | CP Lang | ratio | C++ RSS | CP Lang RSS |
|---|---:|---:|---:|---:|---:|
| n = 2·10^5, q = 10^5, k = 10, seed 1 | 0.223 s | 0.231 s | 1.04x | 14 MB | 18 MB |
| the same, seed 2 (long chains) | 0.164 s | 0.180 s | 1.10x | 14 MB | 18 MB |
| n = 10^6, q = 10^3, k = 10^3, seed 2 | 0.444 s | 0.401 s | 0.90x | 56 MB | 80 MB |

`virtual` builds its key list and candidate list with `List.reserve`, so
neither a comprehension copy nor a growth reallocation runs per query; that
took the small-k cases from 1.12x and 1.18x to the numbers above.

## `LinkCut`

`lct_bench.cp` and `lct_bench.cpp` run the same splay-based link-cut tree
(path sums, re-rooting by a reversal flag, an explicit push stack) on
`python3 lct_gen.py 200000 1000000 1`: n = 2 * 10^5 vertices and 10^6
operations, a third each of `set`, cut-and-relink and `fold_path`.

Both check what `std.tree` checks: `link` traps when the vertices are already
connected (one `root_of`, as KACTL's `assert(!connected(u, v))`) and `cut`
traps when the edge is missing.

| C++ | CP Lang | ratio | C++ RSS | CP Lang RSS |
|---:|---:|---:|---:|---:|
| 1.17 s | 1.18 s | 1.01x | 9 MB | 47 MB |

Best of seven, outputs identical. The generated splay, rotate and pull match
the C++ line for line (optional links are `-1`-niche ints, the monoid op is a
direct call); without the `link` check the C++ runs in 0.94 s, and so does
the generated C++ with that one call deleted. The RSS gap is CP Lang reading
the 21 MB input in one buffer.

`lazy_lct_bench.{cp,cpp}` replace `set` by `apply_path(u, v, x)` (add `x` along
the path, both with a subtree count per splay node) on `python3 lct_gen.py
200000 1000000 SEED lazy`. Best of three, outputs identical:

| seed | C++ | CP Lang | C++ RSS | CP Lang RSS |
|---:|---:|---:|---:|---:|
| 1 | 2.11 s | 1.82 s | 11 MB | 21 MB |
| 2 | 2.12 s | 2.05 s | 11 MB | 18 MB |

The C++ reads with `scanf`, as in `lct_bench.cpp`; the generated splay and
push match it.

## `Rooted.virtual`

`virtual_bench.cp` and `virtual_bench.cpp` build the tree as `root_bench` does,
then per query build the virtual tree of k random keys (sort by preorder,
dedupe, add adjacent LCAs, sort and dedupe, link with a stack) and sum all
pairwise key distances in one bottom-up pass. Input:
`python3 virtual_gen.py n q k seed`. Best of seven, outputs identical:

| input | C++ | CP Lang | ratio | C++ RSS | CP Lang RSS |
|---|---:|---:|---:|---:|---:|
| n = 2·10^5, q = 10^5, k = 10, seed 1 | 0.24 s | 0.27 s | 1.12x | 14 MB | 22 MB |
| the same, seed 2 (long chains) | 0.17 s | 0.20 s | 1.18x | 14 MB | 22 MB |
| n = 10^6, q = 10^3, k = 10^3, seed 2 | 0.51 s | 0.46 s | 0.90x | 58 MB | 98 MB |

The small-k gap is per-query allocation: CP Lang allocates the four `[vt.K]`
arrays and the sort buffers for every query where the C++ reuses them. RSS is
the 64-bit `[V]Int` tables of `Rooted` (`research/frictions/rooted-build.md`).
