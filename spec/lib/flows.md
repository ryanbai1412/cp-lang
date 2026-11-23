# `std.flow`: flows, cuts and matchings

> **0.3.0 status.** This chapter is the family's design survey and API.
> Where it disagrees with the core chapters or the aligned rounds, they
> win: `core/03-numbers-layout.md` (four number types, no `I32`/`U32`,
> `Mod<P>` not `Mint`), `core/05-ownership.md` (constructors own their
> results), `core/07-sequences.md` (capabilities, views keep keys),
> `lib/ops-ranges.md` (`Op`/`Monoid`/laws, `each(...)`, no
> `RangeFenwick`, `MinHeap`/`MaxHeap` as `Heap(by:)`) and
> `lib/sequence-patterns.md`. The stubs in `std/` are the signatures of
> record. Reconciliation of this text is tracked in `delta/`.


The top-down design of the flows-and-matching area: the contest patterns it
must cover, one API that covers them, the laws each function keeps, how the
area uses domains, and what stays user code. `plan/STDLIB.md` §4.12 is the
summary; this file is the reasoning.

## 1. What contests ask

Flow problems are modelling problems. The algorithm is a black box the
contestant trusts; the work is the network, and the bugs are in the network
(node numbering, a missed reverse arc, a cut read from the wrong side, an
overflowing "infinity"). Surveyed sources:

| Source | What it ships |
|---|---|
| AtCoder Library | `mf_graph` (Dinic: `add_edge`, `flow(s, t, limit)`, `min_cut(s)`, `get_edge`, `change_edge`), `mcf_graph` (successive shortest paths with potentials: `flow`, `slope`). No matching type: matching is "build a flow". |
| KACTL | `Dinic`, `PushRelabel`, `EdmondsKarp`, `MinCostMaxFlow`, `GlobalMinCut` (Stoer–Wagner), `GomoryHu`, `hopcroftKarp`, `DFSMatching`, `MinimumVertexCover`, `WeightedMatching` (Hungarian, rectangular `n <= m`), `GeneralMatching` (randomized Tutte matrix), `MatroidIntersection`. |
| ecnerwala (`proto/ports/ecnerwala/*_orig.cpp`) | `cf2147h`: Dinic plus Gusfield's Gomory–Hu loop written inline; `cf2062g`: an O(n^3) Hungarian minimizing Manhattan cost. No other flow code in the 200+ originals. |
| Corpus ports | `scout_batch_15`: `dmopc20c7p5` and `noi09p4` (project selection by min cut), `sdoi17r1p4` (Hungarian inside a fractional binary search), `flow_ext.cp` (min-cost flow and Hungarian prototypes); `scout_batch_5/noi06p4` (project selection, hand-offset node numbering); `examples/local-lib/graph/` (Hopcroft–Karp with König cover, min-cost flow, lower-bound flows). |
| Codeforces / AtCoder / ICPC usage | Max flow with a modelled network is the dominant shape (assignment with capacities, vertex splitting, grid colourings, project selection); bipartite matching with König's theorem second (vertex cover, independent set, DAG path cover); assignment (Hungarian) and min-cost flow (transport, k disjoint cheapest paths, convex costs by unit arcs) third; lower bounds, Gomory–Hu, global min cut and general matching are rare but recurring at Div. 1 / WF level. |

The patterns, in order of frequency, and what each needs from the library:

1. **Max flow on a modelled network.** Arc handles to read flows back,
   unbounded arcs that never overflow, and node spaces built from several
   domains plus a source and a sink.
2. **Min cut recovery.** The source side, and which arcs cross it.
3. **Project selection / maximum-weight closure.** Profits, "taking u
   forces v", answer and chosen set. Three corpus ports hand-build it.
4. **Bipartite matching.** Size, both directions of the matching, König
   cover; independent set and DAG path cover follow in one line.
5. **Assignment.** Minimum and maximum weight perfect matching on a
   possibly rectangular cost table, both directions named.
6. **Min-cost flow.** Flow and cost, negative arc costs, a flow limit, and
   the convex cost curve (`slope`) for "best cost for each k".
7. **Lower bounds.** Feasible circulation, max and min flow with
   `lo <= f <= hi` per arc.
8. **All-pairs min cut.** Gomory–Hu tree.
9. **General matching.** Maximum cardinality matching in a non-bipartite
   graph.

Deliberately not here yet (section 7): matroid intersection, push–relabel, dynamic
capacity changes.

## 2. The API

One module, `std.flow`. Two network types that share `Arc`, and free
functions for everything whose input is a table or an edge list.

