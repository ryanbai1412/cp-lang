# Trees: one rooted value for every static-tree technique

Status: design for `std.tree` (plan/STDLIB.md §4.11, tree and tour part).
Items marked [new] are not in `stdlib/tree.cp` yet; everything else is.

## 1. What contestants do with trees

The survey covers the tree problems in our corpus (`proto/ports/`, 60-odd
files that build a `Rooted`), ecnerwala's `static_forest_t` (in
`proto/ports/ecnerwala/cf2062e1_orig.cpp` and siblings), KACTL's graph/tree
chapter (LCA, binary lifting, HLD, centroid, link-cut tree, compress tree,
tree isomorphism), and the AtCoder Library (which has no tree module: ACL
users hand-write every pattern below on top of `segtree`/`lazy_segtree`).
Codeforces, AtCoder and ICPC tree problems reduce to fifteen patterns; a tree
library is complete when each is a few lines that name the technique.

| # | Pattern | Typical problem | Frequency |
|---|---|---|---|
| P1 | bottom-up DP (postorder fold over children) | sizes, matchings, DP V | every tree contest |
| P2 | top-down propagation (prefix along the root path) | depth, path max to root | very common |
| P3 | rerooting: the P1 answer for every root | DP V, sum of distances | common |
| P4 | subtree as a range of an Euler order | subtree add/sum, CF 620E | very common |
| P5 | ancestry: LCA, distance, k-th ancestor, k-th vertex on a path | almost every query problem | very common |
| P6 | path fold and update (heavy-light) | CSES path queries, CF 2237H | common |
| P7 | first ancestor satisfying a monotone predicate | "highest ancestor with w <= x" | common |
| P8 | edge values (stored at the lower endpoint) | path max edge, MST queries | common |
| P9 | small-to-large (heavy child kept, light merged) | CF 600E, subtree distinct counts | common |
| P10 | centroid decomposition | CF 342E, paths of length k | regular |
| P11 | virtual (auxiliary) tree of k marked vertices | CF 613D, sum-of-k queries | regular |
| P12 | diameter, centre, eccentricity | CF 1805D, farthest vertex | common |
| P13 | subtree isomorphism (canonical shapes) | count distinct subtrees | occasional |
| P14 | forests (several roots, "same tree?") | functional graphs, DSU forests | occasional |
| P15 | dynamic trees: link, cut, path fold | CF 2237H fast path, LCT tasks | rare, hard |

What the corpus says today (`research/frictions/`): P2 has no helper and
every port writes a preorder loop with `if let p = t.parent(v)`
(cf2062e2); P4 composes well but range-add over `t.sub(v)` needs a Fenwick
verb that belongs to the range lane; P6 composes well (`fold_path`,
`apply_path`) and the HLD painter wants an interval map over `t.Pre`
(cf2237h, containers lane); P10 and P3 live only in
`examples/local-lib/graph/centroids.cp`; P15 is missing and costs 2x on
cf2237h; and `Rooted` itself measured 1.23x time and 1.76x memory against
`static_forest_t` (cf2062e1) because it stores eight-byte integers and
per-vertex child lists.

## 2. The idea: one value, one order, every structure

A `Rooted<V, Pre>` is built once in O(n) and answers every static question.
It owns a fresh domain `t.Pre`: the positions of a heavy-first preorder.
That one order makes three things contiguous at once:

- every subtree is one range, `t.sub(v): Range<Pre>`;
- every heavy chain is one range, so a path is O(log n) ranges (`t.chains`);
- the heavy child of `v` sits at `t.pos(v) + 1`, so small-to-large keeps
  it in place and iterates light subtrees as ranges.

