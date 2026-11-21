# Round 4: graphs and trees

Built on rounds 1 to 3: vertices and edges are domains, per-vertex data is
`[V]T`, generators drive traversals, DP tables are typed by their state.

## The model in one paragraph

Vertices and edges are separate domains, so `[V]T` and `[E]T` never mix. A
graph is stored once, as `Graph<V, E>` (compressed rows), and `g[u]` is an
O(1) indexed view of the arcs leaving `u`; an arc is `(to: V, e: E)`.
Every algorithm takes **`next`**: a stored graph, or any function from a
vertex to what follows it (vertices or arcs), so grids, state spaces and
filtered graphs need no adjacency build. Weights are read through a
function of the arc, so one graph serves several weightings. Results that
name new groups (components, blocks) return a fresh domain. A tree is
`Rooted`: one structure with a fresh preorder domain `t.Pre` in which every
subtree and every heavy chain is one range, so range structures (segment
trees, Fenwicks) index it directly.

## Graphs

```python
domain V = 1..=n
domain E = 0..m
let edges = read<[E](u: V, v: V, w: 1..=1e9)>()
let g = Graph.undirected(edges)           # or Graph.directed(edges)

for (to: v, e) in g[u]: ...               # arcs out of u; an undirected edge is two arcs, same e
g[u].len                                  # degree, O(1)
```

| Form | Meaning | Cost |
|---|---|---|
| `Graph.directed(edges)`, `Graph.undirected(edges)` | build from any `[E](u: V, v: V, ...)` | O(n + m) |
| `g[u]` | `Indexed` view of `u`'s arcs, in input order | O(1) |
| `g.has(u, v)` | is there an arc | O(deg u) |
| `g.transpose` | arcs reversed, same `E` | O(n + m) |
| `g.edges` | the `[E]` records it was built from | O(1) |

Validated input types (`Tree<V>`, `SimpleGraph<V, E>`, `ConnectedGraph`,
`Dag`, `Digraph`) read like the plain edge list and also tell `cpl stress`
what to generate and shrink.

### `next`: graphs without building one

```python
bfs(g, from: [s])                                   # a stored graph
bfs(from: [s], next: c => c.adj4.filter(c => grid[c] != '#'))   # a grid: domain Cell = (Row, Col)
bfs(from: [start], next: st => moves(st))           # a state space; State is a domain or hashable
dijkstra(g, from: [(s, 0)], w: a => edges[a.e].w)   # weights through the arc
```

- `next(u)` returns any `Seq` of vertices or of arcs (anything with `.to`).
  An algorithm that needs edge ids (bridges, Euler walks) needs arcs.
- **Storage follows the vertex type.** A domain gives `[V]` arrays; any other
  hashable type gives hash maps. `cpl explain` shows which.
- A `next` function hides the vertex type, so a call with no `from:` names
  it: `scc::<V>(next: u => out[u])`.

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
| `bfs(next, from:, until:)` | unit | O(n + m) |
| `bfs01(next, from:, w:)` | 0 or 1 | O(n + m) |
| `dial(next, from:, w:, max_w:)` | small ints | O(n · max_w + m) |
| `dijkstra(next, from: [(s, d0)], w:, until:)` | non-negative | O(m log n) |
| `bellman_ford(next, from:, w:)` | any; `none` on a negative cycle | O(n m) |
| `floyd_warshall(w: [V, V]?W)` | dense | O(n³) |