<!-- spec-only: signatures and costs, not a program -->
```cp
struct Arc(id: 0..2_147_483_648)
    @get fn rev(self) -> Arc  // O(1)

struct FlowGraph<V>                                     // Dinic
    fn new() -> FlowGraph<V>                            // O(V)
    fn add(mut self, u: V, v: V, cap: ?Int) -> Arc      // O(1); none: never cut
    fn add_both(mut self, u: V, v: V, cap: Int) -> Arc  // O(1); undirected
    fn flow(self, a: Arc) -> Int                        // O(1)
    fn cap(self, a: Arc) -> Int                         // O(1)
    fn source(self, a: Arc) -> V                        // O(1)
    fn target(self, a: Arc) -> V                        // O(1)
    fn max_flow(mut self, s: V, t: V, limit: ?Int = None) -> Int
    // O(V^2 E); O(E sqrt V) on unit networks
    fn push_relabel(mut self, s: V, t: V) -> Int
    // the same flow, O(V^2 sqrt E); highest label, gap
    fn min_cut(self, s: V) -> [V]Bool  // O(V + E)
    fn reset(mut self)                 // O(E)

struct CostFlow<V>                                             // successive shortest paths
    fn new() -> CostFlow<V>                                    // O(V)
    fn add(mut self, u: V, v: V, cap: ?Int, cost: Int) -> Arc  // O(1)
    fn flow(self, a: Arc) -> Int                               // O(1)
    fn cap(self, a: Arc) -> Int
    fn cost(self, a: Arc) -> Int
    fn source(self, a: Arc) -> V
    fn target(self, a: Arc) -> V
    fn min_cost_flow(mut self, s: V, t: V, limit: ?Int = None)
        -> (flow: Int, cost: Int)
        // O(F E log V) with potentials, plus O(V E) once if a cost is negative
    fn slope(mut self, s: V, t: V, limit: ?Int = None)
        -> List<(flow: Int, cost: Int)>  // same cost

fn max_closure<V, G: Next<V>>(before: G, profit: V -> Int)
    -> (value: Int, take: [V]Bool)  // one max flow on V + 2

fn max_matching<L, R>(adj: [L]List<R>)
    -> (size: Int, mate: [L]?R, back: [R]?L)  // O(E sqrt V) Hopcroft–Karp
fn min_vertex_cover<L, R>(adj: [L]List<R>)
    -> (left: [L]Bool, right: [R]Bool)  // O(E sqrt V), König
fn max_weight_matching<L, R>(weight: (L, R) -> Int)
    -> (total: Int, mate: [L]R)  // O(L^2 R) Hungarian, L <= R
fn min_weight_matching<L, R>(weight: (L, R) -> Int)
    -> (total: Int, mate: [L]R)  // O(L^2 R) Hungarian, L <= R
fn general_matching<V>(adj: [V]List<V>)
    -> (size: Int, mate: [V]?V)  // O(V^3) Edmonds' blossoms
fn max_weight_general_matching<V, E>(edges: [E](u: V, v: V, w: Int))
    -> (total: Int, mate: [V]?V)  // O(V^3) weighted blossoms, O(V^2) memory

fn circulation<V, E>(arcs: [E](u: V, v: V, lo: Int, hi: Int)) -> ?[E]Int
fn max_bounded_flow<V, E>(arcs: [E](u: V, v: V, lo: Int, hi: Int), s: V, t: V)
    -> ?(value: Int, flow: [E]Int)
fn min_bounded_flow<V, E>(arcs: [E](u: V, v: V, lo: Int, hi: Int), s: V, t: V)
    -> ?(value: Int, flow: [E]Int)
    // each: two max flows on V + 2, Dinic's bound

fn gomory_hu<V, E>(edges: [E](u: V, v: V, w: Int))
    -> List<(u: V, v: V, w: Int)>  // V - 1 max flows (Gusfield)
fn global_min_cut<V, E>(edges: [E](u: V, v: V, w: Int))
    -> (w: Int, side: [V]Bool)  // Stoer-Wagner, O(V^3), O(V^2) memory
```

### Names

- **Objective first, then the object:** `max_flow`, `min_cut`,
  `min_cost_flow`, `max_closure`, `max_matching`, `max_weight_matching` /
  `min_weight_matching`, `max_bounded_flow` / `min_bounded_flow`,
  `min_vertex_cover`. A contestant who knows one name guesses the rest.
  The objective is never encoded by negating weights: `cf2062g` minimized
  by `max_weight_matching(-cost)` and read `-total` back; the friction
  report names this.
