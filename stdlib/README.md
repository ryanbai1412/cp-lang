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

## The prelude

[`prelude.cp`](prelude.cp) lists the names a program has without a `use`.
Adding a name to the prelude is adding it to a `use` line there; the
compiler holds no list of its own.

```cp
// prelude.cp
use std.dsu.{Dsu}

// a program: no `use` needed
var d = Dsu<V>()
```

A program's own declaration or `use` of the same name wins.

## How it runs today

The Rust front end checks the library as it is written: generic structs,
traits and impls, `use` across modules, strict domain indexing, and
lexical borrow exclusivity.

```
$ cargo run -q -p cpl-cli -- check stdlib/*.cp examples/local-lib/*/*.cp
```

Checking is not running. The Rust interpreter and code generator are not
built yet, so the library's tests run on a reference interpreter,
`proto/cpi/`. It executes
the planned language directly, slowly and with every check on: an index
from the wrong domain, an out-of-range value, a broken algebraic law or a
`none` used as a value stops the test with the source line. The files are
written for the real compiler; when it can run them they should compile
unchanged, and the interpreter stays as the oracle to test it against.

## Bare operators: one structure, many behaviours

A structure never assumes what "combine" means. You hand it the operator,
and the same tree serves sums, maxima or your own monoid. An operator the
language already knows brings its identity with it; anything else is the
pair `Monoid(id, op)`.

```
domain I = 1..=5
let a = [I] i => [3, 1, 4, 1, 5][i.to_idx]
let sums = SegTree<I, Int>(+, a)
let highs = SegTree<I, Int>(max, a)
assert sums.fold(I(2)..=I(4)) == 6
assert highs.fold(I(2)..=I(4)) == 4

// your own monoid is one line: (best, count of best)
let best = Monoid<(Int, Int)>((Int.MIN, 0), (x, y) => if x.0 > y.0: x else: if y.0 > x.0: y else: (x.0, x.1 + y.1))
let t = SegTree<I, (Int, Int)>(best, [I] i => (a[i], 1))
assert t.all == (5, 1)
```

The bare operators are `+ * ^ | & min max gcd`. `min` and `max` over a
`?T` skip `none`, so `SegTree<I, ?Int>(min)` starts empty and the fold of
an empty range is `none` rather than a sentinel.

`algebra.cp` is the whole vocabulary, and it is short:

- `Monoid(id, op)` for an operation of your own, with its laws.
- `Action(id, compose, apply, over: op)` for a lazy update:
  `compose(new_f, old_f)` applies `old_f` first and then `new_f`;
  `apply(f, val, span)` returns the folded value `val` of `span` covered
  leaves after applying `f`.
- `desc(x)` reverses the order of one part of a key:
  `sort(mut xs, by: p => (desc(p.score), p.name))`, or a max-heap entry
  inside a `MinHeap`.
- `Reversible` is a hook with a default: a value is its own reverse unless
  it says otherwise, which is what lets `Rooted.fold_path` fold a
  non-commutative path over one tree.

Each one states its laws, and `laws_hold(m, samples)` checks them on
sample values, which is how the tests catch a "monoid" that is not
associative. The named wrappers of the first design (`Sum`, `Max`, `Min`,
`Prod`, `Xor`, `Less`, `Greater`, `Semigroup`, `Group`, `Order`, `flip`,
`both`, `opt`) are gone: the operator is the name.

