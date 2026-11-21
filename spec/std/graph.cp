//! Graphs: one stored representation and the algorithms over "what comes after v".
//!
//! Vertices and edges are domains, so per-vertex data is `[V]T`, per-edge
//! data is `[E]T`, and mixing the two does not type-check. An algorithm
//! takes `next`: a stored `Graph`, or any fn from a vertex to what follows
//! it. What follows is a list of vertices, or of arcs (records with a `to`
//! field) when the algorithm needs to know which edge it took:
//!
//!     bfs(g, from: [s])
//!     bfs(from: [s], next: u => lists[u])
//!     bfs(from: [s], next: p => p.adj4)
//!     dijkstra(g, from: [(s, 0)], w: a => edges[a.e].w)
//!
//! Weights are read through a fn of the arc, so the same graph serves several
//! weightings. Unreachable is `none`. An algorithm given only a fn cannot see
//! the vertex type, so it is named: `scc::<V>(u => out[u])`; given a `Graph`
//! it is read off the graph: `scc(g)`.
use std.heap.{MinHeap}
use std.dsu.{Dsu}
use std.seq.{reverse}
use std.deque.{Deque}

/// O(deg u) per walk of `out(u)`.
/// What an algorithm walks: a `Graph` gives its arcs out of `u`, and any fn `u => ...` is
/// simply called.
pub trait Neighbours:
    fn out<V>(self, u: V) -> Iter<V> = todo

/// O(1).
/// What a neighbour list holds: an arc says where it leads with its `to` field, and a plain
/// vertex leads to itself.
pub trait Target:
    @get fn to(self) -> Self = todo

/// O(n) to read.
/// The `V.len - 1` edges of a tree on `V`, one `u v` pair per line, as an input
/// gives them. `cpl stress` makes and shrinks it as a tree.
pub type Tree<V> = [0..V.len - 1](u: V, v: V)

/// O(m) to read.
/// One edge on `V` for each value of `E`, `u v` per line, with no loop and no
/// pair twice. `cpl stress` makes and shrinks it so.
pub type SimpleGraph<V, E> = [E](u: V, v: V)

/// O(n + m) to read (connectivity is validated).
/// A `SimpleGraph` in which every vertex reaches every other.
pub type ConnectedGraph<V, E> = [E](u: V, v: V)

/// O(m) to read.
/// One arc `u -> v` on `V` for each value of `E`, with no arc twice and no
/// directed cycle. `cpl stress` makes and shrinks it so.
pub type Dag<V, E> = [E](u: V, v: V)

/// O(n + m) to read (acyclicity is validated).
/// One arc `u -> v` on `V` for each value of `E`, with no loop and no arc
/// twice; `u v` and `v u` are two arcs. `cpl stress` makes and shrinks it so.
pub type Digraph<V, E> = [E](u: V, v: V)