- **Named algorithms where the algorithm is the contract:** `gomory_hu`
  (the tree it returns is Gomory–Hu's, not any cut tree), `general_matching`
  (the problem; Edmonds is documented as the algorithm), `circulation`.
- **Arc accessors are one vocabulary across both networks:** `flow`, `cap`,
  `source`, `target`, `rev`; `CostFlow` adds `cost`. `CostFlow.flow(a)` is
  the arc's flow, so the solver is `min_cost_flow`, not `flow(s, t)` (the
  earlier §4.12 sketch); one name, one meaning.
- **Options are named arguments:** `limit:` on every solver, `cap: None`
  for an unbounded arc. No `_with_limit`, `_inf` or `_by` variants.

### Why two network types and free functions

`FlowGraph` and `CostFlow` are separate because their arcs carry different
data and their solvers different contracts; a cost of zero is not a
max-flow network (SSP is O(F) augmentations, Dinic is not). They share
`Arc` and the accessor names so moving a model from one to the other is a
type change and an extra argument per `add`.

Matching, assignment, closure, bounded flows and Gomory–Hu are functions
because their inputs are values the contestant already has (an adjacency
array, a weight function, an edge table read from input), and their result
is a value. None of them is incremental in contest use. Each builds a
`FlowGraph` internally where it is a flow reduction; the contestant who
needs the network itself (to add an arc, to read a residual) writes the
reduction with `FlowGraph`, which is ten lines, not a rewrite.

## 3. Laws

Every test in `stdlib/flow.cp` checks one of these, most against a brute
force on random small inputs.

- **Capacity and conservation.** After `max_flow(s, t)`, for every arc,
  `0 <= g.flow(a) <= g.cap(a)` and `g.flow(a.rev) == -g.flow(a)`; the net
  flow into every vertex but `s` and `t` is zero.
- **Accumulation.** `max_flow` adds to the flow already there and returns
  the amount added; a second call returns 0. `limit:` stops at that amount;
  `max_flow(s, t, limit: k) == min(k, max_flow(s, t))`.
- **Max-flow min-cut.** After `max_flow(s, t)` without a limit,
  `let side = g.min_cut(s)` has `side[s]`, `not side[t]`, and the
  capacities of arcs from `side` to its complement sum to the flow.
  `min_cut(s)` traps unless such a `max_flow` from `s` came since the last
  `add` or `reset`, so a stale or partial cut cannot be read.
- **No `min_cut(s, t)`.** A combined call returning `(value, side)` was
  considered and rejected: no corpus port reads the side of an s-t cut
  (cf1404e, dmopc20c7p5 and noi06p4 print the value; the closure ports use
  `max_closure`), and the cut value under a second name would be two ways
  to ask `max_flow` one question. `max_flow` then `min_cut(s)` stays the
  one spelling; revisit when two ports read the side.
- **Unbounded arcs.** `cap: None` is 2^60: no finite cut chooses it, and a
  sum of 2^3 of them cannot overflow 63 bits. Finite capacities and their
  total stay below 2^60; the library asserts each `cap >= 0`.
- **Closure.** `max_closure(before, profit)` returns `take` closed under
  `before` (an arc `u -> v` means `take[v]` implies `take[u]`, the direction
  `peel` reads) and maximizing `sum(profit(v) for v if take[v])`, which equals `value`; the empty set is always feasible, so
  `value >= 0`.
- **König.** `max_matching(adj).size` equals the number of `true`s in
  `min_vertex_cover(adj)`, and every edge has a covered endpoint. `mate`
  and `back` are inverse partial maps; every matched pair is an edge.
- **Assignment.** `mate` is injective and total on `L`; `total` is the sum
  of `weight(l, mate[l])`, optimal among all injections `L -> R`; `L.K <=
  R.K` is asserted. `min_weight_matching(w).total ==
  -max_weight_matching((l, r) => -w(l, r)).total`.
- **Min-cost flow.** `min_cost_flow` returns the largest flow up to
  `limit` and, among flows of that value, the cheapest, provided the
  network has no negative-cost cycle (asserted only in debug builds by the
  initial Bellman–Ford). `slope` lists the breakpoints of the convex
  piecewise-linear cost-of-flow function, from `(0, 0)` to the final
  `(flow, cost)`; slopes are nondecreasing.
