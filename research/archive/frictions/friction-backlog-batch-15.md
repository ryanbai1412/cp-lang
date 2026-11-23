# Friction backlog, batch 15: flow, matching, Eulerian tours, and graph connectivity

A Stage-1 scout pass over ten hard unported graph submissions covering network
flow, min-cut, min-cost flow / weighted bipartite matching, lexicographically
smallest matching, bipartite 2-coloring, cyclic Eulerian tours, vertex-
biconnected components / bridges / articulation points, SCC condensation +
virtual trees, and rollback bipartite connectivity. The ports and two proposed
library modules live in [`proto/ports/scout_batch_15/`](../proto/ports/scout_batch_15/):
`noi09p4`, `dmopc20c7p5`, `sdoi17r1p4`, `noi09p1`, `coci25c3p3`, `cco16p3`,
`cco21p3`, `noi21p3`, `noi16p2`, and `btoi20p3`. Items are numbered `N1`–`N11`
so they do not collide with `F` (batch 1), `B` (2 and 4), `G` (3), `K` (5),
`S` (6), `D` (7), `H` (8), or `T` (10).

Verification: all ten ports run on the reference interpreter
(`proto/cp.py lib-run`) and agree with their compiled C++ originals across 40
random inputs each (400 runs total via
`python3 proto/ports/scout_batch_15/check.py`), and all five unit tests in the
proposed `scout.*` modules pass (`python3 proto/cp.py test proto/ports/scout_batch_15`).
Every "today" snippet below comes directly from the runnable ports, and every
error quoted was probed directly against `proto/cp.py lib-run`.

Across the ten ports, non-blank, non-comment LOC drops from **2,029** raw C++
lines to **700** CP Lang lines (**65.5%** reduction), and from **1,261**
library-stripped C++ lines (excluding templates, preambles, `main()`, and
standard data structures) to **700** (**44.5%** reduction).

## Suggested synthesis order for Stage 2

1. **N1 (disjoint-union / sum domains)** — the single biggest design gap of
   this batch: every flow, min-cut, and min-cost flow problem currently
   abandons domain safety and falls back to manual integer offsets.
2. **N2, N3, N4, N5 (stdlib connectivity, flow/matching, virtual-tree, and
   rollback/potential-DSU gaps)** — all four are pure userland modules already
   prototyped and tested in `proto/ports/scout_batch_15/{connectivity,flow_ext}.cp`,
   directly simplifying 6 of the 10 ports.
3. **N6, N7, N8 (sequence formatting, map-entry mutation, plain adjacency
   adaptation)** — small general ergonomics fixes hitting 1–3 ports each.
4. **N9, N10, N11 (2D/struct array broadcast bug, `sort_by`/`dedup` + generic
   record constructor parsing, prelude `any`/`all` + `rotate` + `lower_bound`)**
   — crisp edge-case fixes and standard sequence helpers.

---

## N1. Disjoint-union (sum) domains for layered and multi-role graphs (3 of 10: `noi09p4`, `dmopc20c7p5`, `sdoi17r1p4` via `MinCostFlow`)

Every network-flow, min-cut, matching-via-flow, and multi-state graph problem
combines two to four distinct vertex roles (`Src`, `Item`, `Pair`, `Snk`, or
`Src`, `Boy`, `Girl`, `Snk`). Because `FlowGraph<V>` and `MinCostFlow<V>` take
a single domain `V`, both `noi09p4` and `dmopc20c7p5` (as well as the
`MinCostFlow` test in `flow_ext.cp`) have to abandon typed domains and go back
to raw integer offset arithmetic — the exact bug class domains exist to prevent.

Today in `dmopc20c7p5.cp`:

```cp
domain Node = 0..n + m + 2
let (src, snk) = (Node(n + m), Node(n + m + 1))
for i in Item:
    net.add(Node(i.to_idx), snk, cost[i])
for e in Pair:
    let pnode = Node(n + e.to_idx)
    net.add(src, pnode, w)
    net.add(pnode, Node(u - 1), INF)
```

And today in `noi09p4.cp`:

```cp
domain Node = 0..total_cells + 2
let (src, snk) = (Node(total_cells), Node(total_cells + 1))
...
net.add(src, Node(u.to_idx), score[u])
```

**Proposed general abstraction**: tagged disjoint-union domains
(`domain Node = Src | Item | Pair | Snk`, where unit variants have cardinality
1 and domain variants carry their typed index, flattening at compile time to
`0..(sum of cardinalities)` with zero runtime tag overhead). Wanted:

```cp
domain Node = Src | Item | Pair | Snk
var net = FlowGraph<Node>.new()
for i in Item: net.add(Node.Item(i), Node.Snk, cost[i])
for e in Pair:
    net.add(Node.Src, Node.Pair(e), w)
    net.add(Node.Pair(e), Node.Item(u), INF)
```

