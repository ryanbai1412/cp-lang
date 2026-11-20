# A local library

What a contestant's own library looks like: a folder with a `lib.toml`,
one module per file, tests beside the code, and solutions that import
from it. It holds things the standard library leaves out, written on top
of it.

```text
examples/local-lib/
  lib.toml                    name = "lib"
  ds/                         data structures
  math/                       numbers, algebra, transforms
  graph/                      graph decompositions, flow, 2-SAT
  geom/                       computational geometry
  grid/                       grid extensions
  dp/                         DP optimizations
  spec/                       guessing structure from a brute force
  seq/                        folds and order statistics over lists
  str/                        strings
  solutions/road_reversal.cp  a problem solved with the library
```

```text
$ python3 proto/cp.py test examples/local-lib
$ printf '3 2\n1 2\n2 3\n' | python3 proto/cp.py lib-run examples/local-lib/solutions/road_reversal.cp
YES
[0, 1, 0]
```

## What is in it, and why it is not in `std`

The standard library holds what most contests need most of the time. The
locked **CP Lang 0.2.2** architecture (`research/language-architecture-v2.2.md`)
keeps `std` small enough to know by heart by placing specialized
structures and algorithms — including the 11 0.2.2 relocations (`MonoDeque`,
`Steps`, `TwoSat`, `Mat`, `palindromes`, `shortest`/`walk_back`, `mst`,
`convex_hull`, `Dir8`, and `MinCostFlow`) plus the T13 library structures —
in a local library like this one. Each module below is one file with its
tests.

Data structures, in `ds/`:

- `mono_deque.cp`: `MonoDeque<T>`, monotone sliding-window minimum or
  maximum queue (`push`, `drop_before`, `best`).
- `steps.cp`: `Steps`, range additions over arbitrary integer coordinates
  read back as maximal constant runs.
- `rope.cp`: `Rope<T>`, a sequence with insert, erase, reverse and move of
  any stretch in O(log n) (an implicit treap).
- `range_kth.cp`: `RangeKth<D>`, the k-th smallest value and
  `count_below` on any range of a fixed array (a persistent segment tree).
- `li_chao.cp`: `LiChao` and `Line`, `LiChaoTag<V>` / `TaggedLine<V>` (with
  record/index payloads), and continuous `Float` `LiChaoF` / `LineF`.
- `li_chao_arena.cp`: `LiChaoArena` and `Hull`, many Li Chao trees in
  one pool of nodes: a hull per vertex or per DP layer, lines limited to
  a segment, every past version kept, and a range as wide as an `Int`.
- `tri.cp`: `Tri<D>`, a pair `l <= r` as an index, so `[Tri<D>]T` is an
  interval or symmetric table with no unused half.
- `potential_dsu.cp`: `PotentialDsu<V>`, union-find that knows
  `pot(b) - pot(a)` inside a component and refuses a contradiction.
- `bipartite_dsu.cp`: `BipartiteDsu<V>`, union-find with a side per
  vertex: add edges, learn when the graph stops being two-colourable, and
  take edges back with `mark` / `undo_to`.
- `cover.cp`: `Cover<Y>`, the covered length under adding and removing
  intervals, for area-of-union sweeps.
- `two_stack_fold.cp`: `TwoStackFold<T>`, a queue that knows its fold for
  any monoid: the sliding window for `gcd`, `|` or matrices.
- `mo.cp`: `mo_order` and `steps`, Mo's algorithm as the visiting order
  plus the single moves between two windows.
- `xor_basis.cp`: `XorBasis`, a linear basis over GF(2).
- `interner.cp`: `Interner<K>`, bidirectional key <-> dense integer id interner.
- `rollback_array.cp`: `RollbackArray<D, T>`, `[D]T` with O(1) point writes and
  O(changes) snapshot/rollback.

Numbers, in `math/`:

