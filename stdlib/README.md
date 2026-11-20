# The standard library

Contest data structures and algorithms, written in CP Lang itself. Nothing
here is built into the compiler: a segment tree, a modular number and a
flow network are ordinary `.cp` code on top of the small core (types,
domains, generics, operation values, borrows). If the library can express
it, so can your own.

```text
$ python3 proto/cp.py test stdlib              # every test in every module
$ python3 proto/cp.py test stdlib/segtree.cp   # one module
$ python3 proto/cp.py test stdlib -k lca       # tests whose name contains "lca"
$ python3 proto/cp.py doc stdlib               # the API reference, as Markdown
```

A module is one file, and its tests live in the same file as `@test fn`s.
Most tests compare the structure against a brute force on random inputs.
Every example below is a test in [`tour.cp`](tour.cp), so it runs.

## How it runs today

The Rust compiler does not yet accept generic structs, traits or modules,
so the library runs on a reference interpreter, `proto/cpi/`. It executes
the planned language directly, slowly and with every check on: an index
from the wrong domain, an out-of-range value, a broken algebraic law or a
`none` used as a value stops the test with the source line. The files are
written for the real compiler; when it catches up they should compile
unchanged, and the interpreter stays as the oracle to test it against.

## Operation values: one structure, many behaviours

A structure never assumes what "combine" means. You hand it a value that
says so, and the same tree serves sums, maxima or your own monoid.

```
domain I = 1..=5
let a = [I] i => [3, 1, 4, 1, 5][i.to_idx]
let sums = SegTree<I, Int>.from(Sum<Int>, a)
let highs = SegTree<I, Int>.from(Max<Int>, a)
assert sums.fold(I(2)..=I(4)) == 6
assert highs.fold(I(2)..=I(4)) == 4

// your own monoid is one line: (best, count of best)
let best = Monoid<(Int, Int)>((Int.MIN, 0), (x, y) => if x.0 > y.0: x else: if y.0 > x.0: y else: (x.0, x.1 + y.1))
let t = SegTree<I, (Int, Int)>.from(best, [I] i => (a[i], 1))
assert t.all == (5, 1)
```

`algebra.cp` defines `Semigroup`, `Monoid`, `Group`, `Action` and `Order`,
the ready-made `Sum`, `Prod`, `Max`, `Min`, `Xor`, `Less` and `Greater`,
and the combinators `flip`, `both` and `opt`. Each one states its laws,
and `laws_hold(m, samples)` checks them on sample values, which is how the
tests catch a "monoid" that is not associative.

Each structure asks for exactly what it needs. `Fenwick` wants a `Group`
because it subtracts prefixes; `SegTree` a `Monoid`; `SparseTable` a
`Semigroup` that is idempotent; `LazySegTree` a `Monoid` plus an `Action`.

## Indices know their container

A structure is indexed by a domain, not by `Int`. `Dsu<V>` only accepts
vertices, so passing an edge number is an error at the call, not a wrong
answer three hours later.

```
domain V = 1..=4
domain E = 0..3
var d = Dsu<V>()
for e in E:
    if d.unite(edges[e].u, edges[e].v): joined += 1
assert d.count == 2 and d.same(V(1), V(2))
// `d.find(e)` for an edge index `e` is rejected: an E is not a V
```

The domain also fixes the size. There is no `n` to pass and no `n + 1` to
remember: `Dsu<V>()` has one slot per vertex whether `V` is `0..n` or
`1..=n`.

## Ranges and boundaries

Range queries take a range of the domain, closed or half-open, so there
is no convention to remember. A boundary (`i.l` is the gap before `i`,
`i.r` the gap after) names a prefix without an off-by-one.

```
domain I = 0..6
var f = Fenwick<I, Int>(Sum<Int>)
for i in I: f.add(i, i.to_idx + 1)
assert f.fold(I(1)..=I(3)) == 2 + 3 + 4
assert f.fold(I(1)..I(3)) == 2 + 3
assert f.prefix(I(3).l) == 1 + 2 + 3 and f.prefix(I(3).r) == 1 + 2 + 3 + 4

var lz = LazySegTree<I, Int, Int>.from(Max<Int>, AddMax, [I] i => 0)
lz.apply(I(2)..=I(4), 10)
assert lz.fold(I(0)..=I(5)) == 10
```

The binary searches on a tree return boundaries too:
`t.max_right(i.l, ok)` is the furthest boundary such that the fold from
`i` up to it still satisfies `ok`.

## Graphs are functions

A graph algorithm takes "the neighbours of `u`" as a function. A stored
`Graph<V, E>` is one way to supply it; a formula is another, so an
implicit graph needs no adjacency list at all.

```
let g = Graph<V, E>.directed(ends)
let dist = dijkstra::<V>(u => g.adj(u), [(V(1), 0)], a => w[a.e])
assert [dist[v] for v in V] == [0, 5, 6, 13]

let (C, comp) = scc::<V>(u => g.adj(u))
assert C.K == 2 and comp[V(1)] == comp[V(3)]

domain N = 0..10
let hops = dijkstra::<N>(u => [(to: N(x)) for x in [u.to_idx * 2, u.to_idx + 1] if x < 10], [(N(1), 0)], a => 1)
assert hops[N(9)] == 4
```

`scc` and `components` return a fresh domain `C` of components along with
the labelling, so the condensed graph is indexed by `C` and cannot be
confused with the original. Distances are `?W`: an unreachable vertex is
`none`, not a large number that might take part in arithmetic.

`flow.cp` is Dinic's algorithm. `add` returns the arc, which you later
ask for its flow:

```
var net = FlowGraph<V>()
let a = net.add(V(0), V(1), 3)
...
assert net.max_flow(V(0), V(3)) == 5
assert net.flow(a) == 3
```