`until: t` stops once `t` is settled. Multi-source is a longer `from:`.

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
| `components(next)` | `(domain C, comp: [V]C)` | O(n + m) |
| `scc(next)` | `(domain C, comp: [V]C)`, `C` in topological order | O(n + m) |
| `condense(next, comp)` | `Graph<C, _>` of the components | O(n + m) |
| `two_edge_components(next)` | `(domain C, comp: [V]C)` | O(n + m) |
| `blocks(next)` | `(domain B, block: [E]B)`, biconnected blocks of edges | O(n + m) |
| `bridges(next)`, `cut_vertices(next)` | `[E]Bool`, `[V]Bool` | O(n + m) |
| `topo(next)` | `?List<V>`, `none` on a cycle | O(n + m) |
| `two_color(next)` | `?[V]Bool`, `none` when an odd cycle exists | O(n + m) |
| `find_cycle(next)` | `?List<V>` | O(n + m) |
| `kruskal(edges, w:)` | `(total, take: [E]Bool)` | O(m log m) |
| `euler_walk(edges, directed:)` | `?Walk<V, E>` | O(n + m) |
| `TwoSat<X>()`, `.add_or(a, b)`, `.solve()` | `?[X]Bool` | O(n + m) |
| `Functional(next: [V]V)` | `.after(v, k)`, `.on_cycle(v)`, `.cycle_len(v)`, `.root_of(v)` | O(n log n) build |

Flows and matchings are their own module (`std.flow`), on the same
`V`/`E` domains.

## Trees

```python
let t = Rooted(g, root: V.first!)          # or Rooted.from_parents(par), Rooted.forest(g, from:)
```

| Form | Meaning | Cost |
|---|---|---|
| `t.parent(v)` | `?V` | O(1) |
| `t.children(v)` | `Indexed` view of `v`'s children (stored contiguously) | O(1) |
| `t.depth(v)`, `t.size(v)` | edges from the root; vertices in the subtree | O(1) |
| `t.order` | every vertex, parents before children (BFS order); `.rev` is children first | O(1) |
| `t.lca(a, b)`, `t.dist(a, b)`, `t.is_anc(a, b)` | | O(log n) |
| `t.ancestor(v, k)`, `t.jump(a, b, k)` | `?V`: k steps up; k steps along the path a → b | O(log n) |
| `t.path(a, b)` | `List<V>` | O(length) |
| `t.diameter()`, `t.centroid` | `(len, u, v)`; a centroid | O(n) |

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
var seg = SegTree(max, t.reorder(val))     # [t.Pre]T
seg.fold(t.sub(v))                         # subtree query: one range
t.fold_path(a, b, seg)                     # path query: O(log n) ranges (HLD)
t.apply_path(a, b, lazy, add(x))
for r in t.chains(a, b): ...               # the ranges themselves, when you need them
```

| Form | Meaning | Cost |
|---|---|---|
| `t.sub(v)`, `t.below(v)` | `Range<t.Pre>`: subtree with / without `v` | O(1) |
| `t.chains(a, b)` | `yields Range<t.Pre>`, ordered from `a` to `b` | O(log n) ranges |
| `t.fold_path(a, b, ds, edges:)`, `t.apply_path(...)` | path fold / update over any range structure | O(log n) ranges |
| `t.tour` | `yields Visit<V>`: `Enter` / `Exit` events | O(n) |
| `t.virtual(keys)` | the virtual (auxiliary) tree of `keys` | O(k log k) |
| `Centroids(g)` | centroid decomposition: `.parent(c)`, `.ancestors(v)` with distances | O(n log n) |

### Changing trees

`LinkCut<V, T>(op)`, `LazyLinkCut`, `EulerTourTree`, `JumpTree` (online
binary lifting as leaves are added). Same vertex domain, same method names
as `Rooted` where they mean the same thing (`lca`, `dist`, `jump`,
`fold_path`).

## Open

1. **`g[u]` as the arcs view** replaces `g.out(u)`, `g.get(u)`, `g.deg(u)`
   and `g.lists()`. One spelling, O(1), indexed (so `g[u][0]` and
   `g[u].len` work).
2. **`from:` always takes a list**, `bfs(g, from: [s])`, so single and
   multi-source are one form. Alternative: a separate `bfs(g, s)`.
3. **Hash-backed state spaces.** Storage follows the vertex type: a
   domain gives arrays, anything hashable gives maps. Alternative: require
   a domain and make the user `compress` first.
4. **Names that drift today:** `articulation_points` → `cut_vertices`,
   `biconnected_components` / `edge_blocks` → `blocks`, `Adj` and
   `*_of_pairs` builders → `Graph`, `subtree_sizes` → `t.size(v)`,
   `preorder` → `t.order` (BFS) plus `t.tour`.