- `frac.cp`: `Frac`, exact fractions that sort and round correctly.
- `fixed.cp`: `Fixed<D>`, exact decimals with `D` digits after the
  point, for money and coordinates that must not become a `Float`.
- `congruence.cp`: `Congruence`, a residue class that solves
  `a * x = b (mod m)` and combines with others when the moduli share
  factors (the general Chinese remainder theorem).
- `hom.cp`: `Hom<T>`, affine maps under composition, and `BothWays<T>`,
  the `Reversible` value that lets `Rooted.fold_path` fold them along a
  tree path.
- `matrix.cp`: `Mat<P>`, square matrices over `Mod<P>` with `det`, `rank`,
  `solve`, and rectangular `solve_linear`.
- `poly.cp`: `ntt`, `convolve` and `pow_trunc` modulo 998244353, plus
  arbitrary-modulus 3-prime MTT `convolve_mod` and Lagrange `interpolate`.
- `powers.cp`: `Powers<P>`, the powers of one base in O(1).
- `sos.cp`: `sos_sub`, `sos_sup` and `sos_sub_inv`, sums over subsets and
  supersets of bitmasks.

Graphs, in `graph/`:

- `shortest.cp`: `shortest` (Dijkstra with predecessor tracking) and
  `walk_back` (path reconstruction).
- `mst.cp`: `mst`, Kruskal's minimum spanning forest.
- `two_sat.cp`: `TwoSat<X>` and `Lit<X>`, 2-SAT over a typed variable
  domain backed by `std.graph.scc`.
- `min_cost_flow.cp`: `MinCostFlow<V>` and `CostArc`, successive shortest
  paths with Johnson potentials (SPFA initialization + Dijkstra).
- `functional.cp`: `Functional<V>`, the graph where every vertex has one
  way out, split into its rings and the trees draining into them, with
  `jump`, `dist`, and `jump_while`.
- `centroids.cp`: `Centroids<V>` centroid decomposition and `reroot` all-roots
  tree DP (no inverse required).
- `kruskal_tree.cp`: `KruskalTree<V>` (Kruskal reconstruction / bottleneck
  tree) and `spanning` forest selector.
- `min_count.cp`: `MinCount<P>` shortest-path distance + count semiring and
  `floyd_warshall`.

Geometry and grids, in `geom/` and `grid/`:

- `convex_hull.cp`: `convex_hull`, Andrew's monotone chain over
  `std.geometry.Pt`.
- `dir8.cp`: `Dir8`, eight-way grid directions compatible with
  `std.grid.{step, ray}`.

DP optimizations, in `dp/`:

- `dnc_opt.cp`: `monotone_minima` and `dnc_layer`, divide and conquer
  over a monotone best column.
- `knuth.cp`: `knuth`, interval DPs in O(n^2).

Guessing structure from a brute force, in `spec/` (on top of `std.spec`):

- `recurrence.cp`: `integer_recurrence` and `rational_recurrence` (exact
  Berlekamp-Massey over fractions), `mod_recurrence`, `recurrence_break`,
  `extend`, `nth_term` and `show_recurrence`.
- `interp.cp`: `poly_degree_mod`, `lagrange_at` (O(k) on consecutive
  points), `exact_poly` and `closed_form` with fraction coefficients, and
  `show_poly`.
- `structure.cp`: brute-force `split_dp` and `interval_dp` with
  `split_opt_break`, `aliens_break`, `interval_qi_break`, `inclusion_break`
  and `interval_opt_break`; `split_report` and `interval_report` name the
  optimization that fits; `rolling_depth`, `rolling_window`, `period` and
  `periodic_at`.
- `worked.cp`: five contest problems taken from brute force to structure.

```cp
let small = [tilings(n) for n in 0..12]      // brute force: 1, 0, 3, 0, 11, ...
let c = integer_recurrence(small)!
show_recurrence(c)                           // "a[n] = 4 a[n-2] - a[n-4]"
nth_term::<1000000007>(c, small, 10 ** 18)   // the answer for the real limits
```

