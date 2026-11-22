# Round 4: graphs and trees

Built on rounds 1 to 3: vertices and edges are domains, per-vertex data is
`[V]T`, generators drive traversals, DP tables are typed by their state.

## The model in one paragraph

Vertices and edges are separate domains, so `[V]T` and `[E]T` never mix.
Every algorithm takes a **`Next<V>`**: anything that walks the arcs out of a
vertex. A stored `Graph`, a `[V]List` of arcs and any fn from a vertex to a
`Seq` of arcs all are one, so grids, state spaces and filtered graphs need
no adjacency build. An arc is anything with `.to`; what else it carries
decides which algorithms take it: `.w` for shortest paths, `.e` for
bridges. Weights live on the arcs or come from `g.weighted(f)`; no
algorithm takes a `w:` argument. Results that name new groups (components,
blocks) return a fresh domain. A tree is `Rooted`: one structure with a
fresh preorder domain `t.Pre` in which every subtree and every heavy chain
is one range, so range structures (segment trees, Fenwicks) index it
directly. Decision 146 records the design and its reasons.

## Graphs

```python
domain V = 1..=n
domain E = 0..m
let edges = read<[E](u: V, v: V, w: 1..=1e9)>()
let g = Graph.undirected(edges)       // or .directed; immutable CSR, O(n + m)

for a in g[u]: relax(a.to, a.w)       // arcs out of u, input order; a is (to, e, w)
g.deg(u)                              // O(1)
```

| Form | Meaning | Cost |
|---|---|---|
| `Graph.directed(edges)`, `Graph.undirected(edges)` | build from any `[E]` of records with `u: V, v: V` | O(n + m) |
| `keep: a => (to: a.to, w: a.w)` | choose what each arc stores; run once per arc | O(n + m) |
| `g[u]` | `Indexed` view of `u`'s arcs, in input order | O(1) |
| `g.deg(u)` | arcs out of `u` | O(1) |
| `g.next(u)` | where `u`'s arcs lead (`g.out(u)` mapped to `.to`) | O(1) per arc |
| `g.weighted(f)` | the same graph with `.w` read as `f(a)` | O(1); `f` per read |
| `g.transpose()` | arcs reversed, other fields kept | O(n + m) |
| `g.only(keep)` | induced view: arcs touching a vertex `keep` rejects are skipped | O(1) to make |
| `g.matrix()` | `[V, V]?W`, least weight per pair | O(n² + m) |
| `Graph.of(arcs)` | build from any `Seq` of `(from: V, arc: A)` | O(n + m) |

### Storage

`Graph<V, A>` is compressed sparse rows: n + 1 offsets and one list of arcs,
each vertex's arcs contiguous and in input order (a counting sort, so DFS
order and lexicographic answers are deterministic). An undirected edge is
two arcs with the same `e`; a loop gives two arcs, so degree counts it
twice. Integer widths come from the domains.

**What an arc stores.** By default an arc is the edge record with `u`, `v`
replaced by `to` and `e`, `EdgeArc<V, E, R> = (to: V, e: E, ..R without (u,
v))`, so `a.w` reads the arc you are already on (the `adj[u].push_back({v,
w})` layout). `keep:` narrows or computes:

```python
Graph.undirected(edges)                                 // (to, e, w): 12 bytes at these widths
Graph.undirected(edges, keep: a => (to: a.to, w: a.w))  // no e: not Edged, 8 bytes
Graph.directed(edges, keep: a => (to: a.to))            // bare targets
Graph.undirected(edges, keep: a => (to: a.to, e: a.e, fwd: a.to == edges[a.e].v))
```

`keep` runs once per stored arc at build (2m times for undirected), and
what it returns is stored: `a.fwd` is a field load later. The returned
record decides the capabilities: no `e`, not `Edged`, and `bridges(g)` is a
compile error that says to keep `e`. Carried fields are copies of plain
fields; a heap field (`name: Str`) is a compile error at the build that
points at `keep:`, where you write `.clone()` yourself. The graph does not
hold `edges`; `edges[a.e]` still works for rarely read fields.

**No `add_edge`.** A graph that grows is the plain container, already a
`Next`:

```python
var adj: [V]List<(to: V, w: Int)> = []
adj[u].push((to: v, w: c))
dijkstra(adj, from: [(s, 0)])
```

