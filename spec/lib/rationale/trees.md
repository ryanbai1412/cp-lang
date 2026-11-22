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
| positions in the order | `t.Pre` | fresh per tree; `t.pos(v)`, `t.at[p]` |
| subtree, chain | `Range<Pre>` | `t.sub(v)`, `t.chains(a, b)` |
| cuts between positions | `Pre.Cut` | `r.start`, `r.end` of those ranges |
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
    fn from_parents(par: [V]?V)                            // exactly one None
    fn forest<N>(adj: N, roots: List<V>) [new]            // one tree per root
```

`adj` is anything with `Neighbours` (a `Graph`, `[V]List<V>`, a closure),
so `Rooted(Graph.undirected(read<Tree<V>>()), root: V.first)` reads a tree.
`new` on a graph with cycles roots its breadth-first tree (documented,
tested), so `depth` is the distance from the root; back edges and low-links
are graph algorithms (`std.graph`). A vertex the search never reaches traps;
a forest says so with `forest`.

### 3.2 Shape (all O(1))

```cp
    root: V
    fn parent(self, v: V) -> ?V
    fn children(self, v: V) -> List<V>            // in the order adj listed them
    heavy: [V]?V                                  // largest child, None at a leaf
    fn light(self, v: V) -> yields V [new]        // children except heavy[v]
    fn depth(self, v: V) -> Int                   // edges to the root
    far: [V]Int                                   // length to the root (weighted)
    fn size(self, v: V) -> Int                    // vertices in the subtree
    fn root_of(self, v: V) -> V [new]             // the root of v's tree (forests)
    @get fn subtree_sizes(self) -> [V]Int         // O(n), a copy
```

### 3.3 Orders and positions

```cp
    domain Pre                                   // heavy-first preorder
    fn pos(self, v: V) -> Pre                    // O(1)
    at: [Pre]V                                   // O(1), inverse of pos
    fn sub(self, v: V) -> Range<Pre>             // O(1), t.sub(v).len == t.size(v)
    fn below(self, v: V) -> Range<Pre>           // O(1), sub(v) without v
    fn subtree(self, v: V) -> yields V [new]     // O(size(v)), preorder
    fn reorder<T>(self, a: [V]T) -> [Pre]T       // O(n)
    @get fn tour(self) -> yields (v: V, entering: Bool)   // O(n), 2n events
```

### 3.4 Ancestry and paths

```cp
    fn is_anc(self, a: V, b: V) -> Bool          // O(1); a is_anc a
    fn lower(self, u: V, v: V) -> V              // O(1); the child end of edge uv
    fn lca(self, a: V, b: V) -> V                // O(log n), chain jumps
    fn dist(self, a: V, b: V) -> Int             // O(log n), edges
    fn far_dist(self, a: V, b: V) -> Int [new]   // O(log n), weighted length
    fn ancestor(self, v: V, k: Int) -> ?V        // O(log n); None above the root
    fn step(self, a: V, b: V, k: Int) -> ?V      // O(log n); k-th vertex a -> b
    fn lift_while(self, v: V, pred: V -> Bool) -> ?V  // O(log^2 n), pred monotone
    fn path(self, a: V, b: V) -> List<V>         // O(dist), a first
```

`step(a, b, 1)` is "the neighbour of `a` toward `b`" (ecnerwala's
`get_next`, and `get_subtree` when `b` is below `a`), so neither gets a name.
`lift_while(v, pred)` returns the highest ancestor `u` of `v` (possibly `v`)
with `pred` true on the whole `v..u` path, or `None` if `pred(v)` is false.

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
    // O(k log k + k log n): marked vertices plus pairwise LCAs over a fresh
    // domain vt.K in preorder; vt.at[x] the vertex, vt.up[x] the nearest kept
    // ancestor, vt.len[x] = t.dist(vt.at[up], vt.at[x]), vt.key[x] a key?
    fn virtual(self, keys: List<V>) -> VirtualTree<V, K>

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

### 3.9 Dynamic trees

```cp
struct LinkCut<V, T>
    fn new<M>(op: M, a: ?[V]T = None)             // O(n), like SegTree.new
    fn link(mut self, u: V, v: V)                 // amortized O(log n) each
    fn cut(mut self, u: V, v: V)
    fn connected(mut self, u: V, v: V) -> Bool
    fn root_of(mut self, v: V) -> V
    fn make_root(mut self, v: V)
    fn fold_path(mut self, u: V, v: V) -> T       // path order when Reversible
    fn get(self, v: V) -> T                       // O(1)
    fn set(mut self, v: V, x: T)
    fn lca(mut self, u: V, v: V, root: V) -> ?V