## Trees

`Rooted<V>` computes parents, depths, sizes and a heavy-first preorder in
one pass, and answers `lca`, `dist`, `ancestor` and `step` from it. The
preorder positions are their own domain, `Rooted<V>.Pre`, so a position
cannot be used where a vertex is expected. `Hld<V, T>` adds values on the
vertices, with path and subtree folds over any monoid. Path folds keep
the order from `a` to `b`, so non-commutative monoids work.

```
let r = Rooted<V>(u => [(to: c) for c in kids[u]], V(1))
assert r.lca(V(4), V(6)) == V(2) and r.dist(V(4), V(6)) == 3
var h = Hld<V, Int>(r, Sum<Int>, [V] v => v.to_idx + 1)
assert h.path(V(4), V(6)) == 4 + 2 + 5 + 6
assert h.sub(V(2)) == 2 + 4 + 5 + 6
```

## Numbers

`Mod<P>` carries its modulus in the type. It is a number like any other,
so the generic structures take it as they are. `ModR` is for a modulus
read from the input. `Comb<P>` precomputes factorials.

```
type M = Mod<998244353>
assert M(2).pow(10) == M(1024) and M(3) / M(3) == M(1) and M(-1) + M(1) == M(0)
let c = Comb<998244353>(100)
assert c.choose(10, 3) == M(120)

var f = Fenwick<I, M>(Sum<M>)
```

`/` and `%` on `Int` are Euclidean: the remainder is never negative, so
`x % m` is already the residue you want and `Mod(-1)` needs no fix-up.
`math.cp` has `gcd`, `ext_gcd`, `floor_div`, `ceil_div`, a linear `Sieve`,
`factorize`, `divisors` and `crt`.

## Strings and geometry

String functions compare elements only, so they accept a `Str` or any
list. A `Str` is ASCII bytes with O(1) indexing.

```
assert z_function("abacaba") == [7, 0, 1, 0, 3, 0, 1]
assert suffix_array("abracadabra") == [10, 7, 0, 3, 5, 8, 1, 4, 6, 9, 2]
assert [i for i in find_all("abracadabra", "abra")] == [0, 7]
```

Geometry is on integer points and every predicate is exact: `turn`,
`on_segment`, `segments_meet`, `hull`, `area2`, `locate`, and the orders
`ByXY` and `ByAngle` (which sorts by direction without `atan2`).

```
let square = hull([Pt(0, 0), Pt(2, 0), Pt(1, 1), Pt(2, 2), Pt(0, 2), Pt(1, 0)])
assert square == [Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)] and area2(square) == 8
```

## Bitsets

`Bitset<D>` is a set of indices of `D`, 64 per word. It is a type you
choose, never something a `[D]Bool` silently turns into.

```
var reach = Bitset<D>()
reach.insert(D(0))
for w in [5, 11, 40]: reach.union(reach.shifted(w))
assert reach.has(D(56)) and not reach.has(D(17))
```

## Asking the brute force what it is

`spec.cp` is for the moment when the slow solution works and you are
looking for the fast one. Feed it what the brute force computed and ask
whether it is a polynomial, a linear recurrence, convex, or a cost that
makes a DP optimization valid. Each check returns the first
counterexample, or `none` when the property holds on the data.

```
let slow = [sum(k * k for k in 1..=n) for n in 0..8]
assert poly_degree(slow) == 3
assert poly_at(slow, 1000) == 1000 * 1001 * 2001 / 6

type M = Mod<1000000007>
assert guess_at([M(1), M(1), M(2), M(3), M(5), M(8), M(13), M(21)], 50) == M(20365011074)

assert dnc_break((i, j) => (i - j) * (i - j) + j, 12, 12) == none
assert convexity_break([0, 5, 6, 9]) == 0
```

`guess_at` is Berlekamp-Massey followed by a jump to the `n`-th term; it
answers `none` when there are too few terms to confirm the recurrence it
found. `monge_break`, `dnc_break` and `knuth_break` test the conditions
behind the divide-and-conquer and Knuth optimizations; `convexity_break`
the one behind slope tricks and Lagrangian relaxation. A check that holds
is evidence, not a proof.

## The modules

- `algebra.cp`: operation values and their laws.
- `dsu.cp`, `fenwick.cp`, `segtree.cp`, `lazy.cp`, `sparse.cp`, `heap.cp`,
  `bitset.cp`: the data structures.
- `graph.cp`: `Graph`, `bfs`, `dijkstra`, `shortest` (with the path),
  `topo`, `scc`, `components`, `condense`.
- `flow.cp`: maximum flow and minimum cut.
- `tree.cp`: `Rooted` and `Hld`.
- `mod.cp`, `math.cp`: modular arithmetic, counting, number theory.
- `strings.cp`: Z, prefix function, KMP search, suffix array, LCP,
  palindromes.
- `geometry.cp`: integer points.
- `spec.cp`: structure discovery.
- `tour.cp`: the examples above, as tests.

Every public item has a `///` comment giving what it does, its cost and,
where it matters, its memory. `cp.py doc` prints them.

## Why the structures have `set` and not `t[i] = x`

Every structure here keeps an invariant over its storage: a segment
tree's inner nodes, a Fenwick tree's partial sums, a heap's order. A
`lends mut` projection hands out a cell and cannot repair anything after
the caller writes to it, so these types expose operations (`set`, `add`,
`apply`) and return values by copy. Where there is no invariant between
cells, lend them: `examples/local-lib/math/matrix.cp` does, with
`m.at(i, j) += x`.

## What is not here yet

Ordered sets and maps, hashing, a persistent or dynamic segment tree,
min-cost flow, matching, FFT/NTT and floating-point geometry. For the
shape of a library of your own, see `examples/local-lib/`.