`Fenwick` is sums: it needs `+` and `-` on `T` and nothing else is passed.
`SparseTable` takes any associative idempotent operator (`min`, `max`,
`gcd`, `&`, `|`, or a closure) and `fold(r)` returns `T` (asserting `r` is
non-empty). `LazySegTree<D, T>(act, a)` infers its tag type from `act`,
exposes `all` in O(1) and `update(i, f)` for single-leaf push-modify-pull;
the six stock actions are `AddSum`, `AddMin`, `AddMax`, `AssignSum`,
`AssignMin`, and `AssignMax`.

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
assert d.components == 2 and d.same(V(1), V(2))
// `d.find(e)` for an edge index `e` is rejected: an E is not a V
```

The domain also fixes the size. There is no `n` to pass and no `n + 1` to
remember: `Dsu<V>()` has one slot per vertex whether `V` is `0..n` or
`1..=n`.

`Dsu` also rolls back, for offline dynamic connectivity and divide and
conquer over time: `let snap = d.mark()` (a `Mark`), some `unite`s,
then `d.undo_to(snap)`. `d.groups()` returns every connected component as
`List<List<V>>`. `DsuOf<V, T>(init, join)` keeps a value per set, joined
on merge and restored by `undo_to`; `ParityDsu<V>` knows whether two
positions are on the same side (two-colouring, xor constraints) and
whether every constraint so far agrees (`consistent`).

`Marks<D>` in `marks.cp` (opt-in `use std.marks.{Marks}`) is a
generation-stamped visited table over `D` backed by compact 32-bit stamps
(`[D]0..=2_000_000_000`, 4 bytes/cell) with O(1) `.clear()` and automatic
table reset at the 32-bit ceiling.

Coordinate compression gives the values their own domain, so a rank cannot
be mixed up with a value or with an index of another array:

```
let xs = compress(all)                 // sorted, distinct
var f = Fenwick<xs.D, Int>()           // one slot per distinct value
f.add(xs.at(val), 1)                   // the rank of a value that is present
let hit = xs.first_ge(q)               // ?xs.D: the first rank holding a value >= q, for any q
assert xs[xs.at(val)] == val
```

## Ranges and boundaries

Range queries take a range of the domain, closed or half-open, so there
is no convention to remember. A boundary (`i.l` is the gap before `i`,
`i.r` the gap after) names a prefix without an off-by-one.

```
domain I = 0..6
var f = Fenwick<I, Int>()
for i in I: f.add(i, i.to_idx + 1)
assert f.sum(I(1)..=I(3)) == 2 + 3 + 4
assert f.sum(I(1)..I(3)) == 2 + 3
assert f.prefix(I(3).l) == 1 + 2 + 3 and f.prefix(I(3).r) == 1 + 2 + 3 + 4

var lz = LazySegTree<I, Int>(AddMax, [I] i => 0)
lz.apply(I(2)..=I(4), 10)
assert lz.fold(I(0)..=I(5)) == 10
```

`Fenwick2D<R, C, T>` is the same over a grid: `add(r, c, x)` and
`sum(rows, cols)`.

The binary searches on a tree return boundaries too:
`t.max_right(i.l, ok)` is the furthest boundary such that the fold from
`i` up to it still satisfies `ok`.

## Searching, maps, and reordering

Binary search is two free functions over a range, of `Int`s or of a
domain, and each says in its name which end it returns:

```
let k = bsearch_first(0..n, i => pre[i] >= need)   // ?Int: the first i where it holds
let t = bsearch_last(1..=10**9, x => feasible(x))  // the last x where it holds
```

Sorted lookup distinguishes sequences from ordered collections by return
type:

- Sorted `List<T>` and `[D]T`: `first_ge(x)` / `last_le(x)` return the
  index (`?Int` / `?D`); `first_ge(x, by:)` / `last_le(x, by:)` take a key function
  and return `?Int`.
- Ordered `Map<K, V>`: `first_ge`, `first_gt`, `last_le`, `last_lt`
  return `?(K, V)` pairs. Missing-key reads `m[k]` trap (use `m.get(k)`
  for `?V`); `m[k] = v` inserts or overwrites; compound updates
  (`m[k] += 1`, `m[k] min= v`, `m[k] max= v`) auto-insert from the
  operator identity; `m[k] ??= v` lazily inserts and lends mutably;
  `m.remove(k)` returns the removed `?V`.

In-place sequence operations in `seq.cp` cover `reverse`, `dedup`,
`rotate_left(mut a, k)` (signed Euclidean `k`, so negative `k` rotates
right), `next_permutation`, `prev_permutation`, `runs`, `List.insert`,
`List.remove`, `List.take`, and `swap_at`. A plain ring-buffer `Deque<T>`
lives in `deque.cp` for 0-1 BFS and queues.

## Graphs are functions

A graph algorithm takes "the neighbours of `u`" as a function. A stored
`Graph<V, E>` is one way to supply it; a formula is another, so an
implicit graph needs no adjacency list at all.

```
let g = Graph<V, E>.directed(ends)
let dist = dijkstra(g, from: [(V(1), 0)], w: a => w[a.e])
assert [dist[v] for v in V] == [0, 5, 6, 13]

