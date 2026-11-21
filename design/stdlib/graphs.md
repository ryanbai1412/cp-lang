# Graphs: the `std.graph` design

Owner: the graphs stdlib lane. Scope: shortest paths (BFS, 0-1 BFS,
Dijkstra, fixed-cost searches, Bellman-Ford, Floyd-Warshall), order (topological
sort, peeling, cycles), connectivity (components, SCC, condensation, bridges,
articulation points, blocks), bipartiteness, Euler walks, 2-SAT, functional
graphs, implicit graphs and grid graphs. Trees (`Rooted`), flows and matchings
have their own sections of `plan/STDLIB.md` and are only touched here where a
graph result feeds them.

The design is written against the current language: everything below is
ordinary CP Lang in `stdlib/graph.cp` and its neighbours, with no compiler
knowledge of graphs. Where the best version needs a general core feature, the
feature is named and the graph API works without it.

## 1. What contests ask for

Survey sources: the checked-in ecnerwala originals
(`proto/ports/ecnerwala/*_orig.cpp`, 280 files), the graph-tagged ports under
`proto/ports/`, the friction reports in `research/frictions/`, KACTL's graph
chapter, the AtCoder Library (`scc_graph`, `two_sat`, `dsu`), and the graph
problems that recur on Codeforces, AtCoder (ABC/ARC) and ICPC sets (CSES's
graph section is a fair index of the classics).

The patterns, roughly by frequency in that corpus:

1. **Unweighted distances from one or many sources.** Grids, implicit state
   spaces and plain adjacency; often with the predecessor for path output.
   Multi-source BFS (from all walls, all ones, all cycle vertices) is as common
   as single-source. About 30 of the ecnerwala originals walk a grid with a
   direction table.
2. **Weighted distances.** Dijkstra from one source or a weighted set of
   sources (a super source in disguise), with path recovery or a DP over the
   shortest-path DAG. Weights are often a function of the edge (`w[e]`), the
   direction walked (`ucup5s06g`: 0 forwards, 1 backwards) or the state.
3. **0-1 and fixed-cost moves.** Costs in {0, 1} (deque BFS), or a small set of
   moves with fixed costs over a huge implicit state space (`cf2178h`: one cursor
   per move, no heap). KACTL and ACL omit these; contestants rewrite them each
   time, and that is where the time goes.
4. **Negative weights.** Bellman-Ford with negative-cycle detection (KACTL
   `BellmanFord`), Floyd-Warshall on n <= 500 (KACTL `FloydWarshall`).
5. **Order.** Topological order or "is it a DAG", DP in topological order,
   peeling in-degree zero (`cf2115b`, `cf2137g`, `ucup5s06g`), finding one
   directed cycle to print.
6. **Strongly connected components** and the condensation DAG (ACL
   `scc_graph`, KACTL `SCC`, `ucup4s25c`), almost always followed by a DP
   over the components in topological order.
7. **Undirected structure.** Connected components; bridges and 2-edge
   components with parallel edges; articulation points and vertex blocks
   (KACTL `BiconnectedComponents`), block-cut trees; DFS-tree low-link logic
   that carries problem-specific constraints (`cf2135c`).
8. **Bipartiteness.** Two-colouring with an odd cycle as the failure, colour
   classes per component (`cf2245g`), parity DSU when edges arrive online.
9. **Euler walks.** Hierholzer on directed or undirected multigraphs, with the
   edge order (KACTL `EulerWalk`); de Bruijn-style constructions.
10. **2-SAT.** Clauses over typed variables, "at most one of" groups, an
    assignment or unsatisfiable (ACL `two_sat`, KACTL `2sat`).
11. **Functional graphs.** `v -> f(v)`: rings with trees hanging off them,
    k-th successor, distance to the ring, when two walks meet.
12. **Implicit graphs.** Vertices are states (`(r, subset)`, `(cell, dir)`,
    `(pos, keys)`); edges are moves computed on the fly. The library must take
    a closure as readily as a stored graph.

What is not graph-library work: minimum spanning trees, DSU and parity DSU
live with `std.dsu`; flows and matchings with `std.flow`; trees, LCA and
virtual trees with `std.tree`.