Rejected: `[V]List` inside `Graph` (one allocation per vertex), forward-star
lists (poor locality), arcs of `(to, e)` only with fields read from
`edges[e]` (a cache miss per arc), `e` in a side table (rarely worth it).

Validated input types (`Tree<V>`, `SimpleGraph<V, E>`, `ConnectedGraph`,
`Dag`, `Digraph`) read like the plain edge list and also tell `cpl stress`
what to generate and shrink.

### Traits

```python
pub trait Next<V>:
    type Arc: Target<V>
    fn out(self, u: V) -> yields Self.Arc

pub trait Target<V>:   @get fn to(self) -> V    // a plain vertex is its own target
pub trait Weighted<W>: @get fn w(self) -> W
pub trait Edged<E>:    @get fn e(self) -> E

pub trait WeightedNext<V, W> = Next<V> where Self.Arc: Weighted<W>
pub trait EdgedNext<V, E>    = Next<V> where Self.Arc: Edged<E>
```

A record with a `to`, `w` or `e` field satisfies the matching trait (a field
is a `@get`); a named struct opts in with a bodiless `impl Weighted<Int>
for Road`. Signatures use inline bounds and take the weakest capability
they use:

```python
pub fn dijkstra<V, W>(g: WeightedNext<V, W>, from: Seq<(V, W)>, until: ?V = none)
```

| Takes | Algorithms |
|---|---|
| `Next` | BFS, DFS, topological order, SCC, components, matching, cycles |
| `WeightedNext` | Dijkstra, 0-1 BFS, Dial, Bellman-Ford, Johnson, negative cycles |
| `EdgedNext` | bridges, two-edge components, edge blocks |
| edge records `(u, v, w, ..)` | Kruskal, arborescence, Euler walks, offline connectivity |
| `[V, V]` matrix | Floyd-Warshall |
| own mutable struct | flows |

Whole-graph algorithms (SCC, components, bridges) also need `V: Domain`.

### Customizing: a lambda over `g.out(u)`

```python
dijkstra(u => g.out(u).filter(a => !banned[a.e]), from: [(s, 0)])  // arcs keep .w, .e
bfs(u => g.out(u).filter(a => a.w <= mid), from: [s])              // binary search on the answer
bfs(c => c.adj4().filter(c => grid[c] != '#'), from: [start])      // implicit grid
dijkstra(g.weighted(a => 2 * a.w), from: [(s, 0)])                 // reweight; keeps .to, .e
dijkstra((v, k) => chain(                                          // state space (vertex, coupons)
    g.out(v).map(a => (to: (a.to, k), w: a.w)),
    g.out(v).filter(_ => k < K).map(a => (to: (a.to, k + 1), w: a.w / 2)),
), from: [((s, 0), 0)])
```

- `filter` keeps the arc type, so a filtered graph is still `Weighted` and
  `Edged`; `map` to a record makes new arcs.
- Nothing is allocated: the lambda is a generator, and captures are lent
  per call.
- One way per job: no `g.filter_arcs` or `g.restrict`. `.weighted(f)`
  exists because it forwards `.e` for you; `.weighted((u, a) => ...)` also
  passes the source vertex (Johnson potentials).
- **Storage follows the vertex type.** A domain gives `[V]` arrays; any other
  hashable type gives hash maps. `cpl explain` shows which.

### Shortest paths

All return `Paths<V, W, A>`:

| On `p` | Meaning |
|---|---|
| `p[v]` | distance, `?W` (`none` when unreachable) |
| `p.back[v]` | `?(from: V, via: A)`, the step that reached `v` |
| `p.path(t)`, `p.arcs(t)` | one shortest path, `?List` |
| `p.order` | reached vertices in settling order |

| Form | Weights | Cost |
|---|---|---|
| `bfs(g, from:, until:)` | unit | O(n + m) |
| `bfs01(g, from:, until:)` | 0 or 1 | O(n + m) |
| `dial(g, from:, max_w:, until:)` | small ints | O(n · max_w + m) |
| `dijkstra(g, from: [(s, d0)], until:)` | non-negative | O(m log n) |
| `bellman_ford(g, from:)` | any; `none` on a negative cycle | O(n m) |
| `floyd_warshall(mut d, over: Semiring(min, +)) -> Bool` | dense, in place; `false` on a negative cycle; `over: Semiring(max, min)` gives widest paths | O(n³) |