let (C, comp) = scc(g)
assert C.K == 2 and comp[V(1)] == comp[V(3)]

domain N = 0..10
let hops = dijkstra(from: [(N(1), 0)], next: u => [N(x) for x in [u.to_idx * 2, u.to_idx + 1] if x < 10], w: a => 1)
assert hops[N(9)] == 4
```

An algorithm takes either: `bfs(g, from: [s])` for a stored graph,
`bfs(from: [s], next: u => ...)` for a formula. A neighbour may be a bare
vertex or an arc with a `to` field.

`scc` and `components` return a fresh domain `C` of components along with
the labelling, so the condensed graph is indexed by `C` and cannot be
confused with the original. Distances are `?W`: an unreachable vertex is
`none`, not a large number that might take part in arithmetic.

Connectivity is in `graph.cp` as well: `topo` (returning `?List<V>`),
`two_edge_components` (bridges), `articulation_points`,
`biconnected_components`, and `two_color`.

`flow.cp` is Dinic's algorithm (`FlowGraph`) plus `max_weight_matching`
for assignment problems. `add` returns the arc, which you later ask for
its flow:

```
var net = FlowGraph<V>()
let a = net.add(V(0), V(1), 3)
...
assert net.max_flow(V(0), V(3)) == 5
assert net.flow(a) == 3
```

## Trees

`Rooted<V, Pre>` is the one tree type. It computes parents, depths, sizes and a
heavy-first preorder in one pass (via `Rooted(root:, adj:)`, `Rooted.weighted`,
or `Rooted.from_parents(par)`), and answers `lca`, `dist`, `ancestor`,
`step`, `lift_while`, iterative enter/exit `tour`, and
`virtual_tree` from it. Each tree value owns its own preorder domain
`r.Pre` (with `r.pos(v) -> r.Pre` and `r.reorder(a) -> [r.Pre]T`), so
positions from two different trees cannot be mixed by accident.

A value per vertex lives in any structure over `r.Pre`. `fold_path` and
`apply_path` walk a path over it, and a subtree is the range `r.sub(v)`,
so there is no second "tree with values" type and no second copy of the
data:

```
let r = Rooted(root: V(1), adj: u => kids[u])
assert r.lca(V(4), V(6)) == V(2) and r.dist(V(4), V(6)) == 3
var st = SegTree(+, r.reorder([V] v => v.to_idx + 1))
assert r.fold_path(V(4), V(6), st) == 4 + 2 + 5 + 6
assert st.fold(r.sub(V(2))) == 2 + 4 + 5 + 6
st.set(r.pos(V(2)), 100)
```

With a `LazySegTree`, `r.apply_path(a, b, mut st, tag)` updates the path.
`edges: true` leaves out the top vertex, for a value per edge stored at
its lower end. Path folds keep the order from `a` to `b`: the stretch that
climbs is folded top-down and turned round with `rev()`, so a
non-commutative value implements `Reversible` (store both directions and
swap them; `examples/local-lib/math/hom.cp` does) and everything else
pays nothing.

## Grids

A cell is the pair `(Row, Col)`, and it indexes a `[Row, Col]T` directly.
`grid.cp` adds what a cell can do. The neighbours are named by how many
there are, and they never leave the grid:

```
use std.grid.{adj4, Dir4, step}

for n in p.adj4:
    if open[n] and not seen[n]: ...

let d = Dir4.from("URDL", ch)!        // or "NESW"
let q = p.step(d.cw)                  // ?(Row, Col): none off the edge
```

`adj8`, `ray(d)` (the cells up to the edge), and `transposed` are there
too.

## Numbers

`Mod<P>` carries its modulus in the type. It is a number like any other,
so the generic structures take it as they are (and `.try_inv() -> ?Mod<P>`
handles non-invertible residues cleanly). A modulus read from the
input is the same type: freeze it under a capitalized name
(`let P: Int = read()`, then `type Z = Mod<P>`). `Comb<M>()` lazily
auto-grows factorials on demand with interior mutability, so an immutable
`let c = Comb<M>()` works across test cases.

```
type M = Mod<998244353>
assert M(2).pow(10) == M(1024) and M(3) / M(3) == M(1) and M(-1) + M(1) == M(0)
let c = Comb<M>()
assert c.choose(10, 3) == M(120)