/// O(n + m) memory and build.
/// A fixed graph in compressed sparse row form: one offsets list and one arcs list.
/// Built in O(n + m); memory is n + 1 offsets and one `(to, e)` per arc.
/// An undirected edge is two arcs with the same `e`.
pub struct Graph<V, E>:
    start: List<0..=2_000_000_000>
    arcs: List<(to: V, e: E)>

    /// O(n + m).
    /// One arc `u -> v` per edge.
    pub fn directed(edges: [E](u: V, v: V)) -> Graph<V, E> = todo

    /// O(n + m).
    /// Two arcs per edge, `u -> v` and `v -> u`, sharing the edge id.
    pub fn undirected(edges: [E](u: V, v: V)) -> Graph<V, E> = todo

    fn build(edges: [E](u: V, v: V), both: Bool) -> Graph<V, E> = todo

    /// O(deg u) total, O(1) per arc.
    /// The arcs out of `u`, in edge order: what makes a `Graph` usable wherever `next` is
    /// expected. O(deg u).
    pub fn out(self, u: V) -> yields (to: V, e: E) = todo

    /// O(deg u): copies.
    /// `g[u]`: the arcs out of `u`, as a list.
    pub fn get(self, u: V) -> List<(to: V, e: E)> = todo

    /// The number of arcs out of `u`. O(1).
    pub fn deg(self, u: V) -> Int = todo

    /// O(deg u): is there an arc `u -> v`? A scan of `u`'s arcs, not a lookup.
    pub fn has(self, u: V, v: V) -> Bool = todo

    /// O(n + m): the out-neighbours of every vertex as plain lists, the shape
    /// `std.flow` matchings take (`max_matching(g.lists())`).
    pub fn lists(self) -> [V]List<V> = todo

    /// O(n (n + m)): fewest arcs between every ordered pair, by a BFS from each
    /// vertex. Traps on an unreachable pair; a disconnected graph wants `bfs`.
    pub fn hops(self) -> [V, V]Int = todo

    /// O(n + m): which vertices some vertex of `from` reaches without crossing an
    /// arc `u -> v` for which `skip(u, v)` holds.
    pub fn reachable(self, from: List<V>, skip: (V, V) -> Bool = (u, v) => false) -> [V]Bool = todo

    /// O(m): every edge once, with its id and its ends as given.
    pub @get fn edges(self) -> List<Edge<V, E>> = todo

    /// O(n + m): the DFS tree from `root` of the component holding it. Every
    /// non-tree edge of an undirected graph then joins a vertex to an ancestor.
    pub fn dfs_tree(self, root: V) -> DfsTree<V, E> = todo

    /// O(n + m): `peel(g, ...)` as a method, with the same options.
    pub fn peel(
        self, ready: ?(V -> Bool) = none, on_remove: ?((V, (to: V, e: E)) -> ()) = none,
    ) -> List<V> = todo

    /// The graph with every arc reversed. A rebuild, O(n + m).
    pub fn transpose(self) -> Graph<V, E> = todo

/// O(n + m).
/// Kahn's peeling: repeatedly remove a vertex with no arcs left coming in, and return the
/// vertices in the order they were removed. What stays is every vertex on a cycle or
/// reachable from one, so the order covers all of `V` exactly when the graph is acyclic.
/// For a vertexless closure graph, pass the vertex domain via turbofish: `peel::<V>(u => ...)`.
/// O(n + m).
pub fn peel<V, N, A>(
    next: N, ready: ?(V -> Bool) = none, on_remove: ?((V, A) -> ()) = none,
) -> List<V> = todo

/// O(n + m).
/// A topological order, or `none` if there is a cycle: a `peel` that removed every vertex.
/// O(n + m). For a vertexless closure graph, pass the vertex domain via turbofish:
/// `topo::<V>(u => ...)`.
pub fn topo<V, N>(next: N) -> ?List<V> = todo

/// O(n + m).
/// Strongly connected components. `C` is a fresh domain, numbered so that
/// every arc goes from a smaller or equal component to a larger or equal
/// one (a topological order of the condensation). O(n + m), no recursion.
pub fn scc<V, N>(next: N) -> (domain C, comp: [V]C) = todo

/// O(n + m) memory and build.
/// Compressed sparse row adjacency table over `V` built from pairs or triples.
/// O(n + m) time; memory is n + 1 32-bit offsets and one arc per directed edge.
pub struct Adj<V, A>:
    start: List<0..=2_000_000_000>
    arcs: List<A>

    /// The arcs out of `u`. O(deg u).
    pub fn out(self, u: V) -> yields A = todo

    /// `g[u]`: the arcs out of `u`, as a list. O(deg u).
    pub fn get(self, u: V) -> List<A> = todo

    /// The number of arcs out of `u`. O(1).
    pub fn deg(self, u: V) -> Int = todo

    /// O(n + m): the table with every arc reversed.
    pub fn transpose(self) -> Adj<V, A> = todo

    /// O(deg u): is `v` among `u`'s neighbours? A scan, not a lookup.
    pub fn has(self, u: V, v: V) -> Bool = todo