```

Path updates are a second type, as `LazySegTree` is to `SegTree`, so a plain
`LinkCut` pays for no tags: `LazyLinkCut<V, T, F>.new(act, a)` has the same
verbs plus `apply_path(u, v, f)`, under the same `Action`. `get` splays
because tags are pending. Reversing a path must commute with a tag,
`apply(f, x.rev(), k) == apply(f, x, k).rev()`, true for commutative `over`
and for assignment.

A tree that only grows, one leaf at a time with its parent already in place
(greedy "next jump" trees built right to left, offline insertions), needs
neither `Rooted`'s full build nor splaying:

```cp
struct JumpTree<V>
    fn new() -> JumpTree<V>                       // O(n), every vertex a root
    fn add(mut self, v: V, p: V)                  // O(1), v a lone vertex
    fn parent(self, v: V) -> ?V                   // O(1)
    fn depth(self, v: V) -> Int                   // O(1)
    fn root_of(self, v: V) -> V                   // O(log n)
    fn ancestor(self, v: V, k: Int) -> ?V         // O(log n) each
    fn lca(self, a: V, b: V) -> ?V                // None across trees
    fn lift_while(self, v: V, pred: V -> Bool) -> ?V
```

A permutation's interval structure is a tree too. `PermTree.of(p)` builds the
binary permutation tree of a bijection `p: [I]V` in one O(n) stack pass:
every node covers a run of positions, its children split the run, and its
values form an interval unless the node is `Partial` (a run that closes only
further up, as a `Full` node whose right child fills its gaps). Nodes get a fresh domain of 2n - 1 members, so the per-node
facts are arrays over it and a leaf's missing children are a trap, not a
`-1`:

```cp
enum PermKind: Leaf, Incr, Decr, Partial, Full
struct PermTree<I, V, Node>
    root: Node; kind: [Node]PermKind; first, last: [Node]I; lo, hi: [Node]V
    fn of(p: [I]V) -> PermTree<I, V, _>           // O(n)
    fn leaf(self, i: I) -> Node                   // O(1)
    fn left(self, v: Node) -> Node                // O(1), traps on a leaf
    fn right(self, v: Node) -> Node
    fn span(self, v: Node) -> Range<I>            // positions
    fn vals(self, v: Node) -> Range<V>            // exact unless Partial