var f = Fenwick<I, M>()
```

`/` and `%` on `Int` are Euclidean: the remainder is never negative, so
`x % m` is already the residue you want, `a / b` already equals
`floor(a / b)` for `b > 0`, and `Mod(-1)` needs no fix-up.
`math.cp` has `gcd`, `ext_gcd`, `ceil_div`, `choose2`, `bit_len`, a linear
`Sieve` (with `phi_table`, `mobius_table`, `div_count_table`, `div_sum_table`,
and generic `sieve_multiplicative`), `factor`, `factor_big`, `divisors`,
`divisors_from_factors`, `inv_mod`, `crt`, `crt_all`, `mod_log`, `mod_sqrt`,
`primitive_root`, and `floor_sum`.

`ntt.cp` provides `ntt<P>`, `convolve<P>`, `convolve_mod`, and `convolve_int`
(all with `cyclic: Bool = false`), plus in-place `sos(mut a, over: Sub | Sup = Sub, inverse: Bool = false)`,
`fwht(mut a, inverse: Bool = false)`, and `convolve_or`/`convolve_and`/`convolve_xor`/`convolve_subset`.

`matrix.cp` provides flat row-major `Mat<T>` (`+`, `-`, `*`, `pow`, `mul_vec`,
`transpose`, `product` monoid, exact-field `det`, `rank`, `solve`, `solve_space`,
`inverse`), partial-pivoting `det_real`/`rank_real`/`solve_real`/`inverse_real`
over `Mat<F64>` with explicit `eps:`, and `XorBasis` (with `add`, `reduce`,
`contains`, `max_xor`, `min_xor`, `kth`, `merge`, `monoid`).

## Strings and geometry

String functions compare elements only, so they accept a `Str` or any
list. A `Str` is ASCII bytes with O(1) indexing.

```
assert z_function("abacaba") == [7, 0, 1, 0, 3, 0, 1]
assert suffix_array("abracadabra") == [10, 7, 0, 3, 5, 8, 1, 4, 6, 9, 2]
assert [i for i in find_all("abracadabra", "abra")] == [0, 7]
```

Geometry separates exact integer points (`Pt`) from careful
floating-point vectors (`Vec`). Exact predicates (`turn`, `ByAngle`,
`on_segment`, `segments_meet`, `locate`, `locate_convex`,
`convex_hull`, `minkowski_sum`, `diameter2`, `hull_extreme`,
`closest_pair2`) compute orientation signs in `I128`, staying exact up
to `1e18` coordinates. Floating-point helpers (`Vec`, `close`,
`sign_eps`, `project`, `reflect`, `line_intersect`, `circumcenter`,
`circle_line_intersect`, `circle_intersect`, `HalfPlane`,
`half_plane_intersect`) take explicit `eps: F64 = 1e-9`.

```
let square = [Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)]
assert area2(square) == 8 and locate_convex(square, Pt(1, 1)) == Location.Inside
assert diameter2(square) == 8 and closest_pair2(square) == 4
```

## Bitsets

`Bitset<D>` is a set of indices of `D`, 64 per word. It is a type you
choose, never something a `[D]Bool` silently turns into.

```
var reach = Bitset<D>()
reach.add(D(0))
for w in [5, 11, 40]: reach.union(reach.shifted(w))
assert reach.contains(D(56)) and not reach.contains(D(17))
```

It speaks the collection verbs every container does: `add`, `remove`,
`toggle`, `x in s`, `len`. `shifted(k)` takes a negative `k` to move down.

## DP: the parts a recurrence calls by name

The recurrence stays your code; `dp.cp` holds the structures and
optimizations it calls. A window optimum is `window.cp`'s `MonoDeque`, the convex hull
trick a `LiChao` over anything with `at(x)`, a layer whose best split moves
right `dnc_layer`, an interval DP with monotone splits `knuth`, and
"exactly `k` pieces" over a convex cost `aliens`. Searches on answers live
in `seq.cp` beside `bsearch_first`, and every one returns a position or
`none`, never a sentinel. Each optimization names its law, and `spec.cp`
checks it on small inputs.

```
// Frog 3: dp[i] = min over j < i of dp[j] + (h[i] - h[j])^2 + c
var hull = LiChao<Line, Int>(min, 0..=10**6)
var dp = [0 for _ in h]
for i in 0..h.len:
    if i > 0: dp[i] = hull.at(h[i])! + h[i] * h[i] + c
    hull.add(Line(-2 * h[i], dp[i] + h[i] * h[i]))