/// O(n + m).
/// `Graph.directed(pairs)` on a plain list of `(u, v)`: CSR adjacency of each vertex.
/// No edge domain, and `for v in g[u]` yields the vertex itself. O(n + m).
pub fn directed_of_pairs<V>(edges: List<(V, V)>) -> Adj<V, V> = todo

/// `Graph.undirected(pairs)` on a plain list of `(u, v)`: each edge both ways in CSR. O(n + m).
pub fn undirected_of_pairs<V>(edges: List<(V, V)>) -> Adj<V, V> = todo

/// `Graph.directed(triples)` on a list of `(u, v, w)`: `for (v, w) in g[u]` in CSR. O(n + m).
pub fn directed_of_triples<V, W>(edges: List<(V, V, W)>) -> Adj<V, (V, W)> = todo

/// O(n + m).
/// `Graph.undirected(triples)` on a list of `(u, v, w)`: each edge both ways in CSR, `w` riding
/// along. O(n + m).
pub fn undirected_of_triples<V, W>(edges: List<(V, V, W)>) -> Adj<V, (V, W)> = todo

/// Connected components of an undirected graph, as a fresh domain `C`. O(n + m).
pub fn components<V, N>(next: N) -> (domain C, comp: [V]C) = todo

/// O(n + m).
/// The condensation: the arcs between components, as adjacency lists (duplicates kept).
pub fn condense<V, N, C>(next: N, comp: [V]C) -> [C]List<(to: C)> = todo

/// O(n + m).
/// Two-edge-connected components (bridge blocks) of an undirected graph.
/// Each arc in `next.out(u)` carries `(to: V, e: E)` so parallel edges are handled
/// properly (parallel edges are never bridges). Returns a fresh domain `C`
/// for the components and `comp: [V]C`. Every edge between distinct components
/// is a bridge, so `condense(next, comp)` yields the bridge tree.
/// O(n + m), iterative Tarjan low-link.
pub fn two_edge_components<V, N>(next: N) -> (domain C, comp: [V]C) = todo

/// Articulation points (cut vertices) of an undirected simple graph.

/// Handles disconnected graphs and root child count. O(n + m), iterative.
pub fn articulation_points<V, N>(next: N) -> [V]Bool = todo

/// Vertex-biconnected components (blocks) of an undirected simple graph.
/// Each returned list contains the vertices of one maximal 2-vertex-connected
/// subgraph (or the two endpoints of a bridge). Articulation points appear in
/// every block that shares them. O(n + m), iterative Tarjan.

/// Vertex-biconnected components (blocks) of an undirected graph via Tarjan DFS. O(|V| + |E|).
pub fn biconnected_components<V, N>(next: N) -> List<List<V>> = todo

/// 2-coloring of an undirected graph: returns a side for each vertex, or `none` if an odd cycle exists. O(n + m).
pub fn two_color<V, N>(next: N) -> ?[V]Bool = todo

@test
fn csr_keeps_edge_ids_and_order() = todo

@test
fn bfs_over_a_stored_graph_and_over_a_lambda() = todo

@test
fn dijkstra_matches_floyd_warshall() = todo

@test
fn topo_orders_a_dag_and_rejects_a_cycle() = todo

@test
fn peel_leaves_what_a_cycle_reaches() = todo

@test
fn scc_agrees_with_reachability() = todo

@test
fn components_of_an_undirected_graph() = todo

fn random_edges<V, E>() -> [E](u: V, v: V) = todo

/// Are `a` and `b` connected when edge `skip` and vertex `gone` are removed?
fn linked<V, E>(edges: [E](u: V, v: V), a: V, b: V, skip: ?E, gone: ?V) -> Bool = todo

@test
fn bridges_and_cut_vertices_match_removing_them() = todo

@test
fn blocks_of_a_small_graph() = todo

@test
fn two_color_finds_a_colouring_or_an_odd_cycle() = todo

@test
fn vertexless_closure_graphs_with_turbofish() = todo