```

Folds over it are plain recursion (`cf2187f1`, `cf2187f2`). A top-level `fn`
cannot yet name `t.Node` in its signature (the core gap logged in
`research/frictions/cf2183f.md`), so today it is written generic,
`fn depth<Node>(v: Node)`.

Skew-binary jump pointers (one jump per vertex, Myers 1983) keep it at three
arrays; the queries mean what `Rooted`'s do, so a port switches between them
by changing the constructor.

Every query restructures the splay trees, so even reads take `mut self`.
Path updates (`apply_path` with an `Action`) and subtree aggregates are the
next steps; edge values are a vertex per edge, as in every LCT.

Same verbs as `Rooted` where they mean the same thing (`fold_path`, `lca`),
same monoid convention as every range structure.

## 4. Laws

Every law is a property test in `stdlib/tree.cp`.

1. `t.pos(b) in t.sub(a)` iff `t.is_anc(a, b)`; `t.sub(v).len == t.size(v)`.
2. `t.at[t.pos(v)] == v`; `t.reorder(a)[t.pos(v)] == a[v]`.
3. If `h = t.heavy[v]` then `t.pos(h)` is `t.pos(v)` plus one, and
   `t.size(h) >= t.size(c)` for every child `c`.
4. `t.dist(a, b) == t.depth(a) + t.depth(b) - 2 * t.depth(t.lca(a, b))`.
5. `t.step(a, b, k)` is `t.path(a, b)[k]` for `0 <= k <= t.dist(a, b)`,
   `None` past `b`; `t.ancestor(v, k) == t.step(v, t.root, k)`.
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
- A parent array in label order (every parent has a smaller, or every one a
  larger, label: cf2138c1, cf2187c, cf2089e). Then `for v in V` already visits
  parents first and `for v in V.rev` children first, and `dep[v] = dep[p] + 1`
  is the whole traversal. Building a `Rooted` for it computes the heavy
  order nobody asked for: cf2138c1 at n = 10^6 took 0.27 s / 160 MB against
  0.06 s / 66 MB for the loop, with identical output.

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

let assign = Action(None, (f, g) => f ?? g, (f, s, len) => f ?? s, over: |)
var colours = LazySegTree(assign, t.reorder([V] v => 1 << c[v]))
for _ in 0..read(1..=4e5):
    match read(1..=2):
        1 => colours.apply(t.sub(read<V>()), Some(1 << read<1..=60>()))
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

var near: [V]?Int = None
fn paint(v: V):
    for (c, d) in cd.ancestors(v): near[c] = Some(min(near[c] ?? d, d))
paint(V.first)
for _ in 0..q:
    let (kind, v) = read<(1..=2, V)>()
    if kind == 1: paint(v)
    else: print(min(near[c]! + d for (c, d) in cd.ancestors(v) if near[c] != None))
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
    if let h = t.heavy[v]: solve(h, keep: true)
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
  static API is fast; trees that only grow are `JumpTree`, same verbs again.

## Coverage

Topics a strong contest library covers for trees: AtCoder Library (which
leaves trees to the user over its segment trees), KACTL's tree chapter
(LCA, binary lifting, HLD, link-cut, centroid, compress tree, directed
MST), cp-algorithms' tree section, ecnerwala's `static_forest_t`, and other
top contestants' personal libraries. `have` means a shipped spelling in
`plan/STDLIB.md` 4.11, `new` means a `[new]` signature there, `out` gives the
reason.

| Topic | Status | Spelling or reason |
|---|---|---|
| Rooted tree from adjacency, parents, weights | have | `Rooted(adj, root)`, `Rooted.from_parents`, `w:` |
| Forests | have | `Rooted.forest(adj, roots)`, `root_of` |
| Parent, children, depth, size, heavy, light | have | `parent`, `children`, `depth`, `size`, `heavy`, `light` |
| Euler tour flattening, subtree as range | have | `pos`, `at`, `sub`, `below`, `reorder` |
| Enter/exit events | have | `tour` |
| LCA by chain jumping | have | `lca`, `is_anc`, `lower` |
| Distance, weighted distance | have | `dist` |
| k-th ancestor, k-th vertex on a path | have | `ancestor`, `jump` |
| Binary search up the ancestors | have | `jump_while` |
| Path as vertices | have | `path` |
| HLD path fold and update, edge mode, direction | have | `chains`, `fold_path`, `apply_path`, `edges:` |
| Subtree fold and update | have | any range structure over `t.Pre` with `sub(v)` |
| Bottom-up and top-down DP | have | `up`, `down` |
| Rerooting DP | have | `reroot` |
| Sibling-exclusive folds | have | `excl_fold` |
| Diameter | have | `diameter` |
| Centroid(s) | have | `centroid`, `centroids` |
| Centroid decomposition | have | `Centroids` |
| Virtual (auxiliary) tree | have | `virtual` |
| Small-to-large, DSU on tree | have | keep `heavy`, loop `light`, `sub` ranges |
| Rooted isomorphism (AHU) | have | `shapes` |
| Link-cut path fold, reroot | have | `LinkCut` |
| Link-cut path update | have | `LazyLinkCut` |
| Leaf-growing tree with jumps | have | `JumpTree` |
| Permutation tree | have | `PermTree` |
| Jumps on a functional graph | have | graphs' `Functional.jump` |
| Path k-th by persistence | have | `down` with ranges' `PersistentSegTree` |
| Subtree k-th, rank, distinct | have | ranges' structures over `t.Pre` |
| Heavy chain head and chain range (dynamic DP) | new | `top`, `chain` |
| Subtree under another root | new | `sub_at` |
| On-path test, three-path meet | new | `on_path`, `meet` |
| Per-edge rerooting (the part above v) | new | `above` |
| Online diameter of a vertex set | new | `far_pairs` monoid |
| Descendants at a depth | new | `Bfs`, `by_depth`, `layer` |
| Mo on trees | new | `Ev`, `enter`, `exit`, `path_events` |
| Long-path decomposition | new | `long_paths`, `LongPaths` |
| O(1) LCA | new | `LcaTable` |
| Link-cut distance and k-th on path | new | `LinkCut.dist`, `LinkCut.jump` |
| Growing-tree distance | new | `JumpTree.dist` |
| Euler tour tree | new | `EulerTourTree`, `LazyEulerTourTree` |
| Cartesian tree | new | `cartesian_tree` |
| Kruskal reconstruction tree | new | `kruskal_tree`, `KruskalTree` |
| Prüfer code | new | `prufer`, `from_prufer` |
| Top tree | out | rare, and `EulerTourTree` with `LinkCut` covers its contest uses within 64 KB |
| Fully dynamic connectivity (HDLT) | out | rare, big constant; offline uses containers' segment-tree-over-time |
| Offline Tarjan LCA | out | `lca` is online at O(n) memory; no port needed offline |
| O(1) level ancestor (ladders) | out | `ancestor` is a few chain jumps; ladders cost O(n log n) memory |
| Randomized tree hashing | out | `shapes` is exact at the same cost |
| Unrooted isomorphism | out | two lines: `centroids` then `shapes` (section 3) |
| Every vertex's eccentricity | out | one line from `diameter` (law 9) |
| Centre and radius | out | `jump(u, v, len / 2)` on the diameter |
| Tree knapsack size-bounded merge | out | the bounded loop is the algorithm; `up` carries the lists |
| Random tree generators | out | owned by the stress generators |
| Block-cut tree, dominator tree | out | graphs (`design/stdlib/graphs.md`); they return a `Rooted` |
| Gomory-Hu tree | out | flows (`design/stdlib/flows.md`) |
| Minimum spanning tree | out | `mst` in `std.graph`; the tree side is `kruskal_tree` |
| Directed MST (Chu-Liu/Edmonds) | out | graphs |
| Sqrt blocking of a tree | out | `path_events` with Mo covers its uses |
| Automorphism counting | out | rare; `shapes` gives the classes to count with |
| Edge index helper | out | `lower(u, v)` names an edge by its lower vertex |
| A `dsu_on_tree` higher-order function | out | keep/clear is the algorithm (section 5) |

Counts: 27 have, 15 new, 18 out.

## Coherence

- **Construction.** Every type in the area is built by calling its name:
  `T(args)` is `T.new(args)`. Options are named arguments with defaults
  (`w:`, `edges:`). A second constructor exists only for a different input
  shape and is named `T.from_<input>` (`Rooted.from_parents`) or by the
  shape (`Rooted.forest`). Drift fixed: `Rooted.weighted(adj, root, len)`
  becomes `Rooted(adj, root, w:)`; `PermTree.of(p)` becomes `PermTree(p)`.
  Builders from another kind of input are free functions named for what
  they build (`cartesian_tree`, `kruskal_tree`, `from_prufer`).
- **Arguments.** Path endpoints are `a, b`; edge endpoints `u, v`; a root
  is `root:` or `roots:`; a vertex count of steps is `k`; an edge weight is
  `w`; a monoid is `op`, as in `SegTree` and `LinkCut` (was `m` in `up`,
  `reroot`, `excl_fold`); a predicate is `ok`; an initial value is `init`.
  Positions are `Range<Pre>`, and cuts `t.Pre.bounds` (decision 124).
- **Verbs.** Folds are `fold_*`, updates `apply_*`, both taking the range
  structure, as in ranges. Jumps are `jump` and `jump_while`, the words
  graphs' `Functional` uses (was `step`, `lift_while`).
- **Renames** keep a deprecated alias until a codemod moves the ports.

## Cross-area

| Name | Other area | Resolution |
|---|---|---|
| `jump`, `jump_while` | graphs `Functional.jump` | same meaning; trees renamed to match |
| `excl_fold` | seq `excl_fold` | same meaning; trees uses `op`, seq should too |
| `fold_path`, `apply_path` | ranges `fold`, `apply` | the path form of the range verbs; takes the range structure |
| `dist` | graphs `bfs`/`dijkstra` `.dist` | same meaning, method here, array there |
| `path` | graphs path reconstruction | both return `List<V>` |
| `Bfs` (domain on `Rooted`) | graphs `bfs` function | different kinds (domain, function); kept |
| `w` (`VirtualTree.w`, `KruskalTree.w`) | graphs edge `w` | same meaning |
| `centroids` (method) | `Centroids` (type) | within trees: the method answers the one or two vertices |
| `cartesian_tree` | seq previous/next smaller | trees owns the tree; seq's scans are how users build it today |
| `kruskal_tree` | graphs `mst`, containers `Dsu` | trees owns the tree; `mst` stays the forest of edges |
| `far_pairs` | ranges `Monoid` | a monoid value, usable in every range structure |
| `Tree<V>` | graphs input aliases | the shared edge-list input type in the prelude |
| block-cut, dominator, Gomory-Hu trees | graphs, flows | their owners; results are `Rooted` |
| `PersistentSegTree` path k-th | ranges | composition, no tree name |

## Powers

Signatures in 4.11 that today's public language cannot write as specified:

1. **Protocols with associated types and default methods.**
   Needed by `fold_path`, `apply_path` (`ds: S` should be bound by a range
   fold protocol with its value type, so errors name the call, not the body)
   and by `LinkCut.dist`, `LinkCut.jump` (written once for `LinkCut` and
   `LazyLinkCut`). Smallest example:
   `fn fold_path<S: Fold<Pre>>(self, a: V, b: V, ds: S) -> S.Out`.
2. **Generators steered by their items** (decision 126). Needed by
   `subtree` and `tour` to skip a subtree mid-walk. Smallest example:
   `for (v, step) in t.subtree(r): if bad(v): step.prune()`.
3. **Niches.** Needed for the memory the costs promise in `parent`
   (`[V]?V`), `cartesian_tree` (`[D]?D`), `VirtualTree.parent` and
   `KruskalTree.w`: `?V` stored in the width of `V`. Smallest example:
   `let par: [V]?V` taking 4 bytes per vertex, as the C++ `int` with -1.

Fresh per-value domains (`Bfs`, `Ev`, `L`, `N`) use the mechanism `Pre` and
`VirtualTree`'s `K` already use, so they need no new power.

## Dreams

Tree dreams in `proto/ports/*/dream/` (all under `ecnerwala`), the tree APIs
each uses, and checker errors on main that are not tree APIs.

| Dream | Tree APIs | Other errors (owner) |
|---|---|---|
| cf1984e | `Rooted`, `reroot` | record-of-monoids literal `(count: +, taken: or)` (ranges) |
| cf2035f | `Rooted`, `up` | clean |
| cf2062e1 | `Rooted`, `is_anc`, `lca` | `with io(base: 1)` context (core) |
| cf2062e2 | `Rooted`, `at`, `lca`, `pos`, `sub` | `with io(base: 1)` context (core) |
| cf2089e | `from_parents`, `path` | `Poly` (math); its `next` is user code |
| cf2101e | `Centroids`, `piece` | `merge` unknown (seq) |
| cf2124i | `from_parents`, `children` | `V.windows(2)` destructuring (seq) |
| cf2135f | `Rooted(adj, root:)`, `Pre.rev` | `PMultiset` (containers), an argument parse error |
| cf2152g | `Rooted`, `is_anc`, `reorder`, `sub` | `Flip` action shape (ranges) |
| cf2183d2 | `Rooted`, `depth`, `parent` | `List.max` (seq) |
| cf2183f | `Tree`, `Rooted`, `at`, `below`, `pos`, `sub` | clean |
| cf2187f1, cf2187f2 | `PermTree.of` (spec: `PermTree(p)`) | lowercase variant patterns (syntax) |
| cf2222g | `Rooted`, `at`, `centroid`, `children`, `subtree`, `subtree_sizes` | `List.counts` (seq), names from it |
| cf2237h | `Rooted`, `chains`, `pos` | named tuple parameters (syntax) |
| cf2237i1, cf2237i2 | `from_parents`, `at`, `children` | clean |

Every tree API these dreams use is in the spec; the two spelled differently
are `PermTree.of` and `Rooted.weighted`, both kept as aliases.