This single construct cleanly solves flow networks, bipartite matching
reductions, layered Dijkstra (`(V, K)` / sum of layers), and composite-state
graphs without any flow-specific compiler magic.

---

## N2. Undirected connectivity in `std.graph`: 2-ECC / bridges, vertex-biconnected components, articulation points, and 2-coloring (3 of 10: `cco16p3`, `noi16p2`, `coci25c3p3`)

`std.graph` has `scc`, `components`, and `condense`, so directed connectivity
is one line, but undirected biconnectivity and bipartiteness have no library
counterparts. Three originals (`cco16p3`, `noi16p2`, `coci25c3p3`) each
hand-wrote 30–100 lines of Tarjan low-link or coloring DFS, and `cco16p3`
subtask 5 had to maintain separate `bridges`, `artic`, `tin`, `low`, `par`,
and `dfs3` routines just to enumerate vertex-biconnected blocks.

Wanted (and running today via [`connectivity.cp`](../proto/ports/scout_batch_15/connectivity.cp)):

```cp
// cco16p3.cp — vertex-biconnected blocks in 1 line
for block in biconnected_components::<V>(u => g.adj(u)):
    ...

// noi16p2.cp — articulation points on sparse neighborhood
let cut = articulation_points::<Cell>(u => [(to: v) for v in adj[u]])

// coci25c3p3.cp — bipartite 2-coloring
let side = two_color::<V>(u => g.adj(u))!
```

**Proposed general abstraction**: move `two_edge_components`,
`biconnected_components`, `articulation_points`, and `two_color` into
`std.graph` (alongside `scc` and `components`). Because `two_edge_components`
returns `(domain C, comp: [V]C)` and identifies parent edges by `a.e`
(handling multiedges properly), it composes directly with existing
`std.graph.condense` to produce the bridge tree with zero duplicate bridge
edges.

---

## N3. Min-cost max-flow and weighted bipartite matching in `std.flow` (1 of 10: `sdoi17r1p4`; generalizes to all assignment / MCMF problems)

`stdlib/README.md` notes that min-cost flow and matching are not yet in
`stdlib/`. `sdoi17r1p4`'s original spent 90 lines on a dense `__int128` matrix
flow routine (running Dijkstra over negative reversed edges without potentials).

Wanted (and running today via [`flow_ext.cp`](../proto/ports/scout_batch_15/flow_ext.cp)):

```cp
use std.flow.{MinCostFlow, max_weight_matching}

// sdoi17r1p4.cp — O(n^3) Hungarian inside fractional binary search
let best = max_weight_matching::<Boy, Girl>((i, j) => SCALE * a[i][j] - mid * b[i][j]).total

// General network with signed costs (SPFA + Johnson potentials + Dijkstra)
var net = MinCostFlow<Node>.new()
net.add(u, v, cap, cost)
let (flow, total_cost) = net.min_cost_flow(src, snk, limit)
```

**Proposed general abstraction**: promote `MinCostFlow<V>` and
`max_weight_matching<L, R>` from `flow_ext.cp` into `std.flow` (or a sibling
`std.matching` for unweighted Hopcroft-Karp + weighted Hungarian). Shipping
Hungarian alongside `MinCostFlow` is worth the ~45 lines because dense `n x n`
matching inside binary search needs O(n^3) with small constants rather than
general-graph SSP.

---

## N4. Compressed (virtual / auxiliary) trees on `std.tree.Rooted<V>` (1 of 10: `noi21p3`; generalizes to all per-query subset-tree problems)

`std.tree.Rooted<V>` already provides `tin`, `is_anc`, `lca`, and `far`, which
are the exact building blocks of a virtual tree — yet callers must rewrite the
sort-by-`tin`, adjacent-LCA, dedup, and ancestor-stack loop on every virtual-tree
problem. In `noi21p3`, once `scc` and `Rooted.weighted` replace 180 lines of
C++ boilerplate, building the virtual tree per query is the only remaining
reusable tree algorithm.

Wanted (and running today via [`connectivity.cp`](../proto/ports/scout_batch_15/connectivity.cp)):

```cp
// noi21p3.cp — virtual tree over query endpoints in 1 line
let vt = tree.virtual_tree(keys)
for e in vt.edges:
    if vis_f[e.u] and vis_r[e.v]:
        ans += tree.far[e.v] - tree.far[e.u] - wt[e.v]
```

**Proposed general abstraction**: add `pub fn virtual_tree(self, keys: List<V>) -> (nodes: List<V>, edges: List<(u: V, v: V)>)`
as a method on `std.tree.Rooted<V>`. It is ~20 lines of pure userland code
using only `self.tin`, `self.lca`, and `self.is_anc`.

---