@test
fn a_plain_edge_list_needs_no_edge_domain() = todo

/// O(n) memory.
/// What a search found: `p[v]` is the distance to `v` (`none` if unreachable),
/// `back[v]` is `(from, via)`, the vertex before `v` on one shortest path and the
/// arc or move that reached it (`none` at a source), and `order` the reached
/// vertices in settling order, so every vertex comes after its `from` and
/// distances never decrease. A search given `until: t` stops once `t` is settled:
/// distances in `order` are exact, and any other vertex holds `none` or an upper
/// bound. Cost: the witnesses take one slot per vertex and one write per improving
/// relaxation, paid even when only `p[v]` is read.
pub struct Paths<V, W, A>:
    dist: [V]?W
    back: [V]?(from: V, via: A)
    order: List<V>

    /// `p[v]`: the distance to `v`, or `none` if no source reaches it. O(1).
    pub fn get(self, v: V) -> ?W = todo

    /// O(1) per item: `for (v, d) in bfs(g, from: [s])` walks the reached
    /// vertices in settling order with their distances.
    pub fn iter(self) -> yields (v: V, d: W) = todo

    /// O(n + m): the arcs on some shortest path (`p[a.to] == p[u] + w(a)`), the
    /// DAG that counting or listing shortest paths walks in `order`.
    pub fn tight<N>(self, next: N, w: A -> W) -> [V]List<A> = todo

    /// Does some source reach `v`? O(1).
    pub fn reached(self, v: V) -> Bool = todo

    /// O(path length).
    /// The vertices of one shortest path from a source to `t`, both included, or
    /// `none` if `t` is unreachable. O(length).
    pub fn path(self, t: V) -> ?List<V> = todo

    /// The arcs of `path(t)`, in walking order. O(length).
    pub fn arcs(self, t: V) -> ?List<A> = todo

/// O(n + m).
/// Fewest arcs from any of `from` to each vertex; `until: t` stops once `t` is
/// reached. O(n + m).
pub fn bfs<V, N, A>(next: N, from: List<V>, until: ?V = none) -> Paths<V, Int, A> = todo

/// O(n + m).
/// Shortest distances when every weight `w(a)` is 0 or 1, from every vertex of `from`
/// at distance 0: a deque, with 0-arcs pushed to the front; `until: t` stops once
/// `t` is settled. O(n + m); traps on any other weight.
pub fn bfs01<V, N, A>(next: N, from: List<V>, w: A -> Int, until: ?V = none) -> Paths<V, Int, A> = todo

/// O((n + m) log n).
/// Shortest distances for non-negative weights, from `(vertex, starting distance)`
/// pairs; `until: t` stops once `t` is settled. O((n + m) log m) with a binary heap
/// and lazy deletion; traps on a negative weight.
pub fn dijkstra<V, N, A, W>(
    next: N,
    from: List<(V, W)>,
    w: A -> W,
    until: ?V = none,
) -> Paths<V, W, A> = todo

/// O(|S| |M|) time, O(|S|) memory.
/// Shortest distances over a state space `S` whose moves have fixed non-negative
/// costs: from every state of `from` at distance 0, `step(mv, s)` is where move `mv`
/// leads from `s` (`none`: not allowed there), at cost `cost(mv)`. A cursor per move
/// into `order` replaces the heap: settled states come in non-decreasing distance, so
/// each move's first state whose step is still unsettled is its cheapest candidate;
/// `until: t` stops once `t` is settled, and only settled states get a distance.
/// O(|S| * |moves|) time, O(|S|) memory.
pub fn fixed_cost_paths<S, M, W>(
    from: List<S>,
    moves: List<M>,
    cost: M -> W,
    step: (M, S) -> ?S,
    until: ?S = none,
) -> Paths<S, W, M> = todo