Sequences and strings, in `seq/` and `str/`:

- `fold_others.cp`: `fold_others` (for every position, the fold of all
  the others, with no inverse needed) and `fold_balanced`.
- `order_stats.cp`: `ranks`, `lis` (strict and non-decreasing, with index
  reconstruction), and offline 2D Fenwick `count_in_rects`.
- `palindromes.cp`: `palindromes` (Manacher's linear-time odd and even
  palindrome radii).
- `borders.cp`: `borders`, `period` and `overlaps`.
- `trie.cp`: `Trie<A>` over an alphabet that is a domain.
- `suffix_automaton.cp`: `SuffixAutomaton<A>`, built online.
- `hash2d.cp`: `Hash2D<R, C>`, the hash of any rectangle of a grid.

Union-find with undo lives in `std.dsu` (`Dsu.mark()` / `Dsu.undo_to(snap)`),
while `TwoSat` lives here in `graph/two_sat.cp` per the 0.2.2 stdlib split.

## The five T13 structures, by example

These are the structures the v2 design names for a local library and that
`std` will never hold. Each is shown the way a solution would use it.

`Tri<D>` is an index, not a container. It implements `Index` with
`K = D.K * (D.K + 1) / 2`, so the table type is the ordinary `[Tri<I>]T`
and an interval DP stores exactly the cells with `l <= r`. `of_len` gives
the intervals in the order the DP fills them, and `splits` the two parts
of every cut:

```
use lib.ds.tri.{Tri}

domain I = 0..n
var dp: [Tri<I>]Int = 0
for len in 2..=n:
    for t in Tri<I>.of_len(len):
        dp[t] = min(dp[x] + dp[y] for (x, y) in t.splits) + sum(a[i] for i in t.l..=t.r)
print(dp[Tri<I>(I.first, I.last)])
```

For a symmetric table, `Tri<V>.sym(a, b)` is the same cell as
`Tri<V>.sym(b, a)`.

`Fixed<D>` keeps a decimal as an `Int` counting units of `10 ** -D`.
`parse` takes `12.5` and `12.50` as the same value and returns `none` for
text it would have to round; printing always shows `D` digits. `+`, `-`
and comparisons are exact; `*` and `/` round down to `D` digits:

```
use lib.math.fixed.{Fixed}

type Money = Fixed<2>
let total = Money.parse("10")!
let share = total / Money.of(3)          // 3.33
let left = total - share.times(3)        // 0.01
print("{share} each, {left} left over")
```

`Congruence` is one residue class `x = r (mod m)`. `meet` intersects two
of them and is `none` when they contradict, whatever the moduli have in
common; `all` folds a whole system; `first` and `count` then answer the
usual questions about the solutions:

```
use lib.math.congruence.{Congruence}

// gear i shows its mark every period[i] ticks, first at tick start[i]
let sys = Congruence.all([Congruence.new(3, 4), Congruence.new(1, 6), Congruence.new(7, 9)])
if sys == none: print("never")
else: print(sys!.first(at_least: 7))     // 7, and again every sys!.m = 36 ticks

let x = Congruence.solve(6, 4, 10)!      // 6 * x = 4 (mod 10): x = 4 (mod 5)
```

`Functional<V>` takes the array of successors. `rings` lists every cycle,
`peel_order` lists the tree vertices with each one before the vertex it
points at, and `jump` and `dist` cost O(log n) even for `k = 10 ** 18`,
because past the tree only the position on the ring matters:

```
use lib.graph.functional.{Functional}

let g = Functional<V>(next)
var load: [V]Int = 1
for v in g.peel_order: load[next[v]] += load[v]      // fold every tree into its ring
for ring in g.rings: print(sum(load[v] for v in ring))
print(g.jump(start, k))                               // where the walk is after k steps
print(g.dist(a, b))                                   // none if a never reaches b
```