let root2 = bsearch_real(0.0, 2.0, x => x * x >= 2.0)!   // ?F64
let peak = ternary_max(0..n, i => a[i])!                   // a position of the top
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

- `algebra.cp`: `Monoid`, `Action` (`compose(new_f, old_f)`,
  `apply(f, val, span)`), `desc`, `Reversible`, and their laws.
- `dsu.cp` (`Dsu` with `components`, rollback `mark`/`undo_to`, and
  `groups`; `DsuOf` with a value per set; `ParityDsu`), `marks.cp` (opt-in `Marks<D>` with 32-bit stamps and O(1)
  `clear`), `fenwick.cp` (1D and 2D), `segtree.cp`, `lazy.cp` (with
  `update(i, f)` and `all`), `sparse.cp` (`fold -> T`, asserts non-empty),
  `heap.cp` (`MinHeap`, `MaxHeap`, with `take`), `deque.cp` (`Deque` with
  `take`), `bag.cp` (`SortedBag` ordered multiset with `rank`/`nth` and
  4-lookups), `bitset.cp`: the data structures.
- `seq.cp`: `scan`, `bsearch_first` / `bsearch_last`, `min(by:)` / `max(by:)`
  (first-extremum tie-breaking, `none` on empty), `compress` (with range
  slicing), `Range.l` / `Range.r`, `first_ge` / `last_le` (with optional `by:`),
  ordered `Map` 4-lookups (`first_ge`, `first_gt`, `last_le`, `last_lt`
  returning `?(K, V)`), `swap_at`, `List.take`, `List.insert`,
  `List.remove`, `reverse`, `dedup`, `rotate_left` (signed Euclidean `k`),
  `next_permutation`, `prev_permutation`, `runs`; searches on answers
  `bsearch_real`, `ternary_max` / `ternary_min` (and `_real`),
  `parallel_bsearch`.