/// O(n m).
/// Shortest distances with any weights, from `(vertex, starting distance)` pairs, or
/// `none` when a source reaches a negative cycle. `order` lists each reached vertex
/// after its parent. O(n * m).
pub fn bellman_ford<V, N, A, W>(next: N, from: List<(V, W)>, w: A -> W) -> ?Paths<V, W, A> = todo

/// O(n^3).
/// All-pairs shortest distances from the direct weights `w[u, v]` (`none`: no edge),
/// or `none` if there is a negative cycle. A vertex is at distance 0 from itself
/// unless a cheaper cycle exists. O(n^3) time, O(n^2) memory.
pub fn floyd_warshall<V>(w: [V, V]?Int) -> ?[V, V]?Int = todo

/// O(n + m).
/// One directed cycle, as its vertices in walking order (each is followed by the next,
/// and the last by the first), or `none` if the graph is acyclic. O(n + m), no recursion.
pub fn find_cycle<V, N>(next: N) -> ?List<V> = todo

/// O(n + m).
/// Which edges are bridges: removing one disconnects its ends. Needs arcs with edge ids
/// (a `Graph.undirected`), so a parallel edge is not mistaken for the way back. O(n + m).
pub fn bridges<V, E, N>(next: N) -> [E]Bool = todo

/// O(m) memory.
/// A walk: where it starts, then each arc taken (`to` is where the arc leads, `e` the
/// edge it uses), the same arcs a `Graph` yields.
pub struct Walk<V, E>:
    start: V
    arcs: List<(to: V, e: E)>

    /// The vertices visited, `start` first. O(length).
    pub @get fn vertices(self) -> List<V> = todo

/// O(n + m).
/// A walk that uses every edge exactly once (Hierholzer's algorithm), or `none` if
/// there is none. `directed` says whether `(u, v)` may only be walked from `u` to `v`.
/// A closed walk starts at the smallest vertex with an edge; an open one at the
/// vertex with an extra way out (directed) or the smaller odd vertex. O(n + m).
pub fn euler_walk<V, E>(edges: [E](u: V, v: V), directed: Bool) -> ?Walk<V, E> = todo

/// O(1).
/// A variable or its negation: `Lit(x, true)` is "x holds".
pub struct Lit<X>(x: X, value: Bool):
    /// O(1).
    /// The opposite literal.
    pub @get fn flip(self) -> Lit<X> = todo

/// A conjunction of two-literal clauses over the variables `X`. Every clause is O(1)
/// to add; `solve` is O(|X| + clauses).
pub struct TwoSat<X>:
    /// Implications between literal nodes: node `2 * i` is "variable i is false",
    /// `2 * i + 1` "is true"; variables past `X.K` are helpers of `at_most_one`.
    imp: List<List<Int>>

    /// An instance with no clauses over the variables of `X`. O(|X|).
    pub fn new() -> TwoSat<X> = todo

    /// O(1).
    /// At least one of `a`, `b` holds.
    pub fn add_or(mut self, a: Lit<X>, b: Lit<X>) = todo

    /// O(1).
    /// `a` holds.
    pub fn must(mut self, a: Lit<X>) = todo

    /// O(1).
    /// If `a` holds then `b` holds.
    pub fn implies(mut self, a: Lit<X>, b: Lit<X>) = todo

    /// O(k).
    /// At most one of `lits` holds: one helper variable per literal ("some literal so
    /// far holds") keeps it O(k) clauses instead of O(k^2).
    pub fn at_most_one(mut self, lits: List<Lit<X>>) = todo

    /// O(1): exactly one of `a`, `b` holds.
    pub fn add_xor(mut self, a: Lit<X>, b: Lit<X>) = todo

    /// O(1): `a` and `b` hold together or not at all.
    pub fn add_eq(mut self, a: Lit<X>, b: Lit<X>) = todo

    /// O(n + m) over the clauses.
    /// An assignment satisfying every clause, or `none` if there is none.
    pub fn solve(self) -> ?[X]Bool = todo

    fn clause(mut self, a: Int, b: Int) = todo

fn lit_node<X>(l: Lit<X>) -> Int = todo