`until: t` stops once `t` is settled. `from:` is any `Seq`; multi-source is
a longer one. `p.tight(g)` is the shortest-path DAG.

All-pairs on a dense graph starts from the weight matrix:

```python
var d = g.matrix()                          // [V, V]?W, O(n² + m)
if not floyd_warshall(mut d): print("NEGATIVE CYCLE")
```

### Traversal generators

```python
for step in dfs(g, from: [s]):
    match step:
        Enter(v) => if seen_enough(v): step.prune()
        Exit(v) => done[v] = true
```

`dfs` yields `Enter(v)` / `Exit(v)` and supports `step.prune()`, `break`
and `return` (round 2's generators). A DFS tree with edge
classification is `g.dfs_tree(root)`.

### Structure

Every grouping returns a fresh domain and the map into it:

| Form | Result | Cost |
|---|---|---|
| `components(g)` | `(domain C, comp: [V]C)` | O(n + m) |
| `complement(g)` | the complement graph of the missing arcs `g`: `bfs`, `dfs`, `reachable` and `components` over it cost O(n + m) | O(1) to make |
| `scc(g)` | `(domain C, comp: [V]C)`, `C` in topological order | O(n + m) |
| `condense(g, comp)` | `Graph<C, _>` of the components | O(n + m) |
| `two_edge_components(g)` | `(domain C, comp: [V]C)` | O(n + m) |
| `blocks(g)` | `(domain B, block: [E]B)`, biconnected blocks of edges | O(n + m) |
| `bridges(g)`, `cut_vertices(g)` | `[E]Bool`, `[V]Bool` | O(n + m) |
| `topo(g)` | `?List<V>`, `none` on a cycle | O(n + m) |
| `two_color(g)` | `?[V]Bool`, `none` when an odd cycle exists | O(n + m) |
| `find_cycle(g)` | `?List<V>` | O(n + m) |
| `kruskal(edges)` (records with `w`) | `(total, take: [E]Bool)` | O(m log m) |
| `euler_walk(edges, directed:)` | `?Walk<V, E>` | O(n + m) |
| `TwoSat<X>()`, `.add_or(a, b)`, `.solve()` | `?[X]Bool` | O(n + m) |
| `Functional(next: [V]V)` | `.after(v, k)`, `.on_cycle(v)`, `.cycle_len(v)`, `.root_of(v)` | O(n log n) build |

Flows and matchings are their own module (`std.flow`), on the same
`V`/`E` domains.

## Trees

```python
let t = Rooted(g, root: V.first!)          // or Rooted.from_parents(par), Rooted.forest(g, from:)
```

| Form | Meaning | Cost |
|---|---|---|
| `t.parent(v)` | `?V` | O(1) |
| `t.child_arcs(v)` | the arcs from `v` to its children, with their fields | O(1) per child |
| `Lift.of(next)` | binary lifting over any partial successor `V -> ?V` (a greedy "next pick") | O(n log n) build |
| `t.up_arc(v)` | `?A`: the arc `v` was entered by, with its fields (`t.up_arc(v)!.cost`) | O(1) |
| `t.children(v)` | `Indexed` view of `v`'s children (stored contiguously) | O(1) |
| `t.depth(v)`, `t.size(v)` | edges from the root; vertices in the subtree | O(1) |
| `t.order` | every vertex, parents before children (BFS order); `.rev` is children first | O(1) |
| `t.lca(a, b)`, `t.dist(a, b)`, `t.is_anc(a, b)` | | O(log n) |
| `t.ancestor(v, k)`, `t.jump(a, b, k)` | `?V`: k steps up; k steps along the path a → b | O(log n) |
| `t.path(a, b)` | `List<V>` | O(length) |
| `t.diameter()`, `t.centroid()` | `(len, u, v)`; a centroid | O(n) |

**Tree DP without recursion** walks `t.order.rev`:

```python
var size: [V]Int = 1
for v in t.order.rev:
    for c in t.children(v): size[v] += size[c]
```

The library forms for the shapes that recur:

| Form | Meaning | Cost |
|---|---|---|
| `t.up(op, lift:, finish:)` | bottom-up DP: fold children with `op` | O(n) calls |
| `t.reroot(op, lift:, finish:)` | the same DP for every root at once; no inverse needed | O(n) calls |
| `t.down(init, f)` | top-down: `out[v] = f(out[parent], v)` | O(n) calls |

Recursion stays available and natural (`for c in t.children(v): mine.absorb(go(c))`).

### Subtrees and paths are ranges

`t.Pre` is a fresh domain per tree: an Euler order where every subtree and
every heavy chain is contiguous. `t.pos(v): t.Pre`; another tree's `Pre`
doesn't mix.

```python
var seg = SegTree(max, t.reorder(val))     // [t.Pre]T
seg.fold(t.sub(v))                         // subtree query: one range
t.fold_path(a, b, seg)                     // path query: O(log n) ranges (HLD)
t.apply_path(a, b, lazy, add(x))
for r in t.chains(a, b): ...               // the ranges themselves, when you need them
```

| Form | Meaning | Cost |
|---|---|---|
| `t.sub(v)`, `t.below(v)` | `Range<t.Pre>`: subtree with / without `v` | O(1) |
| `t.chains(a, b)` | `yields Range<t.Pre>`, ordered from `a` to `b` | O(log n) ranges |
| `t.fold_path(a, b, ds, edges:)`, `t.apply_path(...)` | path fold / update over any range structure | O(log n) ranges |
| `t.tour` | `yields Visit<V>`: `Enter` / `Exit` events | O(1) to make, O(n) to walk |
| `t.virtual(keys)` | the virtual (auxiliary) tree of `keys` | O(k log k) |
| `Centroids(g)` | centroid decomposition: `.parent(c)`, `.ancestors(v)` with distances | O(n log n) |

### Changing trees

`LinkCut<V, T>(op)`, `LazyLinkCut`, `EulerTourTree`, `JumpTree` (online
binary lifting as leaves are added). Same vertex domain, same method names
as `Rooted` where they mean the same thing (`lca`, `dist`, `jump`,
`fold_path`).

### Also in the prelude

| Form | Meaning | Cost |
|---|---|---|
| `Lit(x, value)` | a 2-SAT literal, "x is `value`", for `TwoSat` | O(1) |
| `p.adj4`, `p.adj8`, `p.ray(d)`, `Dir4`, `Dir8` | grid neighbours of a `(Row, Col)` cell inside the grid; the cells from `p` to the edge in direction `d` | O(1) per cell |
| `kruskal_tree(es, w:)`, `KruskalTree` | the reconstruction tree over a fresh node domain; `reach(v, limit:)` is one subtree, `top(v, ok:)` tests the merging edges themselves, `up(op, leaf:)` folds leaf values per node; flip the order with `w: e => desc(e.w)` | O(m log m), O(log n) per `reach` |

## Open

1. **Weight algebra.** Dijkstra's distance is `Ord + Add + Zero` today.
   Ryan leans to a monoid-like argument so minimax, bottleneck and
   reliability paths are one call; not designed yet.
2. **Hash-backed state spaces.** Storage follows the vertex type: a
   domain gives arrays, anything hashable gives maps. Alternative: require
   a domain and make the user `compress` first.
3. **Tuples as edges.** Builds need named records (`read<[E](u: V, v: V)>`);
   several dreams still pass unnamed `(V, V)` tuples and need the one-line
   `map` that names them.
4. **Names that drift today:** `articulation_points` → `cut_vertices`,
   `biconnected_components` / `edge_blocks` → `blocks`, `Adj` and
   `*_of_pairs` builders → `Graph` or `[V]List`, `subtree_sizes` →
   `t.size(v)`, `preorder` → `t.order` (BFS) plus `t.tour`.
5. **DFS and trees: agreed (decision 147), rewrite queued.** `step.prune()`,
   events carry the arc, `Again(from, via)`, shared `seen:`, free
   `reachable`, `g.dfs_tree` → `(tree: Rooted, tree_edge: [E]Bool)`.
   Derived tree structures (`LcaTable(t)`, `Lift(t, ...)`, `Centroids(t)`)
   own their data; `t` stays free (05-ownership.md §5).