- **Bounds.** A returned `flow` has `lo <= flow[e] <= hi` on every arc and
  conserves at every vertex (for `circulation`) or every vertex but `s`
  and `t` (bounded flows), and `value` is the net flow out of `s`. `None`
  means no feasible flow exists.
- **Gomory–Hu.** The result is a tree on `V` (`V.K - 1` edges, connected;
  components of the input are joined by `w = 0` edges), and for every
  `s != t` the minimum `s-t` cut of the input equals the minimum `w` on the
  tree path from `s` to `t`.
- **General matching.** `mate` is an involution on matched vertices, every
  matched pair is an edge, and `size` is maximum (no augmenting path).

## 4. Domains and index spaces

- **Vertices are a domain parameter.** `FlowGraph<V>`, `CostFlow<V>` and
  every function are generic over the vertex domain, so a capacity array is
  `[V]Int`, a cut is `[V]Bool`, and passing an `Item` where a `Node` is
  expected does not type-check.
- **Bipartite sides are two domains.** `max_matching<L, R>` and the
  Hungarian take two domains and return `[L]?R` and `[R]?L`; there is no
  `n + j` numbering of the right side, the classic bug of copying a matching
  out of a flow network.
- **Edges are a domain when the input gives them.** Bounded flows and
  Gomory–Hu take `[E](u: V, v: V, ...)`, the shape `read` produces and
  `Graph<V, E>` already uses, and bounded flows return `[E]Int`: the flow on
  arc `e` is `flow[e]`, never `flow[i]` with `i` an index into a list built
  in a different order.
- **Arcs are handles, not a domain.** A network grows while it is built, so
  `Arc` is a typed handle like `Dsu` roots: returned by `add`, valid
  forever, meaningless across networks.
- **Node spaces made of several domains.** The modelled network is usually
  `Item + Pair + {S, T}` or `Left + Right + {S, T}`. Today a port flattens
  it into `domain Node = 0..n + m + 2` and writes `Node(n + e.to_idx)`
  (`noi06p4` calls this friction K11; `dmopc20c7p5`, `noi09p4` repeat it).
  The fix is the sum domain of spec §21.11, `domain Node = Item + Pair +
  labels("S", "T")` with `Node(i)` injecting and `Node.S` naming the extra
  positions. It is a core feature (`domain A + B` is a type error today), so
  it belongs to the core-extensions lane; `std.flow` needs nothing special
  from it, because every API here is generic over `V`. Until it lands, the
  library removes the most common reason for the flattening: `max_closure`,
  bounded flows and the matchings build their own source and sink inside a
  fresh domain, so the contestant's node space is only their own domains.

## 5. What stays user code

- **The model.** Which arcs, which capacities, vertex splitting, layered
  time-expanded graphs: this is the problem. The library makes it cheap to
  write (typed nodes, handles, unbounded arcs) and does not guess it.
- **Corollaries that are one line from a result:** maximum independent set
  (complement of `min_vertex_cover`), minimum DAG path cover
  (`V.K - max_matching(...).size`), the cut arcs
  (`arcs.filter(a => side[g.source(a)] and not side[g.target(a)])`), the
  min cut between `s` and `t` from a Gomory–Hu tree (a path minimum).
- **Fractional programming around a solver** (`sdoi17r1p4`): a binary
  search over a weight function the solver takes as a closure.
- **Problem-specific structure on top of a cut** (`cf2147h`'s Gallai parity
  split): the library gives the tree; the parity argument is the idea.

## 6. Exemplars

Real problems written against the API. The ports under `proto/ports/` that
these replace are noted; sum domains (section 4) are the one feature used
here that the core does not have yet.

### cf2147h: H. Maxflow GCD Coloring

```cp
// The gcd of all pairwise max flows is the gcd of the Gomory–Hu tree's
// weights. If it is not 1, one colour; otherwise Gallai's parity split.
@!multitest
domain V = read(1..=50)
let m: 0..=choose2(V.count) = read()
let edges = read<[m](u: V, v: V, w: 1..=1e6)>()
if gomory_hu(edges).map(e => e.w).fold(gcd) != 1: return answer([V.all])
// ... even_split as in design/EXEMPLARS-PORTS.md
```

The port reimplemented Gusfield: a parent array, `n - 1` resets, a
`max_flow`, a `min_cut` read and the re-parenting loop. The idea is "gcd of
all pairwise max flows equals the gcd of the cut tree's weights"; the line
says it.

### cf2062g: G. Permutation Factory

```cp
// Half the minimum Manhattan matching between points (i, p[i]) and
// (j, q[j]) is a lower bound and is achieved by sorting in two stages.
let (_, mate) = min_weight_matching<I, I>((i, j) => i.dist(j) + p[i].dist(q[j]))
```