@test
fn paths_witness_their_distances() = todo

@test
fn fixed_cost_paths_match_dijkstra_on_the_same_moves() = todo

@test
fn until_stops_at_the_target_with_its_exact_distance() = todo

@test
fn bellman_ford_matches_floyd_warshall_and_finds_negative_cycles() = todo

@test
fn find_cycle_exists_exactly_when_topo_fails() = todo

@test
fn bridges_are_the_edges_whose_removal_disconnects() = todo

@test
fn euler_walks_use_every_edge_once_and_exist_when_brute_force_finds_one() = todo
/// Does `walk` trace connected edges, starting at `start`?
fn follows<V>(edges: List<(V, V)>, walk: List<Int>, directed: Bool, start: V) -> Bool = todo

fn has_walk<V>(edges: List<(V, V)>, directed: Bool) -> Bool = todo

fn extend<V>(edges: List<(V, V)>, directed: Bool, mut used: List<Bool>, at: V, done: Int) -> Bool = todo

@test
fn at_most_one_forbids_every_pair_and_allows_each_single() = todo


// ---- Spec surface without implementations yet ----------------------------

/// O(n + m) total, O(1) per step: a depth-first walk from
/// `from`, yielding `Enter` on discovery and `Exit` when a vertex is finished.
/// `continue` after `Enter(v)` still descends; the loop steers with `break`.
pub fn dfs<V, N>(next: N, from: List<V>) -> yields Visit<V> = todo

/// O(1): one event of `dfs`, the same shape as `Rooted.tour` yields.
pub enum Visit<V>: Enter(v: V), Exit(v: V)

/// O(n + m + W): shortest paths for small integer weights `0 ..= max_w` with a
/// bucket queue (Dial). Same result shape as `dijkstra`.
pub fn dial<V, N, A>(next: N, from: List<V>, w: A -> Int, max_w: Int, until: ?V = none) -> Paths<V, Int, A> = todo

/// O(n m log n): all-pairs shortest paths with negative arcs and no negative
/// cycle (Johnson: one Bellman-Ford, then a Dijkstra per source). `none` if a
/// negative cycle exists.
pub fn johnson<V, N, A, W>(next: N, w: A -> W) -> ?[V, V]?W = todo

/// O((n + m) log n + k log k): the lengths of the `k` shortest walks from any
/// of `from` to `to`, repeats allowed, ascending (Eppstein). Shorter if fewer exist.
pub fn k_shortest<V, N, A, W>(next: N, from: List<V>, to: V, k: Int, w: A -> W) -> List<W> = todo

/// O(n m): some cycle of negative total weight, as its arcs, or `none`.
pub fn negative_cycle<V, N, A, W>(next: N, w: A -> W) -> ?List<A> = todo

/// O(n m): the cycle of least mean weight as (total, length), or `none` on an
/// acyclic graph (Karp).
pub fn min_mean_cycle<V, N, A>(next: N, w: A -> Int) -> ?(sum: Int, len: Int) = todo

/// O(n m): the length of a shortest cycle (the girth), or `none` if acyclic.
/// Undirected graphs count a cycle only if it uses at least three edges.
pub fn girth<V, N>(next: N, directed: Bool) -> ?Int = todo

/// O((n + m) log n): the lexicographically smallest topological order, by a
/// heap instead of a queue; `none` if there is a cycle.
pub fn smallest_topo<V, N>(next: N) -> ?List<V> = todo

/// O(n + m): DAG DP in topological order. Each vertex starts at `init(v)` and
/// every arc `u -> a.to` folds `step(val[u], a)` into `val[a.to]` with the
/// operation value `op` (W10: `op: max` gives longest paths, `op: +` counts).
/// `none` on a cycle.
pub fn dag_fold<V, N, A, T, M>(next: N, init: V -> T, step: (T, A) -> T, op: M) -> ?[V]T = todo