## 2. Principles

**One graph protocol.** Every algorithm takes `next: N` where `N` is anything
`Neighbours`: a `Graph<V, E>`, a table `[V]List<V>`, or a closure
`u => ...` returning or yielding successors. A successor is either a vertex or
an arc `(to: V, e: E)`; the `Target` trait reads `.to` from either. So the
same `bfs` runs on a stored graph, a grid and a state space, and an algorithm
that needs edge identities (bridges with parallel edges, Euler walks) asks for
arcs and says so in its signature.

**Index spaces carry meaning.** Vertices are a domain `V`, edges a domain `E`;
results are arrays over them (`[V]?Int`, `[E]Bool`), so a distance can never
be indexed by an edge. Algorithms that invent a new index space return a
fresh domain existentially (decision 33): `let (domain C, comp) = scc(g)`
makes `C` a real domain with `[C]` arrays, ordered so that the order means
something (topological for `scc`, discovery order for `components`).
Implicit searches index their results by the state domain the contestant
declared, so `dist[(r, s)]` is checked like any array access.

**Names are the textbook's.** `bfs`, `bfs01`, `dijkstra`, `bellman_ford`,
`floyd_warshall`, `topo`, `scc`, `bridges`, `articulation_points`,
`euler_walk`, `TwoSat`, `Functional`. The name says the algorithm, so the
contestant knows the complexity and the compiler never substitutes another
one (AGENTS.md, "name the algorithm").

**Options are named.** `from:`, `w:`, `directed:`, `moves:`, `cost:`,
`step:`. `next` is the only positional argument.

**Results answer the follow-up question.** A search does not return a bare
distance array that forces the contestant to rerun it with parents: it
returns `Paths`, which indexes like the distance array and also knows how
each vertex was reached. Failure is an optional (`topo` is `none` on a
cycle), never a sentinel.

**Costs are written on every function**, in terms of n = |V|, m = number of
arcs examined, and the cost of one call to a user closure counted as O(1).

## 3. The API

### 3.1 Stored graphs

```cp
trait Neighbours            // fn out(self, u) -> yields successor
trait Target                // fn to(self) -> V; a bare V is its own target
struct Graph<V, E>          // CSR: O(n + m) memory, arcs grouped by tail
    fn directed(edges: [E](u: V, v: V)) -> Graph<V, E>     // O(n + m)
    fn undirected(edges: [E](u: V, v: V)) -> Graph<V, E>   // O(n + m), 2 arcs per edge
    fn out(self, u: V) -> yields (to: V, e: E)             // O(deg u)
    fn get(self, u: V) -> List<(to: V, e: E)>              // g[u], O(deg u), allocates
    fn deg(self, u: V) -> Int                              // O(1)
    fn transpose(self) -> Graph<V, E>                      // O(n + m)
// List-form hooks, never called by name: `Graph.directed(pairs)` with a plain
// list resolves to `directed_of_pairs`, and a list of triples to
// `directed_of_triples` (the general `<fn>_of_pairs`/`<fn>_of_triples` rule).
fn directed_of_pairs<V>(edges: List<(V, V)>) -> [V]List<V>                 // O(n + m)
fn undirected_of_pairs<V>(edges: List<(V, V)>) -> [V]List<V>               // O(n + m)
fn directed_of_triples<V, W>(edges: List<(V, V, W)>) -> [V]List<(V, W)>    // O(n + m)
fn undirected_of_triples<V, W>(edges: List<(V, V, W)>) -> [V]List<(V, W)>  // O(n + m)
```

One constructor per kind of graph, not a `directed: Bool` option. The
contestant writes two spellings, `Graph.directed(edges)` and
`Graph.undirected(edges)`, and the tuple shape picks pairs, triples or an
edge-domain array; the four `*_of_*` functions are the hooks the compiler's
list-form rule resolves to, not names anyone types. Directedness is a fact
the statement gives once and the value keeps, so it reads as the type of
graph being built. `euler_walk(next, directed:)` differs because it takes an
arbitrary `next` that cannot say which kind it is, and the option changes
the algorithm's degree condition. Reverse: add
`Graph(edges, directed: Bool)` and keep the two constructors as its
spellings.