`LiChaoArena` is `LiChao` with the tree taken out of the structure: the
arena owns the nodes, and a `Hull` is a handle to one set of lines in it.
`add` works in place and makes at most one node. `added` leaves the old
hull valid and returns a new one, which is what a hull per tree vertex
needs: each child extends its parent's lines without disturbing its
siblings.

```
use lib.ds.li_chao.{Line}
use lib.ds.li_chao_arena.{LiChaoArena}

var pool = LiChaoArena(-1_000_000_000, 1_000_000_000)
var hull = [V] _ => pool.empty
for v in order:                                       // parents before children
    if parent[v] != none:
        dp[v] = pool.min_at(hull[parent[v]!], h[v])! + h[v] * h[v]
        hull[v] = hull[parent[v]!]
    hull[v] = pool.added(hull[v], Line(-2 * h[v], dp[v] + h[v] * h[v]))
```

`add_on(hull, from, to, line)` limits a line to a segment, `merge` pours
one hull into another for small-to-large, and `clear` empties the whole
pool in one call between test cases.

## Importing

The name in `lib.toml` is the root of every import. A file anywhere below
the folder finds it by walking up, so a solution needs no configuration:

```
use lib.ds.bipartite_dsu.{BipartiteDsu}

let (m, k): (Int, Int) = read()
domain Road = 1..=m
var d = BipartiteDsu<Road>()
for _ in 0..k:
    let (a, b): (Int, Int) = read()
    d.add_edge(Road(a), Road(b))
if d.bipartite:
    print("YES")
    print([if d.find(r).1: 1 else: 0 for r in Road])
else:
    print("NO")
```

A library module imports the standard library the same way a solution
does, and takes what `std` takes. `two_stack_fold.cp` accepts a bare
operator because `Monoid<T>.of` does the work:

```
use std.algebra.{Monoid}

pub fn new<M>(op: M) -> TwoStackFold<T>:
    let m = Monoid<T>.of(op)
    TwoStackFold(m, [], m.id, [])

var q = TwoStackFold<Int>(gcd)
```

## The conventions it follows

Only `pub` items are visible to an importer; helpers stay private to
their file. A module holds declarations only, so importing it cannot run
code.

Structures are indexed by a domain. `PotentialDsu<V>` takes vertices,
`Trie<A>` takes letters of its alphabet, `Cover<Y>` takes ranges of its
cells. A structure whose positions shift with every edit (`Rope`) is the
exception, and says so: its positions are plain `Int`s.

Absence is `none`, not a sentinel. `LiChao.min_at` returns `?Int`
because a tree with no lines has no minimum; `RangeKth.kth` returns
`none` when the range is too short; `PotentialDsu.diff` returns `none`
for two vertices nothing relates.

A type with arithmetic implements the operator traits and then works
everywhere a number does. `Mat<P>` implements `Mul`, so `pow` is the
usual square-and-multiply, and `Mat<P>.product(n)` packages it as a
`Monoid` for a segment tree of matrices. `Frac` implements `Ord`, so
`sort`, `min` and `max` take it as they are.

A structure with no invariant between its cells may lend them.
`Mat<P>.at(i, j)` is a projection (`lends mut Mod<P>`), so
`m.at(i, j) += x` writes into the matrix and nothing is copied. The
structures that keep an invariant (a union-find, a Li Chao tree) expose
operations instead, so no caller can break them.

Every test checks against something simpler: `rope.cp` replays random
edits on a plain list, `range_kth.cp` sorts each range, `poly.cp`
multiplies the schoolbook way, `suffix_automaton.cp` counts substrings by
enumeration, `knuth.cp` runs the cubic DP. Most files end with one small
problem solved with the structure, which is the quickest way to see how
it is meant to be used.

## Starting your own

Copy the folder, keep `lib.toml`, and delete what you do not use. To use
a library that lives elsewhere, name it in the project's `cp.toml`:

```toml
[libs]
lib = "../my-cp-lib"
```