/// O(n + m): the number of `from -> v` paths of a DAG for every `v`, in `T`
/// (a modular type keeps it exact). `none` on a cycle.
pub fn count_paths<T, V, N>(next: N, from: List<V>) -> ?[V]T = todo

/// O(n m / 64): reachability of every pair as bitset rows (transitive
/// closure through the condensation).
pub fn closure<V, N>(next: N) -> [V]Bitset<V> = todo

/// O(m log m): offline incremental SCC; for each arc of `arcs`, added in
/// order, the first time its ends share a strongly connected component.
pub fn scc_times<V, E>(arcs: [E](u: V, v: V)) -> [E]?Int = todo

/// O(n + m): the block-cut tree. `B` numbers the biconnected blocks; the tree
/// joins each block to the cut vertices it contains, as `(block, vertex)` pairs.
pub fn block_cut_tree<V, N>(next: N) -> (domain B, links: List<(b: B, v: V)>) = todo

/// O(n + m): the biconnected block of every edge (edges, not vertices, are
/// partitioned; a cut vertex lies in several blocks).
pub fn edge_blocks<V, E, N>(next: N) -> (domain B, block: [E]B) = todo

/// O(n + m alpha): connected components of the complement of `next` (the
/// arcs of `next` are the missing edges), each as a list of vertices.
pub fn complement_components<V, N>(next: N) -> List<List<V>> = todo

/// O(n + m): an odd cycle witnessing that `next` is not bipartite, or `none`.
pub fn odd_cycle<V, N>(next: N) -> ?List<V> = todo

/// O((n + m) alpha): the immediate dominator of every vertex reachable from
/// `root` (Lengauer-Tarjan); `none` for `root` and unreachable vertices.
pub fn dominators<V, N>(next: N, root: V) -> [V]?V = todo

/// O(m sqrt m), O(1) per item: every triangle of an undirected simple graph once.
pub fn triangles<V, N>(next: N) -> yields (V, V, V) = todo

/// O(3^(n/3)) worst case: a maximum clique of the graph given as bitset rows
/// (Bron-Kerbosch with pivoting); `n` up to about 64 in contests.
pub fn max_clique<V>(adj: [V]Bitset<V>) -> List<V> = todo

/// O(2^n n): the chromatic number by inclusion-exclusion over independent sets.
pub fn chromatic_number<V>(adj: [V]Bitset<V>) -> Int = todo

/// O(k^n) output: the de Bruijn sequence over `0 ..< k` of order `n`, length k^n
/// (an Euler walk of the de Bruijn graph).
pub fn de_bruijn(k: Int, n: Int) -> List<Int> = todo

/// O(m log m): a minimum spanning forest by Kruskal, as the chosen edges and
/// their total. A negated `w` gives a maximum one.
pub fn kruskal<V, E, W>(edges: [E](u: V, v: V), w: E -> W) -> (total: W, take: [E]Bool) = todo

/// O(n^2): a minimum spanning tree of the dense graph `w` (Prim without a
/// heap); `none` if disconnected.
pub fn prim<V, W>(w: [V, V]?W) -> ?(total: W, parent: [V]?V) = todo

/// O(m log n) with `cheapest` answering each component's lightest outgoing
/// edge: Boruvka rounds, for implicit complete graphs (xor MST, Manhattan
/// MST). `cheapest(comp)` returns, per component, its best edge or `none`.
pub fn boruvka<V, C, W>(cheapest: ([V]Int) -> List<?(u: V, v: V, w: W)>) -> (total: W, edges: List<(u: V, v: V)>) = todo

/// O(m log n): a minimum spanning arborescence rooted at `root` (Chu-Liu /
/// Edmonds with a skew heap), as each vertex's chosen parent edge; `none` if
/// some vertex is unreachable.
pub fn arborescence<V, E, W>(edges: [E](u: V, v: V), w: E -> W, root: V) -> ?(total: W, parent: [V]?E) = todo

/// O(n + m): a spanning forest as chosen edges, by BFS.
pub fn spanning_forest<V, E, N>(next: N) -> [E]Bool = todo