So there is no `Hld`, `EulerTour` or `Lca` type. A value per vertex lives in
any range structure over `t.Pre` (`SegTree`, `LazySegTree`, `Fenwick`,
`SparseTable`, an interval map), and `t.sub`, `t.fold_path` and
`t.apply_path` hand that structure the right ranges. Subtree queries,
path queries and path-and-subtree mixes share one structure and one
position map, which is exactly the invariant an editorial states ("flatten
by HLD order; subtrees and chains are both ranges").

Domains keep the index spaces apart:

| Space | Type | Where it comes from |
|---|---|---|
| vertices | `V` | the user's `domain V` |
| positions in the order | `t.Pre` | fresh per tree; `t.pos(v)`, `t.at(p)` |
| subtree, chain | `Range<Pre>` | `t.sub(v)`, `t.chains(a, b)` |
| cuts between positions | `Pre.Cut` | `r.lo`, `r.hi` of those ranges |
| edges | the lower endpoint `V` | `t.lower(u, v)`; `edges: true` on path ops |
| centroid ancestors | `V`, with a distance | `cd.ancestors(v)` |
| subtree shapes | fresh `Shape` | `t.shapes()` [new] |

Two trees built in one program have different `Pre` domains, so a segment
tree indexed by one tree's positions cannot be queried with the other's
ranges: the classic "two Euler tours, mixed up `tin`" bug is a type error.
Raw `Int` never appears in a signature except for depths, lengths, sizes and
counts, which really are numbers.

## 3. The API

Costs: n vertices; every query is worst case unless marked amortized.
Memory is stated for constructors and anything that allocates.

### 3.1 Building

```cp
struct Rooted<V, Pre>
    // O(n) time and memory, iterative (no recursion depth limit)
    fn new<N>(adj: N, root: V) -> Rooted<V, Pre>          // Rooted(adj, root: r)
    fn weighted<N, A>(adj: N, root: V, len: A -> Int)     // edge lengths for far()
    fn from_parents(par: [V]?V)                            // exactly one none
    fn forest<N>(adj: N, roots: List<V>) [new]            // one tree per root
```

`adj` is anything with `Neighbours` (a `Graph`, `[V]List<V>`, a closure),
so `Rooted(Graph.undirected(read<Tree<V>>()), root: V.first)` reads a tree.
`new` on a graph with cycles roots its DFS tree (documented, tested). A
vertex the DFS never reaches traps; a forest says so with `forest`.

### 3.2 Shape (all O(1))

```cp
    root: V
    fn parent(self, v: V) -> ?V
    fn children(self, v: V) -> List<V>            // in the order adj listed them
    fn heavy(self, v: V) -> ?V [new]              // largest child, none at a leaf
    fn light(self, v: V) -> yields V [new]        // children except heavy(v)
    fn depth(self, v: V) -> Int                   // edges to the root
    fn far(self, v: V) -> Int [new: was a field]  // length to the root (weighted)
    fn size(self, v: V) -> Int                    // vertices in the subtree
    fn root_of(self, v: V) -> V [new]             // the root of v's tree (forests)
    @get fn subtree_sizes(self) -> [V]Int         // O(n), a copy
```

### 3.3 Orders and positions

```cp
    domain Pre                                   // heavy-first preorder
    fn pos(self, v: V) -> Pre                    // O(1)
    fn at(self, p: Pre) -> V [new: was a field]  // O(1), inverse of pos
    fn sub(self, v: V) -> Range<Pre>             // O(1), t.sub(v).len == t.size(v)
    fn subtree(self, v: V) -> yields V [new]     // O(size(v)), preorder
    fn reorder<T>(self, a: [V]T) -> [Pre]T       // O(n)
    @get fn preorder(self) -> List<V>            // O(n), in Pre order
    @get fn postorder(self) -> List<V>           // O(n), children before parent
    @get fn tour(self) -> yields (v: V, entering: Bool)   // O(n), 2n events
```

### 3.4 Ancestry and paths

```cp
    fn is_anc(self, a: V, b: V) -> Bool          // O(1); a is_anc a
    fn lower(self, u: V, v: V) -> V              // O(1); the child end of edge uv
    fn lca(self, a: V, b: V) -> V                // O(log n), chain jumps
    fn dist(self, a: V, b: V) -> Int             // O(log n), edges
    fn far_dist(self, a: V, b: V) -> Int [new]   // O(log n), weighted length
    fn ancestor(self, v: V, k: Int) -> ?V        // O(log n); none above the root
    fn step(self, a: V, b: V, k: Int) -> ?V      // O(log n); k-th vertex a -> b
    fn lift_while(self, v: V, pred: V -> Bool) -> ?V  // O(log^2 n), pred monotone
    fn path(self, a: V, b: V) -> List<V>         // O(dist), a first
```

`step(a, b, 1)` is "the neighbour of `a` toward `b`" (ecnerwala's
`get_next`, and `get_subtree` when `b` is below `a`), so neither gets a name.
`lift_while(v, pred)` returns the highest ancestor `u` of `v` (possibly `v`)
with `pred` true on the whole `v..u` path, or `none` if `pred(v)` is false.

### 3.5 Path and subtree structures (heavy-light)

```cp
    // O(log n) ranges; rising tells which ranges run upward from a
    fn chains(self, a: V, b: V, edges: Bool = false)
        -> yields (range: Range<Pre>, rising: Bool)
    // O(log n) range folds: O(log^2 n) on a SegTree, O(log n) on a SparseTable
    fn fold_path<S, T>(self, a: V, b: V, ds: S, edges: Bool = false) -> T
    // O(log n) range applies
    fn apply_path<S, F>(self, a: V, b: V, mut ds: S, tag: F, edges: Bool = false)
```

`ds` is any structure with `fold(Range<Pre>)` (respectively
`apply(Range<Pre>, tag)`) and a monoid. When the monoid is
`Reversible`, `fold_path` reverses rising pieces, so the result is the fold
in path order from `a` to `b`; commutative monoids pay nothing.
`edges: true` drops the top vertex of the path, the convention that stores
an edge's value at its lower endpoint (`t.pos(t.lower(u, v))`).

### 3.6 Folds over the whole tree

```cp
    // O(n): v gets f(value at parent, v); the root gets f(init, root)
    fn down<T>(self, init: T, f: (T, V) -> T) -> [V]T [new]
    // O(n): v gets finish(fold over children c of lift(value at c, c, v), v)
    fn up<T, E, M>(self, m: M, lift: (T, V, V) -> E, finish: (E, V) -> T) -> [V]T [new]
    // O(n): the value up() would give v if the tree were rooted at v
    fn reroot<T, E, M>(self, m: M, lift: (T, V, V) -> E, finish: (E, V) -> T) -> [V]T [new]
    // O(deg v): each child c of v with the fold of the others' f
    fn excl_fold<T, M>(self, v: V, m: M, f: V -> T) -> yields (c: V, rest: T)
```

`up` and `reroot` take the same three arguments, so promoting a rooted DP
to an all-roots DP is a one-word edit. `lift(x, c, p)` sees both ends of
the edge, which is where edge weights and direction enter. `reroot` uses
prefix and suffix products of siblings, so `m` needs no inverse (no
division modulo a composite).

### 3.7 Whole-tree facts

```cp
    fn diameter(self) -> (len: Int, u: V, v: V)   // O(n), weighted when built so
    @get fn centroid(self) -> V                   // O(depth)
    @get fn centroids(self) -> List<V>            // O(depth), one or two
    fn shapes(self) -> (domain Shape, of: [V]Shape) [new]  // O(n log n), AHU
```

`t.shapes().of[a] == t.shapes().of[b]` exactly when the subtrees of `a`
and `b` are isomorphic as rooted trees. Unrooted isomorphism roots both
trees at their centroids (at most two candidates each).

### 3.8 Derived trees

```cp
    // O(k log k + k log n): marked vertices plus pairwise LCAs, preorder,
    // and the virtual edges (u above v); lengths are t.dist(u, v)
    fn virtual_tree(self, keys: List<V>) -> (nodes: List<V>, edges: List<(u: V, v: V)>)

struct Centroids<V> [new: promoted from examples/local-lib/graph]
    fn new<N>(adj: N) -> Centroids<V>             // O(n log n) time and memory
    root: V
    fn parent(self, c: V) -> ?V                   // O(1), in the centroid tree
    fn depth(self, c: V) -> Int                   // O(1), <= log2 n
    fn ancestors(self, v: V) -> List<(c: V, dist: Int)>   // O(1); v's own first
    fn piece(self, c: V) -> yields (v: V, dist: Int, branch: V)  // O(|piece|)
```

`ancestors(v)` lists the centroids whose piece contains `v`, nearest first,
with the tree distance from each: the whole structure behind "nearest
marked vertex" and "sum over marked vertices of a function of distance".
`piece(c)` walks `c`'s component with distances and the neighbour of `c`
(`branch`) each vertex hangs off, for "count paths through `c`, excluding
pairs in the same branch".

### 3.9 Dynamic trees (next step)

```cp
struct LinkCut<V, T> [new, after the static API]
    fn new<M>(m: M, init: [V]T)                   // O(n)
    fn link(mut self, u: V, v: V)                 // amortized O(log n)
    fn cut(mut self, u: V, v: V)
    fn connected(mut self, u: V, v: V) -> Bool
    fn fold_path(mut self, u: V, v: V) -> T       // path order when Reversible
    fn set(mut self, v: V, x: T)
    fn lca(mut self, u: V, v: V, root: V) -> ?V
```

Same verbs as `Rooted` where they mean the same thing (`fold_path`, `lca`),
same monoid convention as every range structure.

## 4. Laws

Every law is a property test in `stdlib/tree.cp`.

1. `t.pos(b) in t.sub(a)` iff `t.is_anc(a, b)`; `t.sub(v).len == t.size(v)`.
2. `t.at(t.pos(v)) == v`; `t.reorder(a)[t.pos(v)] == a[v]`.
3. If `h = t.heavy(v)` then `t.pos(h)` is `t.pos(v)` plus one, and
   `t.size(h) >= t.size(c)` for every child `c`.
4. `t.dist(a, b) == t.depth(a) + t.depth(b) - 2 * t.depth(t.lca(a, b))`.
5. `t.step(a, b, k)` is `t.path(a, b)[k]` for `0 <= k <= t.dist(a, b)`,
   `none` past `b`; `t.ancestor(v, k) == t.step(v, t.root, k)`.
6. `t.fold_path(a, b, ds)` equals folding `t.path(a, b)` in order (for
   `Reversible` or commutative monoids); `fold_path(b, a)` is its `rev`.
7. The ranges of `t.chains(a, b)` partition `t.path(a, b)`; there are at
   most 2 log2 n of them.
8. `t.up(m, lift, finish)[t.root] == t.reroot(m, lift, finish)[t.root]`, and
   `reroot(...)[v]` equals `Rooted(adj, root: v).up(...)[v]`.
9. `t.diameter().len == max over a, b of t.dist(a, b)`; for every `v`, the
   eccentricity of `v` is `max(t.dist(v, d.u), t.dist(v, d.v))`.
10. `Centroids`: `cd.depth(c) <= log2 n`; each `v` lies in the piece of
    every centroid in `cd.ancestors(v)` and no other.

## 5. What stays user code

- The recurrence. `up`, `down` and `reroot` own traversal order and sibling
  bookkeeping; the DP's meaning is the user's closure.
- The range structure and its monoid. `Rooted` never stores values.
- What to do with each small-to-large step: the library gives
  `heavy`, `light` and `subtree`; the container being merged (a map, a
  `Bitset`, a counter array over `Pre`) is the problem's.
- Offline orderings (Mo's algorithm on trees, sorting queries by LCA), and
  eccentricity for all vertices (law 9 is the whole algorithm).
- Tree generation and reading formats: `Tree<V>`, parent arrays and edge
  lists are input types, not tree methods.

## 6. Problems written against the API

### AtCoder DP V, Subtree (rerooting)

For every root, the number of connected black sets containing it, modulo
`m` (not necessarily prime). The rooted DP: `down[v]` is the product over
children of `down[c] + 1`. The all-roots DP is the same three arguments.

```cp
let n: 1..=1e5 = read()
let m: 2..=1e9 = read()
domain V = 1..=n
let t = Rooted(Graph.undirected(read<Tree<V>>()), root: V.first)

// a child either is all white or contributes a set containing it
let ways = t.reroot(Monoid(1, (a, b) => a * b % m), (d, c, p) => d + 1, (acc, v) => acc)
for v in V: print(ways[v])
```

The C++ is two DFS passes with prefix and suffix arrays (about 50 lines);
here the editorial sentence is the code.

### Codeforces 620E, New Year Tree (subtree as a range)

Recolour a subtree; count distinct colours (at most 60) in a subtree.

```cp
domain V = read(1..=4e5)
let c: [V]1..=60 = read()
let t = Rooted(Graph.undirected(read<Tree<V>>()), root: V.first)

let assign = Action(none, (f, g) => f ?? g, (f, s, len) => f ?? s, over: |)
var colours = LazySegTree(assign, t.reorder([V] v => 1 << c[v]))
for _ in 0..read(1..=4e5):
    match read(1..=2):
        1 => colours.apply(t.sub(read<V>()), some(1 << read<1..=60>()))
        2 => print(colours.fold(t.sub(read<V>())).popcount)
```

### CSES 2134, Path Queries II (heavy-light path fold)

Point updates of vertex values, maximum on a path.

```cp
domain V = read(1..=2e5)
let q: 1..=2e5 = read()
let x: [V]1..=1e9 = read()
let t = Rooted(Graph.undirected(read<Tree<V>>()), root: V.first)

var best = SegTree(max, t.reorder(x))
for _ in 0..q:
    match read(1..=2):
        1 => best.set(t.pos(read<V>()), read<1..=1e9>())
        2 => print(t.fold_path(read<V>(), read<V>(), best))
```

Swapping `SegTree(max, ...)` for a `SparseTable` (static values) or an
edge-valued tree (`edges: true`) is one local edit each.

### Codeforces 342E, Xenia and Tree (centroid decomposition)

Vertex 1 starts red; paint a vertex red, or report the distance to the
nearest red vertex. Every path from `v` passes through one of `v`'s centroid
ancestors, so each centroid remembers its nearest red vertex.

```cp
domain V = read(1..=1e5)
let q: 1..=1e5 = read()
let cd = Centroids(Graph.undirected(read<Tree<V>>()))

var near: [V]?Int = none
fn paint(v: V):
    for (c, d) in cd.ancestors(v): near[c] = some(min(near[c] ?? d, d))
paint(V.first)
for _ in 0..q:
    let (kind, v) = read<(1..=2, V)>()
    if kind == 1: paint(v)
    else: print(min(near[c]! + d for (c, d) in cd.ancestors(v) if near[c] != none))
```

### Codeforces 600E, Lomsat gelral (small-to-large)

For every vertex, the sum of the colours that occur most often in its
subtree. Keep the heavy child's counts; add each light subtree vertex by
vertex: every vertex is added O(log n) times.

```cp
domain V = read(1..=1e5)
let c: [V]1..=V.count = read()
let t = Rooted(Graph.undirected(read<Tree<V>>()), root: V.first)

var cnt: [1..=V.count]Int = 0
var (top, sum) = (0, 0)
fn add(u: V):
    cnt[c[u]] += 1
    if cnt[c[u]] > top: (top, sum) = (cnt[c[u]], 0)
    if cnt[c[u]] == top: sum += c[u]

var ans: [V]Int = 0
```

A light child's subtree is one range of `t.Pre`, so adding it and clearing
it are loops over `t.subtree(c)`; the keep/clear order is the algorithm and
stays visible:

```cp
fn solve(v: V, keep: Bool):
    for c in t.light(v): solve(c, keep: false)
    if let h = t.heavy(v): solve(h, keep: true)
    for c in t.light(v):
        for u in t.subtree(c): add(u)
    add(v)
    ans[v] = sum
    if not keep:
        for u in t.subtree(v): cnt[c[u]] -= 1
        (top, sum) = (0, 0)
solve(t.root, keep: true)
for v in V: print(ans[v])
```

Recursion goes as deep as the tree; `solve` touches each vertex O(log n) times.

## 7. Performance plan

The reference is ecnerwala's `static_forest_t`: `int` arrays, one child
CSR, RMQ LCA. `Rooted` will match its layout where that never loses:

- depths, sizes and positions are range-typed (`0..V.count`), so they take
  four bytes when `V` fits (the general width rule, no special case);
- children move from per-vertex lists to one CSR array over `Pre` (children
  of `v` are contiguous after its heavy child), halving construction memory;
- `far` is stored only by `weighted`; `new` reads `far` as `depth`.

LCA stays chain jumping: O(log n) worst case but two or three jumps on
random trees, O(n) memory instead of the O(n log n) of a sparse table.
Matched benchmarks (bench/tree/) build a 1e6-vertex tree and run 1e6 LCA,
`dist`, `ancestor`, subtree and path-fold queries against the C++
equivalent with the same algorithm; target 1.1x.

## 8. Decisions

Logged in `decisions.md` under "trees":

- one `Rooted` value owns the heavy-first preorder; no HLD, Euler or LCA
  types (kept from the first `Rooted` decision);
- `up`/`reroot` share a signature, `down` replaces the preorder-and-parent
  loop; no `top_down` returning pairs (one closure, one result per vertex);
- `heavy`/`light`/`subtree` are the small-to-large primitives; no
  `dsu_on_tree` higher-order function, because the keep/clear logic is the
  algorithm and must stay visible;
- `Centroids` is a separate struct built from adjacency, because it is a
  different tree (its parent is not the rooted parent);
- `step(a, b, 1)` covers "next vertex toward", no `toward`;
- dynamic trees are a separate `LinkCut` with the same verbs, after the
  static API is fast.