## N5. Rollback DSU and group-potential DSU in `std.dsu` (1 of 10: `btoi20p3`)

`examples/local-lib/ds/rollback_dsu.cp` lives outside `stdlib/` and only tracks
connectivity, not edge parity. Solving `btoi20p3` (offline odd-cycle queries
under range deletion) with plain `RollbackDsu` would force doubling the vertex
domain to `0..2 * n` and losing `V` type safety.

Today we wrote `BipartiteDsu<V>` in [`connectivity.cp`](../proto/ports/scout_batch_15/connectivity.cp):

```cp
let before_right = dsu.depth
while split >= ql and dsu.is_bipartite:
    add(split)
    split -= 1
max_r[E(mid)] = split
dsu.rewind(before_right)
```

**Proposed general abstraction**: promote `RollbackDsu<V>` to `std.dsu` and
add a group-parameterized `PotentialDsu<V, T>(g: Group<T>)` (and its rollback
variant). Because `std.algebra` already has `Group<T>` (`Xor<Bool>`, `Sum<Int>`),
one userland structure unifies plain union-find, bipartite/parity DSU (with
`Xor<Bool>`), and difference-constraint DSU (`x[v] - x[u] = w` with `Sum<Int>`)
without a separate one-off `BipartiteDsu` type.

---

## N6. Space-separated sequence output and fixed-precision `Float` formatting (3 of 10: `noi09p1`, `coci25c3p3`, `sdoi17r1p4`)

`print(list)` formats a `List` with brackets and commas (`[5, 3, 1]`), and
`List` has no `.join(sep)` (*checked*: `["a", "b"].join(" ")` fails with
`has no method 'join'`). Two ports therefore repeat a 5-line string-building
loop to print one space-separated answer line, and `sdoi17r1p4` writes 10 lines
of integer digit extraction because `print` has no fixed-decimal formatting
for `Float` (*checked*: `f"{1.5:.6f}"` fails with `unknown name 'f'`, and
`round` fails with `unknown name 'round'`).

Today in `noi09p1.cp` and `coci25c3p3.cp`:

```cp
var line = ""
for i in Pos:
    if i.to_idx > 0: line = line + " "
    line = line + "{ans[i]!.to_idx}"
print(line)
```

And today in `sdoi17r1p4.cp`:

```cp
let scaled6 = (lo + 5) / 10
let whole = scaled6 / 1000000
var frac = scaled6 % 1000000
var digits = List.filled(6, 0)
for k in (0..6).rev:
    digits[k] = frac % 10
    frac /= 10
var out = Str(whole) + "."
for d in digits: out = out + Str(d)
print(out)
```

**Proposed general abstraction**: add `join(sep)` on sequences/iterators (or
space-separated `print_line(xs)`) and standard format specifiers inside CP
Lang's existing string interpolation (`"{Float(lo) / 1e7:.6f}"`). Wanted:

```cp
print([ans[i]!.to_idx for i in Pos].join(" "))
print("{Float(lo) / 1e7:.6f}")
```

---

## N7. `Map` entry / default-container insertion (`m[k].push(x)` and `get_or_put`) (1 of 10: `noi16p2`)

`stdlib/tour.cp` already auto-initializes missing map keys on fold assignments
(`m[6] += 2`, `m[3] max= -5`), so writing `by_row[r].push(c)` looks equally
valid — yet *checked* on `proto/cp.py lib-run`, `var m: Map<Int, List<Int>> = Map(); m[1].push(2)`
fails with `key 1 is not in the map`. `noi16p2` (grouping blocked cells by row)
must write manual get-check-insert boilerplate:

```cp
var cols = by_row.get(r) ?? []
cols.push(c)
by_row[r] = cols
```

**Proposed general abstraction**: `map.get_or_put(k, => init)` returning a
mutable slot, plus auto-defaulting missing entries to empty for
`Map<K, List<V>>` / `Map<K, Set<V>>` (matching the existing fold-identity rule
in `stdlib/tour.cp`).

---

## N8. Adjacency adapter friction between `[V]List<V>` and `std.graph`/`Rooted`'s `(to: V, ...)` protocol (3 of 10: `noi16p2`, `noi21p3`, `noi09p4`)

`std.graph` algorithms (`bfs`, `scc`, `components`, `condense`, `topo`) and
`std.tree.Rooted` expect `adj(u)` to yield records with a `.to` field,
matching `Graph<V, E>.adj(u)`. That works well when the edge domain `E` is
known upfront, but when a graph or spanning tree is built dynamically (the
row-segment sweep and sparse radius-2 grid in `noi16p2`, or the peeled DAG
spanning tree in `noi21p3`), you hold `[V]List<V>` and must wrap every call in
a temporary list comprehension just to attach `.to`:

```cp
// noi16p2.cp (twice) and noi21p3.cp
let (C, _) = components::<Seg>(u => [(to: Seg(v)) for v in seg_adj[u.to_idx]])
let cut = articulation_points::<Cell>(u => [(to: v) for v in adj[u]])
let tree = Rooted<C>.weighted(u => [(to: v) for v in tree_adj[u]], root, a => wt[a.to])
```

**Proposed general abstraction**: (1) allow `adj: V -> Iter<V>` directly on
unweighted `std.graph` and `Rooted` algorithms via a small `ArcTarget<V>`
trait implemented by both `V` and any record with `to: V`, so
`components::<Cell>(u => adj[u])` works with zero wrapper allocations; and
(2) add a `GraphBuilder<V>` in `std.graph` that collects edges dynamically and
freezes them into `(domain E, Graph<V, E>)` when the edge count is only known
after a sweep.

---

## N9. Multi-dimensional and struct-valued array broadcast `var a: [D1][D2]T = init` fails silently at init (1 of 10: `cco16p3`; affects all 2D tables and `Mod<P>` DPs)

Scalar broadcast `var deg: [V]Int = 0` works for 1D primitive arrays, so
writing a 2D adjacency matrix `var has_edge: [V][V]Bool = false` in `cco16p3`
(or a struct-valued array `var dp: [C]Mod<P> = Mod(0)`) looks completely
idiomatic. *Checked* on `proto/cp.py lib-run`: `var has_edge: [V][V]Bool = false`
fills the outer `[V]` with `false` instead of `[V]Bool` and crashes on first
write with `cannot index false`, forcing nested lambdas:

```cp
var has_edge = [V] _ => [V] _ => false
```

**Proposed general abstraction**: unify `[D1]...[Dk]T = expr` broadcast
semantics so any value of element type `T` (including nested arrays, tuples,
and user structs like `Mod<P>`) recursively fills every dimension consistently.

---

## N10. `List.sort_by(key)` / `dedup()` and `List<(u: V, v: V)>()` constructor parsing (2 of 10: `noi21p3`, `cco16p3`)

Writing `virtual_tree` for `noi21p3` and pair-deduplication for `cco16p3` hit
three sharp edges on `proto/cp.py lib-run`:

1. `keys.sort_by(v => tree.tin[v].ord)` fails with `has no method 'sort_by'`,
   forcing Schwartzian tuple packing (`var by_tin = [(tree.tin[v].ord, v) for v in keys]; by_tin.sort()`).
2. Deduplicating a sorted list has no `.dedup()` method, forcing two separate
   4-line manual loops in `virtual_tree` (and an O(k^2) dedup loop in `cco16p3`).
3. `var edges = List<(u: V, v: V)>()` fails to parse as a constructor call and
   crashes with `cannot compare List with (u: T, v: T)` because `<` before `(`
   is parsed as less-than rather than a generic type argument, forcing
   `var edges: List<(u: V, v: V)> = []`.

**Proposed general abstraction**: add `sort_by(key)` and `dedup()` to `List<T>`
(in `std.seq` / prelude), and allow record/tuple types inside `Type<...>`
expressions (or consistently promote `var x: List<T> = []` as the single
documented empty-container form and reject `List<...>()` cleanly in the
parser).

---

## N11. Missing small prelude/sequence helpers: `any`/`all`, `List.rotate`, `lower_bound` on sorted sequences, and small-slice `permutations` (4 of 10: `noi09p1`, `cco21p3`, `cco16p3`, `noi16p2`)

Four standard one-liners required manual loops because the prelude and
`std.seq` do not yet provide them (*all checked* on `proto/cp.py lib-run`):

- `any(ans[i] == none for i in Pos)` fails with `unknown name 'any'`, forcing
  two explicit boolean-flag loops in `noi09p1.cp`.
- `nbrs.rotate(1)` fails with `has no method 'rotate'`, forcing manual slice
  concatenation twice in `cco21p3.cp` (`nbrs[1..nbrs.len] + [nbrs[0]]` and
  `order[pi + 1..d] + order[0..pi]`).
- `cco21p3.cp` writes a 6-line manual binary search on `cum_steps` because
  `lower_bound` exists on `Fenwick` (`std.fenwick`) but not on sorted
  `List<T>` / `[D]T`.
- `cco16p3.cp` writes three nested loops (`i1`, `i2`, `i3`, `i4 = 10 - i1 - i2 - i3`)
  to test the 24 cyclic orderings of a 5-vertex block because `List` has no
  `next_permutation` / `permutations` iterator.

**Proposed general abstraction**: add `any` and `all` to the prelude beside
`sum`, `min`, and `max`; add `rotate_left` and `next_permutation` on `List`;
and add `partition_point` / `lower_bound` over `Seq<T>` (building on batch
10's `T3`) so binary search on prefix-sum arrays is one call instead of a
hand-rolled loop.