/// O(n) build: the functional graph `v -> next[v]` (every vertex one out-arc).
pub struct Functional<V>:
    next: [V]V

impl<V> Functional<V>:
    /// O(n log n) build (cycles, tails, binary lifting).
    pub fn new(next: [V]V) -> Functional<V> = todo

    /// O(1): does `v` lie on a cycle?
    pub fn on_cycle(self, v: V) -> Bool = todo

    /// O(1): the first cycle vertex reached from `v` (`v` itself on a cycle): the
    /// root of the in-tree hanging off the cycle that holds `v`.
    pub fn root_of(self, v: V) -> V = todo

    /// O(1): steps from `v` to `root_of(v)`.
    pub fn depth(self, v: V) -> Int = todo

    /// O(1): the length of the cycle `v` eventually reaches.
    pub fn cycle_len(self, v: V) -> Int = todo

    /// O(log k): the vertex `k` steps after `v` (binary lifting). Never `none`,
    /// unlike `Rooted.ancestor`, since every vertex has an out-arc.
    pub fn after(self, v: V, k: Int) -> V = todo

    /// O(log n): the first vertex `u` on the walk from `v` with `pred(u)`, for a
    /// `pred` monotone along every walk, or `none` if no such vertex appears.
    pub fn jump_while(self, v: V, pred: V -> Bool) -> ?V = todo

    /// O(log n): the fewest steps from `a` to `b`, or `none` if `b` is never reached.
    pub fn dist(self, a: V, b: V) -> ?Int = todo

    /// O(n): every cycle once, each in walk order.
    pub fn cycles(self) -> List<List<V>> = todo

/// O(log n) amortized per edge: 2-edge-connectivity under edge insertions.
pub struct IncrementalBridges<V>:
    bridges: Int

impl<V> IncrementalBridges<V>:
    /// O(n): no edges yet.
    pub fn new() -> IncrementalBridges<V> = todo

    /// O(log n) amortized: add the edge `u - v`.
    pub fn add(mut self, u: V, v: V) = todo

    /// O(alpha): are `u` and `v` joined by two edge-disjoint paths?
    pub fn two_edge_same(mut self, u: V, v: V) -> Bool = todo

    /// O(alpha): are `u` and `v` connected?
    pub fn same(mut self, u: V, v: V) -> Bool = todo

/// O(1): one edge of a `Graph`: its id and its ends as given.
pub struct Edge<V, E>(e: E, u: V, v: V)

/// O(n) memory: a DFS tree of a stored graph, with the tree queries a DFS
/// argument needs. `Rooted` in `std.tree` has the full set.
pub struct DfsTree<V, E>:
    par: [V]?V

impl<V, E> DfsTree<V, E>:
    /// O(1): the parent of `v`, `none` at the root and off the tree.
    pub fn parent(self, v: V) -> ?V = todo

    /// O(1): arcs from the root to `v`.
    pub fn depth(self, v: V) -> Int = todo

    /// O(1): is `e` a tree edge (and not a parallel copy of one)?
    pub fn has(self, e: Edge<V, E>) -> Bool = todo

    /// O(1): the ends of `e` as (ancestor, descendant). Every edge of an
    /// undirected graph has such an orientation in its DFS tree.
    pub fn orient_down(self, e: Edge<V, E>) -> (u: V, v: V) = todo

    /// O(path length): the vertices from `a` to `b`, both included.
    pub fn path(self, a: V, b: V) -> List<V> = todo

    /// O(n log n), O(1) per item: the Euler tour, 2n - 1 visits where consecutive
    /// visits are joined by a tree edge; children are toured in ascending `by(c)`.
    pub fn euler_tour<K>(self, by: V -> K) -> yields V = todo

/// O(n + m): `Graph.directed` on a plain list of `(u, v)` arcs, as an
/// adjacency table indexed by vertex.
pub fn adjacency<V>(arcs: List<(u: V, v: V)>) -> Adj<V, V> = todo