- `dp.cp`: `Line`, `LiChao`, `monotone_minima`, `dnc_layer`,
  `monge_path`, `knuth`, `aliens`, `subset_sums` (SOS is `ntt.cp`'s `sos`).
- `slope.cp`: `Concave` and `Convex`, integer functions kept as sorted slopes
  (slope trick).
- `graph.cp`: `Graph`, `bfs`, `dijkstra`, `topo` (including vertexless
  `topo::<V>`), `scc`, `components`, `condense`, `two_edge_components`,
  `articulation_points`, `biconnected_components`, `two_color`.
- `flow.cp`: `FlowGraph` (Dinic's max flow and min cut) and
  `max_weight_matching`.
- `tree.cp`: `Rooted` (value-owned `t.Pre`, `t.pos`, `t.reorder`,
  `Rooted.from_parents`, `Rooted.weighted`, iterative `t.tour`,
  `virtual_tree`).
- `grid.cp`: `Dir4`, `adj4`, `adj8`, `step`, `ray`, `transposed`.
- `mod.cp`, `math.cp`: modular arithmetic (`try_inv`), interior-mutable
  auto-growing `Comb<M>()` (including `multinomial`), number theory (`gcd`,
  `ext_gcd`, `ceil_div`, `choose2`, `bit_len`, `Sieve`, `sieve_multiplicative`,
  `factor`, `factor_big`, `divisors`, `divisors_from_factors`, `inv_mod`,
  `crt`, `crt_all`, `mod_log`, `mod_sqrt`, `primitive_root`, `floor_sum`).
- `ntt.cp`: `ntt<P>`, `convolve<P>`, `convolve_mod`, `convolve_int` (`cyclic:`),
  `SosOver`, `sos` (`over:`, `inverse:`), `fwht` (`inverse:`), `convolve_or`, `convolve_and`, `convolve_xor`, `convolve_subset`.
- `poly.cp`: formal power series and multipoint operations over NTT (`deriv`, `integral`,
  `inv`, `log`, `exp`, `pow`, `sqrt`, `eval`, `div_mod`, `prod_linear`, `taylor_shift`,
  `eval_all`, `compose`).
- `matrix.cp`: `Mat<T>` (arithmetic, `pow`, `product`, exact `det`/`rank`/`solve`/`solve_space`/`inverse`),
  `det_real`/`rank_real`/`solve_real`/`inverse_real` (`eps:`), and `XorBasis`.
- `strings.cp`: Z, prefix function, KMP (`find_all`, `prefix_matches`,
  `borders`, `period`, `overlap`), Lyndon factorization & `min_rotation`,
  `manacher` (`Palindromes`) & `PalindromicTree<A>`, `suffix_array` /
  `lcp_array` / `SuffixArray<T>`, `SuffixAutomaton<A>`, `Trie<A>`,
  `AhoCorasick<A>`, generalized `RollingHash` (`lcp`, `lcs`, `lcp_with`),
  `HashVal`, and `Hash2D<R, C>`.
- `geometry.cp`: exact integer points (`Pt` with `Ord`/`Show`, `dot`/`dot128`,
  `cross`/`cross128`, `dist2`/`dist`, `turn`, `same_dir`, `ByAngle`, `arg`,
  `on_segment`, `segments_meet`, `any_segments_meet`, `lower_hull`, `upper_hull`,
  `convex_hull`, `convex_hull_idx`, `minkowski_sum`, `minkowski_upper`,
  `area2`/`area2_128`, `Location`, `locate`, `locate_convex`, `diameter2`,
  `min_bounding_box`, `hull_extreme`, `hull_tangents`, `closest_pair2`,
  `closest_pair`), careful floating-point geometry (`Vec`, `close`, `sign_eps`,
  `project`, `reflect`, `line_dist`, `seg_dist`, `line_intersect`,
  `circumcenter`, `circle_line_intersect`, `circle_intersect`,
  `circle_tangents`, `circle_circle_tangents`, `circles_union_area`,
  `HalfPlane`, `half_plane_intersect`, `polygon_area`, `polygon_cut`,
  `min_enclosing_circle`, `circle_polygon_area`), and exact 3D
  geometry (`Pt3`, `dot3`/`dot3_128`, `cross3`, `dist2_3`, `convex_hull_3d`).
- `spec.cp`: structure discovery.
- `tour.cp`: the examples above, as tests.

Every public item has a `///` comment giving what it does, its cost and,
where it matters, its memory. `cp.py doc` prints them.

## Why the structures have `set` and not `t[i] = x`

Every structure here keeps an invariant over its storage: a segment
tree's inner nodes, a Fenwick tree's partial sums, a heap's order. A
`lends mut` projection hands out a cell and cannot repair anything after
the caller writes to it, so these types expose operations (`set`, `update`,
`add`, `apply`) and return values by copy. Where there is no invariant
between cells, lend them: `examples/local-lib/math/matrix.cp` does, with
`m.at(i, j) += x`.

## What is deliberately not here

The standard library is what most contests need most of the time. The
structures a problem needs once a season live in
[`examples/local-lib/`](../examples/local-lib/), written on top of this
library the way your own would be: `MonoDeque`, `Steps`, `TwoSat`, `Mat`,
`palindromes`, `shortest` / `walk_back`, `mst`, `convex_hull`, `Dir8`,
`MinCostFlow`, `Rope`, `RangeKth`, `LiChao`, `LiChaoArena`, `Tri`,
`PotentialDsu`, `Cover`, `TwoStackFold`, `mo_order`,
`XorBasis`, `Interner`, `RollbackArray`, `Frac`, `Fixed`, `Congruence`,
`Hom`, `Powers`, NTT, `sos_sub` / `sos_sup`, `monotone_minima` /
`dnc_layer`, `knuth`, `fold_others`, `order_stats`, `borders` /
`overlaps`, `Trie`, `SuffixAutomaton`, `Hash2D`, `Functional`,
`Centroids`, `KruskalTree`, `MinCount`.

Not written yet, in either place: ordered sets and maps as library code
(`SortedBag` is in `bag.cp`) and a real-valued `bsearch`.