The objective is a minimum; it is written as one. The port negated every
weight into `max_weight_matching` and repeated `.ord` subtraction for the
distances; `dist` is the domain distance of spec §21.

### DMOPC '20 C7 P5 (project selection)

```cp
// Take every reward; forfeit a pair or buy both its items. Max closure.
with io(base: 1)
domain Item = read(1..=5000)
domain Pair = read(0..=5000)
let cost: [Item]0..=1e9 = read()
let pairs: [Pair](u: Item, v: Item, w: 0..=1e9) = read()

domain Node = Item + Pair
fn profit(v: Node) -> Int = match v: Item(i) => -cost[i], Pair(e) => pairs[e].w
let needs = Graph.directed(
    [(u: Node(i), v: Node(e)) for e in Pair for i in [pairs[e].u, pairs[e].v]]
)
print(max_closure(before: needs, profit: profit).value)
```

The port flattened `Item + Pair + Src + Snk` into `0..n + m + 2`, wrote the
source and sink arcs, and printed `total_reward - max_flow`: the textbook
reduction, re-derived. `max_closure` names it, and its `take` answers the
"which projects" variant with no second pass over the residual graph.
`noi06p4` and `noi09p4` are the same shape.

### cf863F: F. Almost Permutation (convex costs)

```cp
// Each position takes one value in [lo, hi]; the k-th use of a value costs
// 2k - 1, so total cost is sum cnt^2. Min-cost flow of n units.
domain Pos = read(1..=50)
domain Val = 1..=Pos.count
var lo: [Pos]Val = Val.first!
var hi: [Pos]Val = Val.last!
for _ in 0..read<0..=100>():
    let (t, l, r, v) = read<(1..=2, Pos, Pos, Val)>()
    for x in l..=r:
        if t == 1: lo[x] = max(lo[x], v) else: hi[x] = min(hi[x], v)

domain Node = Pos + Val + labels("S", "T")
var net = CostFlow<Node>()
for x in Pos:
    net.add(Node.S, Node(x), 1, cost: 0)
    for v in lo[x]..=hi[x]: net.add(Node(x), Node(v), 1, cost: 0)
for v in Val:
    for k in 1..=Pos.count: net.add(Node(v), Node.T, 1, cost: 2 * k - 1)
let (flow, cost) = net.min_cost_flow(Node.S, Node.T)
print(if flow == Pos.count: cost else: -1)
```

Convex costs as unit arcs of increasing cost is the standard trick; it is
user code because it is the model.

### SDOI 2017 Ball Dance (assignment inside a binary search)

```cp
// sum a / sum b >= r iff the best assignment of a - r b is >= 0.
let ok = r => max_weight_matching<Boy, Girl>((i, j) => a[i][j] * SCALE - r * b[i][j]).total >= 0
print(partition_point(0..=hi, ok) - 1)
```

The solver takes the weight as a closure, so the search re-solves without
building a matrix per probe.

## 7. The maximal version

The whole area, in the order the corpus is likely to want it:

1. **Sum domains** (core lane): network nodes as `Item + Pair + labels(...)`.
   The largest single friction in the area.
2. **`global_min_cut(edges)`** (Stoer–Wagner, O(V^3)) next to `gomory_hu`.
   Shipped: the dense form, since a global cut problem has small `V`.
3. **`max_weight_general_matching`** (Edmonds weighted, O(V^3)): shipped, dense, like
   `global_min_cut`. Still open:
   `max_weight_matching` over sparse edges (min-cost flow on unit arcs).
4. **`min_cost_circulation`** with negative cycles (cycle cancelling or
   cost scaling), and `min_cost_flow` over bounded arcs.
5. **Dynamic networks:** `set_cap(a, c)` with re-flow from the current
   flow, as ACL's `change_edge`; incremental bipartite matching.
6. ~~Push-relabel as a named second max-flow solver~~: landed as
   `FlowGraph.push_relabel(s, t)` (decision 112). On cf1404e's 200 x 200
   all-`#` grid the port takes 24.5 ms (Dinic: 442 ms; the original: 17 ms).
   Its solve is 1.02x matched C++ (`bench/flow/prgrid`).
7. **Matroid intersection** and **bipartite edge colouring** (KACTL).
8. **Tooling:** `cpl explain` for a network (vertex and arc counts, the
   bound the solver will meet), and a trace view of a min cut over the
   user's own domains.