Laws: `Graph.directed(edges)` has one arc `(to: v, e)` out of `u` per edge, in
edge order; `undirected` adds `(to: u, e)` out of `v`, so both directions of
one edge share `e` (that is what makes parallel edges and "don't walk back
along the same edge" correct). `g.transpose().transpose()` has the same arcs
as `g`.

### 3.2 Shortest paths

```cp
struct Paths<V, W, A>       // the result of every single-pair-family search
    dist: [V]?W             // none: unreachable
    back: [V]?(from: V, via: A)  // the vertex before v on one shortest path
                                 // and the arc (or move) that reached v
    order: List<V>          // reached vertices in settling order
    fn get(self, v: V) -> ?W                  // p[v], O(1)
    fn reached(self, v: V) -> Bool            // O(1)
    fn path(self, t: V) -> ?List<V>           // source .. t, O(len)
    fn arcs(self, t: V) -> ?List<A>           // arcs from source to t, O(len)

fn bfs<V, N>(next: N, from: List<V>, until: ?V = none) -> Paths<V, Int, A>
    // O(n + m)
fn bfs01<V, N, A>(next: N, from: List<V>, w: A -> Int, until: ?V = none)
    -> Paths<V, Int, A>
    // O(n + m); deque; traps unless w(a) is 0 or 1
fn dijkstra<V, N, A, W>(next: N, from: List<(V, W)>, w: A -> W, until: ?V = none)
    -> Paths<V, W, A>
    // O((n + m) log m) binary heap with lazy deletion; traps on a negative weight
fn fixed_cost_paths<S, M, W>(
    from: List<S>, moves: List<M>, cost: M -> W, step: (M, S) -> ?S, until: ?S = none,
) -> Paths<S, W, M>
    // O(|S| * |moves|) time, O(|S|) memory: one cursor per move, no heap

`until: t` stops a search once `t` is settled: `p[t]` and every distance in
`p.order` are exact, and `p.order` ends at `t`; a vertex outside `p.order`
holds `none` or an upper bound. The cost bounds become the part of the space
cheaper than `t`, which is what a hand-written loop that stops at the target
pays (`cf2178h`).
fn bellman_ford<V, N, A, W>(next: N, from: List<(V, W)>, w: A -> W) -> ?Paths<V, W, A>
    // O(n * m); none when a negative cycle is reachable from a source
fn floyd_warshall<V, W>(w: [V, V]?W) -> ?[V, V]?W
    // O(n^3) time, O(n^2) memory; none on a negative cycle
```

Sources: unit and 0-1 searches start every source at distance 0;
weighted searches take `(vertex, starting distance)`, which is the super
source without the extra vertex, and lets "start at `a[v]` from every `v`"
be one argument.

Laws, for every search and every reached `v`:
- `p[s] == 0` (or the given start) for a source `s` that no cheaper path
  reaches; `p[v] == none` exactly when `v` is unreachable.
- Optimality: for every arc `a` from reached `u`, `p[a.to]! <= p[u]! + w(a)`.
- Witness: for non-source `v` with `b = p.back[v]!`, `p[v]! == p[b.from]! + w(b.via)`, so
  `path(t)` starts at a source, ends at `t`, and its weights sum to `p[t]!`.
- `order` is non-decreasing in `p`; every vertex appears after its `from`,
  so a DP over the shortest-path DAG is a loop over `order`.
- `bfs(next, from)` equals `dijkstra(next, [(s, 0) for s in from], a => 1)`
  and `bfs01` equals `dijkstra` with the same `w` (as distances; ties in
  `back` may differ).

`Paths` costs two arrays of n plus the order list. That is what the C++
original keeps whenever it prints a path; when the distance is all a
problem needs, the witness is one extra write per improving relaxation, and the
benchmarks in `bench/graph/` measure them against a distance-only C++ BFS and
Dijkstra.

`fixed_cost_paths` is the `cf2178h` shape: a state domain `S`, a finite list
of moves each with a fixed non-negative cost, and a partial transition. With
states settled in non-decreasing distance, each move keeps a cursor into
the settle order: its first state whose step is still unsettled is its
cheapest candidate, so settling the cheapest of k candidates keeps the order
without a heap or a queue of pending states. It is the right tool whenever the move set is small and the state
space is large; for arbitrary per-arc weights use `dijkstra` on a closure.

### 3.3 Order and cycles

```cp
fn topo<V, N>(next: N) -> ?List<V>          // O(n + m) Kahn; none on a cycle
fn peel<V, N>(next: N) -> List<V>           // O(n + m) repeatedly remove in-degree 0
fn find_cycle<V, N>(next: N) -> ?List<V>    // O(n + m) one directed cycle, in order
```

Laws: in `topo(g)!` every arc goes from an earlier vertex to a later one;
`topo(g) == none` iff `find_cycle(g) != none`; `peel(g)` is a prefix-closed
removal order and its complement is exactly the vertices on or downstream
of a cycle (the vertices that keep an incoming arc forever). For a closed
cycle `c = find_cycle(g)!`, each `c[i + 1]` is a successor of `c[i]` and
`c.last` reaches `c[0]`.

### 3.4 Connectivity

```cp
fn components<V, N>(next: N) -> (domain C, comp: [V]C)          // O(n + m)
fn scc<V, N>(next: N) -> (domain C, comp: [V]C)                 // O(n + m) Tarjan
fn condense<V, N, C>(next: N, comp: [V]C) -> [C]List<(to: C)>   // O(n + m)
fn two_edge_components<V, N>(next: N) -> (domain C, comp: [V]C) // O(n + m), arcs
fn bridges<V, E, N>(next: N) -> [E]Bool                         // O(n + m), arcs
fn articulation_points<V, N>(next: N) -> [V]Bool                // O(n + m)
fn biconnected_components<V, N>(next: N) -> List<List<V>>       // O(n + m)
```

Laws:
- `components`: `comp[u] == comp[v]` iff `u` and `v` are joined by a path
  (`next` is read as undirected: give it both directions).
- `scc`: `comp[u] == comp[v]` iff each reaches the other, and every arc
  `u -> v` has `comp[u] <= comp[v]`, so `C` in order is a topological order
  of the condensation. (ACL numbers the same way.)
- `two_edge_components` and `bridges` need arcs with edge ids, so a parallel
  edge is not mistaken for the tree edge back to the parent; an edge `e` is
  a bridge iff its ends lie in different 2-edge components.
- `articulation_points[v]` iff removing `v` increases the number of
  components; every edge lies in exactly one `biconnected_components` block
  and two blocks share at most one vertex, an articulation point.

All the DFS-based algorithms are iterative: no recursion depth limit applies
on a path of 10^6 vertices.

### 3.5 Bipartiteness

```cp
fn two_color<V, N>(next: N) -> ?[V]Bool     // O(n + m); none iff an odd cycle
```

Law: for every arc `u -> v`, `colour[u] != colour[v]`; within each component
the first vertex in `V` order gets `false`, so the answer is deterministic and
per-component class sizes are a fold over `components`. Online parity
constraints belong to the parity DSU in `std.dsu`.

### 3.6 Euler walks

```cp
struct Walk<V, E>
    start: V
    arcs: List<(to: V, e: E)>
    @get fn vertices(self) -> List<V>       // start then each arc's head, O(len)
fn euler_walk<V, E>(edges: [E](u: V, v: V), directed: Bool) -> ?Walk<V, E>
    // O(n + m) Hierholzer; uses every edge once; none if no such walk
```

Laws: the walk uses every `e` in `E` exactly once, consecutive arcs share a
vertex, and an undirected edge may be walked either way (the arc says which
way it was used, by its head). The walk is closed whenever every vertex is
balanced, and then starts at the smallest vertex with an edge; otherwise it
starts at the unique vertex with out-degree surplus (directed) or the smaller
odd vertex (undirected). `none` iff the degree condition fails or the edges
are not in one component.

### 3.7 2-SAT

```cp
struct Lit<X>(x: X, value: Bool)
    @get fn flip(self) -> Lit<X>
struct TwoSat<X>                            // O(|X| + clauses) memory
    fn new() -> TwoSat<X>
    fn add_or(mut self, a: Lit<X>, b: Lit<X>)        // O(1)
    fn implies(mut self, a: Lit<X>, b: Lit<X>)       // O(1)
    fn must(mut self, a: Lit<X>)                     // O(1)
    fn at_most_one(mut self, lits: List<Lit<X>>)     // O(k), prefix auxiliaries
    fn solve(self) -> ?[X]Bool              // O(|X| + clauses), SCC
```

Laws: `solve()` is `none` iff no assignment satisfies every clause, and an
assignment satisfies every clause added. The variables are a domain `X`, so
"variable 2i+1 means the negation of 2i" never appears in user code.

### 3.8 Functional graphs

```cp
struct Functional<V>                        // O(n log n) build (jump table)
    fn new(next: [V]V) -> Functional<V>
    fn on_ring(self, v: V) -> Bool          // O(1)
    fn ring_of(self, v: V) -> Int           // O(1)
    fn steps_to_ring(self, v: V) -> Int     // O(1)
    fn root(self, v: V) -> V                // O(1) first ring vertex reached
    fn jump(self, v: V, k: Int) -> V        // O(log n), any k >= 0
    fn dist(self, a: V, b: V) -> ?Int       // O(log n) steps from a to b
    fn orbit(self, v: V) -> Int             // O(1) distinct vertices from v
    fn jump_while(self, v: V, pred: V -> Bool, max_steps: Int = V.K) -> ?(V, Int)
```

Laws: `jump(v, a + b) == jump(jump(v, a), b)`; `jump(v, steps_to_ring(v))
== root(v)`; `on_ring(v)` iff `jump(v, k) == v` for some `k >= 1`;
`dist(a, b) == some(k)` iff `k` is the least `k` with `jump(a, k) == b`.

### 3.9 Implicit and grid graphs

There is no separate implicit-graph type: a closure is a graph. The vertex
domain is whatever the contestant declared, including a product of domains
(`(r: R, s: Subset<T>)`) once the core lane lands product domains, and a
`(Row, Col)` cell today:

```cp
let p = bfs(c => [d for d in adj4(c) if wall[d] == false], from: [start])
```

`std.grid` supplies the moves (`Dir4`, `Dir8`, `step`, `adj4`, `adj8`, `ray`);
the graph library never learns what a grid is. A state with extra
coordinates is a tuple of domains, and `fixed_cost_paths` takes the moves
as a value (an enum's `all`) so the transition is written once, as a
function of the move.

## 4. What stays user code

- The transition itself: which moves exist, what they cost, which states are
  legal. The library never takes a "grid with walls" or a "state encoder".
- DFS whose per-vertex logic is the problem (`cf2135c` propagates a forced
  value along low-link): a recursive `fn dfs` over `g.out(u)` reads like the
  editorial and the library has nothing to add except `Graph`.
- DP over an order the library provides (`order`, `topo`, `C`): the loop is
  the recurrence, so it is the contestant's.
- Counting shortest paths, k-th shortest paths and lexicographically smallest
  paths: a loop over `p.order` with `p.dist` is short and exactly what the
  editorial says; a semiring-generic search would hide it.

## 5. Real problems against the API

### ucup5s06g: free cycle vertices, pay for reversed edges

Vertices on or below a cycle cost nothing; every other vertex pays the
fewest reversed edges needed to be reached from one of them.

```cp
@!multitest
let (n, m): (1..=5e5, 0..=5e5) = read()
domain V = 1..=n
domain E = 0..m
let edges: [E](u: V, v: V) = read()

// What survives peeling in-degree zero keeps an incoming edge: it is free.
var peeled: [V]Bool = false
for v in peel(Graph.directed(edges)): peeled[v] = true

// Walking an edge forwards is free and backwards costs one.
let dist = bfs01(
    Graph.undirected(edges),
    from: [v for v in V if not peeled[v]],
    w: a => if edges[a.e].v == a.to: 0 else: 1,
)
print(if all(dist.reached(v) for v in V): sum(dist[v]! for v in V) else: -1)
```

The weights are 0 and 1, so the algorithm is the deque, not the heap.

### cf2178h: Create or Duplicate

Every move has a fixed cost and the state is the value mod `m` with the set
of types started, so shortest path over a 2^4 * m state space needs no heap.

```cp
domain T = 0..3
enum Move: Create(T), Double(1..=3)

@!multitest
let x: [T]1..=5e5 = read()
let (m, k): (2..=5e5, 1..=5e5) = read()
domain R = Zmod(m)
domain State = (r: R, s: Subset<T>)        // product domain: core lane

fn cost(mv: Move) -> Int = match mv:
    Create(t) => x[t]
    Double(j) => j * k
fn step(mv: Move, (r, s): State) -> ?State = match mv:
    Create(t) => (r + x[t], s.with(t))
    Double(j) => (2 * r, s) if s.len == j

let dist = fixed_cost_paths(from: [(R(0), {})], moves: Move.all, cost:, step:)
print(dist[(R(0), T.all)]! - sum(x))
```

### ucup4s25c: SCCs and BFS depth mod 3

Every edge inside a strongly connected component must step the BFS depth by
one modulo 3.

```cp
@!multitest
let m: Int = read()
var verts: Map<(Int, Int), Int> = Map()
fn id(k: (Int, Int)) -> Int: verts.get_or_insert(k, verts.len)
var raw: List<(Int, Int)> = []
for _ in 0..m:
    let t: [0..3]Int = read()
    for (x, y, z) in t.permutations: raw.push((id((x, y)), id((y, z))))

domain V = verts.len
let edges = [(V(u), V(v)) for (u, v) in raw]
let next = Graph.directed(edges)
let (domain C, comp) = scc(next)
let roots = [first(u for u in V if comp[u] == c)! for c in C]
let depth = bfs(u => [v for v in next[u] if comp[v] == comp[u]], from: roots)
let bad = any(comp[u] == comp[v] and depth[v]! % 3 != (depth[u]! + 1) % 3 for (u, v) in edges)
print(if bad: "yes" else: "no")
```

### CSES 1193 Labyrinth: grid BFS with the path

```cp
let (n, m): (1..=1000, 1..=1000) = read()
domain Row = 0..n
domain Col = 0..m
let g: [Row, Col]Char = read()
let at = (c: Char) => first(p for p in (Row, Col) if g[p] == c)!

let p = bfs(c => [d for d in adj4(c) if g[d] != '#'], from: [at('A')])
if let path = p.path(at('B')):
    print("YES", path.len - 1)
    print(String([dir4_between(a, b).letter for (a, b) in path.pairs]))
else:
    print("NO")
```

`dir4_between` and `.letter` are `std.grid`'s side of the bargain; the graph
side is one `bfs` and `path`.

### CSES 1684 Giant Pizza: 2-SAT over typed toppings

```cp
let (n, m): (Int, 1..=1e5) = read()
domain Topping = 1..=m
var sat = TwoSat<Topping>.new()
for _ in 0..n:
    let (a, x, b, y): (Char, Topping, Char, Topping) = read()
    sat.add_or(Lit(x, a == '+'), Lit(y, b == '+'))
if let pick = sat.solve():
    print([if pick[t]: '+' else: '-' for t in Topping], sep: " ")
else:
    print("IMPOSSIBLE")
```

## 6. Dependencies on the core lane

- Product domains (`domain State = (r: R, s: Subset<T>)`) for implicit state
  spaces; until then a state is encoded into a declared domain by the
  contestant (`cf2178h` today).
- `Zmod(m)` as a domain, and enum-valued domains (`Move.all`).
- Allocation-free closure successors: a closure that yields instead of
  returning a `List` makes `bfs` over a grid allocation-free. The graph code
  already iterates successors with `for`, so it gains this for free.

## 7. The maximal version

- `block_cut_tree` returning a `Rooted` over a fresh block-plus-vertex
  domain, so "queries on paths through articulation points" becomes LCA.
- `dominator_tree` (Lengauer-Tarjan) returning `Rooted`.
- Shortest-path DAG views: `p.dag` as a `Neighbours` over the tight arcs, so
  counting paths is `dp` over `p.order` with `p.dag`.
- `max_dist:` on the searches, next to the landed `until: t`.
- Johnson's reweighting, `k_shortest` (Eppstein) and minimum mean cycle are
  library candidates only once ports ask for them.
