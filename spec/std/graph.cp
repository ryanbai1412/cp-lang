//! Graphs: one stored representation and the algorithms over "what comes after v".
//!
//! Vertices and edges are domains, so per-vertex data is `[V]T`, per-edge
//! data is `[E]T`, and mixing the two does not type-check. An algorithm
//! takes any `Next<V>`: a stored `Graph`, a `[V]List` of arcs, or any fn
//! from a vertex to a `Seq` of arcs. An arc is anything with `.to`; what
//! else it carries (`.w`, `.e`) decides which algorithms accept it:
//!
//!     bfs(g, from: [s])
//!     dijkstra(g, from: [(s, 0)])                          # arcs carry w
//!     dijkstra(g.weighted(a => 2 * a.w), from: [(s, 0)])   # weights from a fn
//!     bfs(p => p.adj4().filter(q => grid[q] != '#'), from: [start])
//!     bridges(g)                                           # arcs carry e
//!
//! Customizing a graph is a lambda over `g.out(u)`; nothing is rebuilt.
//! Unreachable is `None`. Whole-graph algorithms need `V: Domain`.
use std.heap.{MinHeap}
use std.dsu.{Dsu}
use std.seq.{reverse}
use std.deque.{Deque}

/// O(deg u) per walk of `out(u)`.
/// What every graph algorithm walks: the arcs out of `u`. Implemented by
/// `Graph`, by any `[V]S` with `S: Seq` of arcs, and by any fn `V -> Seq`.
pub trait Next<V>:
    type Arc: Target<V>
    /// The arcs out of `u`, lazily. Every graph algorithm reads the graph only through this.
    ///
    /// cost: O(1) per arc yielded
    /// order: the implementation's arc order (input order for `Graph`, list order for `[V]List`)
    /// traps: never
    /// example: `domain V = 0..3; let g: [V]List<V> = [[2, 1], [], []]; g.out(0).collect() == [2, 1]`
    fn out(self, u: V) -> yields Self.Arc = todo

/// O(1). Where an arc leads. A plain vertex is its own target, and a record
/// with a `to: V` field is one (a field is a `@get`).
pub trait Target<V>:
    /// The vertex an arc leads to. A plain vertex is its own target; a record's `to` field is
    /// its target.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..3; V(2).to == 2`
    /// example: `domain V = 0..3; let a: (to: V, w: Int) = (to: 1, w: 7); a.to == 1`
    @get fn to(self) -> V = self

/// O(1). The weight of an arc: a record with a `w` field, or `.weighted(f)`.
pub trait Weighted<W>:
    /// The weight of an arc: its `w` field, or `f(a)` through `g.weighted(f)`.
    ///
    /// cost: O(1) plus the cost of `f` under `weighted`
    /// traps: never
    /// example: `domain V = 0..3; let a: (to: V, w: Int) = (to: 1, w: 7); a.w == 7`
    @get fn w(self) -> W = todo

/// O(1). The edge an arc belongs to; both arcs of an undirected edge share it.
pub trait Edged<E>:
    /// The edge an arc belongs to. Both arcs of an undirected edge carry the same `e`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..2; domain E = 0..1; let es: [E](u: V, v: V) = [(u: 0, v: 1)]; let g = Graph.undirected(es); g.out(0).collect()[0].e == g.out(1).collect()[0].e`
    @get fn e(self) -> E = todo

/// A graph whose arcs carry weights: shortest paths, MST over arcs, DAG folds.
pub trait WeightedNext<V, W> = Next<V> where Self.Arc: Weighted<W>

/// A graph whose arcs carry edge ids: bridges, two-edge components, edge blocks.
/// These skip the arc back along the same edge, not to the same vertex, so
/// parallel edges are right.
pub trait EdgedNext<V, E> = Next<V> where Self.Arc: Edged<E>

/// The arc a build stores by default: the target, the edge id, and every
/// field of the edge record except `u` and `v`. Every carried field must be
/// plain; a heap field is a compile error that points at `keep:`.
pub type EdgeArc<V, E, R> = (to: V, e: E, ..R without (u, v))

/// O(1) per arc read: `g.weighted(f)`, the arcs of `g` with `.w` read as
/// `f(a)` (or `f(u, a)`), `.to` and `.e` passed through. Nothing is stored.
pub struct Reweighted<G, F>:
    g: G
    f: F

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
/// A fixed graph in compressed sparse row form: n + 1 offsets and one arcs
/// list, the arcs of each vertex contiguous and in input order. Immutable; a
/// graph that grows is a `[V]List` of arcs, which is already a `Next`.
pub struct Graph<V: Domain, A: Target<V>>:
    start: [V.cuts]Int
    arcs: List<A>

    /// A graph with one arc `u -> v` per edge of `edges`, in compressed sparse rows.
    ///
    /// The arcs out of each vertex keep the order of their edges in `edges` (a stable counting
    /// sort), so DFS orders and first-found answers are deterministic. Each arc is
    /// `EdgeArc<V, E, R>`, the edge record with `u`, `v` replaced by `to`, `e`, unless `keep`
    /// maps it to something else; `keep` runs once per arc at build and what it returns is
    /// stored.
    ///
    /// cost: O(n + m) time and memory, plus `m` calls of `keep`
    /// empty: no edges gives every vertex no arcs; an empty `V` gives the empty graph
    /// traps: never
    /// example: `domain V = 0..3; domain E = 0..2; let es: [E](u: V, v: V) = [(u: 0, v: 2), (u: 0, v: 1)]; Graph.directed(es).out(0).collect() == [(to: 2, e: 0), (to: 1, e: 1)]`
    /// example: `domain V = 0..3; domain E = 0..2; let es: [E](u: V, v: V) = [(u: 0, v: 2), (u: 0, v: 1)]; Graph.directed(es, keep: a => (to: a.to)).out(0).collect() == [(to: 2), (to: 1)]`
    pub fn directed<E: Domain, R: (u: V, v: V, ..), B: Target<V>>(
        edges: [E]R,
        keep: ?(EdgeArc<V, E, R> -> B) = None,
    ) -> Graph<V, B> = todo

    /// A graph with two arcs per edge of `edges`, `u -> v` and `v -> u`, both carrying the
    /// same `e`. A loop `u -> u` gives two arcs at `u`, so it counts twice in `deg`.
    ///
    /// The arcs out of each vertex come in the order of their edges in `edges`; a loop's two
    /// arcs are adjacent. `keep` runs once per stored arc, so `2m` times.
    ///
    /// cost: O(n + m) time and memory, plus `2m` calls of `keep`
    /// empty: no edges gives every vertex no arcs
    /// traps: never
    /// example: `domain V = 0..3; domain E = 0..2; let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 1, v: 2)]; Graph.undirected(es).out(1).collect() == [(to: 0, e: 0), (to: 2, e: 1)]`
    /// example: `domain V = 0..1; domain E = 0..1; let es: [E](u: V, v: V) = [(u: 0, v: 0)]; Graph.undirected(es).deg(0) == 2`
    pub fn undirected<E: Domain, R: (u: V, v: V, ..), B: Target<V>>(
        edges: [E]R,
        keep: ?(EdgeArc<V, E, R> -> B) = None,
    ) -> Graph<V, B> = todo

    /// A graph from `(from, arc)` pairs: `arc` is stored as one of `from`'s arcs. The base form
    /// the other builders reduce to.
    ///
    /// cost: O(n + m)
    /// empty: no pairs gives every vertex no arcs
    /// order: each vertex's arcs in the order the pairs come
    /// traps: never
    /// example: `domain V = 0..2; let g = Graph.of([(from: V(1), arc: V(0)), (from: V(0), arc: V(1)), (from: V(1), arc: V(1))]); g.next(1).collect() == [0, 1]`
    pub fn of(arcs: Seq<(from: V, arc: A)>) -> Graph<V, A> = todo

    /// `g[u]`: the arcs out of `u` as an `Indexed` view, without copying.
    ///
    /// The view is a slice of the graph's arc list, so like every slice it keeps its source's
    /// keys: `start[u]..start[u + 1]`. Iterate it, or use `.first`, `.len` and `.to_list()`.
    ///
    /// cost: O(1)
    /// order: the arcs in input order
    /// traps: never
    /// example: `domain V = 0..3; domain E = 0..2; let es: [E](u: V, v: V) = [(u: 0, v: 2), (u: 0, v: 1)]; let g = Graph.directed(es); g[0].len == 2 and g[1].len == 0`
    pub fn at(self, u: V) -> lends Slice<A> = todo

    /// The arcs out of `u`, lazily: the `Next` implementation.
    ///
    /// cost: O(1) to make, O(1) per arc
    /// order: input order
    /// traps: never
    /// example: `domain V = 0..3; domain E = 0..2; let es: [E](u: V, v: V) = [(u: 0, v: 2), (u: 0, v: 1)]; Graph.directed(es).out(0).collect() == [(to: 2, e: 0), (to: 1, e: 1)]`
    pub fn out(self, u: V) -> yields A:
        unchecked:
            for i in start[u.to_idx]..start[u.to_idx + 1]:
                yield arcs[i]

    /// The weight matrix: `d[u, v]` is the least `.w` over the arcs `u -> v`, `None` without
    /// one. The diagonal holds the least loop weight, or `None`; `floyd_warshall` starts it at
    /// zero itself.
    ///
    /// cost: O(n^2 + m) time, O(n^2) memory
    /// ties: parallel arcs keep the least weight
    /// traps: never
    /// example: `domain V = 0..2; domain E = 0..3; let es: [E](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 5), (u: 0, v: 1, w: 3), (u: 1, v: 0, w: 4)]; let d = Graph.directed(es).matrix(); d[0, 1] == Some(3) and d[1, 0] == Some(4) and d[0, 0] == None`
    pub fn matrix<W: Ord>(self) -> [V, V]?W where A: Weighted<W> = todo

    /// The number of arcs out of `u`. An undirected loop counts twice.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..3; domain E = 0..2; let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 1, v: 2)]; Graph.undirected(es).deg(1) == 2`
    pub fn deg(self, u: V) -> Int = start[u.to_idx + 1] - start[u.to_idx]

    /// Where `u`'s arcs lead: `out(u)` mapped to `.to`, repeats kept.
    ///
    /// cost: O(1) per vertex yielded
    /// order: input order
    /// traps: never
    /// example: `domain V = 0..3; domain E = 0..2; let es: [E](u: V, v: V) = [(u: 0, v: 2), (u: 0, v: 1)]; Graph.directed(es).next(0).collect() == [2, 1]`
    pub fn next(self, u: V) -> yields V = todo

    /// The induced view on the vertices `keep` accepts: arcs from or to a rejected vertex are
    /// skipped. `from` also skips the arcs leaving a vertex it rejects, so that vertex can be
    /// reached but not left. Rejected vertices stay in `V` with no arcs. See the free `only`.
    ///
    /// cost: O(1) to make; `keep` and `from` run per arc walked
    /// traps: never
    /// example: `domain V = 0..4; domain E = 0..4; let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 0, v: 2), (u: 1, v: 3), (u: 2, v: 3)]; Graph.directed(es).only(keep: v => v != 1).deg(0) == 1`
    /// example: `domain V = 0..4; domain E = 0..4; let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 0, v: 2), (u: 1, v: 3), (u: 2, v: 3)]; let h = Graph.directed(es).only(from: v => v != 1); h.deg(0) == 2 and h.deg(1) == 0`
    pub fn only(self, keep: V -> Bool = v => true, from: V -> Bool = v => true) -> Only<V, Self> = todo

    /// The same graph with each arc's `.w` read as `f(a)`; `.to` and `.e` pass through, so it
    /// is a `WeightedNext` and stays `Edged`. On a weighted graph it reweights. Nothing is
    /// stored.
    ///
    /// cost: O(1) to make; `f` runs on every arc read
    /// traps: never
    /// example: `domain V = 0..3; domain E = 0..2; let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 1, v: 2)]; dijkstra(Graph.directed(es).weighted(a => 5), from: [(V(0), 0)])[2] == Some(10)`
    pub fn weighted<W>(self, f: A -> W) -> lends Reweighted<Graph<V, A>, A -> W> = todo

    /// A BFS spanning tree rooted at `V.first`, as a `Rooted`. Each vertex's parent is the
    /// vertex whose arc first reached it, scanning vertices in BFS order and arcs in input
    /// order. A forest is `spanning_forest`.
    ///
    /// cost: O(n + m)
    /// traps: when the graph is not connected from `V.first`, or `V` is empty
    /// example: `domain V = 0..3; domain E = 0..2; let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 1, v: 2)]; let t = Graph.undirected(es).spanning_tree(); t.parent(2) == Some(1) and t.parent(0) == None`
    pub fn spanning_tree(self) -> Rooted<V, _> = todo

    /// True when some arc `u -> v` exists. A scan of `u`'s arcs, not a lookup.
    ///
    /// cost: O(deg u)
    /// traps: never
    /// example: `domain V = 0..3; domain E = 0..2; let es: [E](u: V, v: V) = [(u: 0, v: 2), (u: 0, v: 1)]; let g = Graph.directed(es); g.has(0, 2) and not g.has(2, 0)`
    pub fn has(self, u: V, v: V) -> Bool = todo

    /// The targets of every vertex as plain lists, `lists()[u] == next(u).collect()`: the shape
    /// `std.flow` matchings take.
    ///
    /// cost: O(n + m)
    /// order: keyed by `V`; each list in input order, repeats kept
    /// traps: never
    /// example: `domain V = 0..3; domain E = 0..2; let es: [E](u: V, v: V) = [(u: 0, v: 2), (u: 0, v: 1)]; Graph.directed(es).lists()[0] == [2, 1]`
    pub fn lists(self) -> [V]List<V> = todo

    /// The fewest arcs between every ordered pair, by a BFS from each vertex; `0` on the
    /// diagonal. A graph with unreachable pairs wants `bfs`.
    ///
    /// cost: O(n (n + m)) time, O(n^2) memory
    /// order: keyed by `(from, to)`
    /// traps: when some vertex cannot reach another
    /// example: `domain V = 0..3; domain E = 0..2; let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 1, v: 2)]; let h = Graph.undirected(es).hops(); h[0, 2] == 2 and h[1, 1] == 0`
    pub fn hops(self) -> [V, V]Int = todo


    /// Every edge once, as `Edge(e, u, v)` with its ends as given to the build. Needs arcs that
    /// keep `e`; the graph keeps one source arc per edge.
    ///
    /// cost: O(1) to make, O(1) per edge
    /// order: increasing `e`
    /// traps: never
    /// example: `domain V = 0..3; domain E = 0..2; let es: [E](u: V, v: V) = [(u: 1, v: 0), (u: 0, v: 2)]; Graph.undirected(es).edges().collect() == [Edge(0, 1, 0), Edge(1, 0, 2)]`
    pub fn edges<E>(self) -> yields Edge<V, E> where A: Edged<E> = todo

    /// The DFS tree from `root` of the component holding it, as a `Rooted` tree and a mask of the
    /// edges it uses. The search takes arcs in input order and descends at once (recursive order,
    /// run without recursion). On an undirected graph every edge with `tree_edge[e] == false`
    /// then joins a vertex to one of its ancestors in `tree`, which is what bridges, cut vertices
    /// and back-edge DPs need. Vertices outside the component are not in `tree`.
    ///
    /// cost: O(n + m)
    /// order: children in the order the search reached them
    /// traps: never
    /// example: `domain V = 0..3; domain E = 0..3; let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 1, v: 2), (u: 2, v: 0)]; let (tree, tree_edge) = Graph.undirected(es).dfs_tree(0); tree.parent(2) == Some(1) and tree.depth(2) == 2 and tree_edge == [true, true, false]`
    pub fn dfs_tree<E>(self, root: V) -> (tree: Rooted<V, V>, tree_edge: [E]Bool) where A: Edged<E> = todo

    /// `peel(g, ready:, on_remove:)` as a method; see the free `peel`.
    ///
    /// cost: O(n + m) plus the calls of `ready` and `on_remove`
    /// order: removal order
    /// traps: never
    /// example: `domain V = 0..4; domain E = 0..4; let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 0, v: 2), (u: 1, v: 3), (u: 2, v: 3)]; Graph.directed(es).peel() == [0, 1, 2, 3]`
    pub fn peel(
        self, ready: ?(V -> Bool) = None, on_remove: ?((V, A) -> ()) = None,
    ) -> List<V> = todo

    /// Every arc `u -> a.to` reversed to `a.to -> u`, its other fields kept (`(..a, to: u)`).
    ///
    /// cost: O(n + m)
    /// order: each vertex's new arcs by original source in vertex order, then input order
    /// traps: never
    /// example: `domain V = 0..4; domain E = 0..4; let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 0, v: 2), (u: 1, v: 3), (u: 2, v: 3)]; Graph.directed(es).transpose().out(3).collect() == [(to: 1, e: 2), (to: 2, e: 3)]`
    pub fn transpose(self) -> Graph<V, A> where A: (to: V, ..) = todo

/// Kahn's peeling: repeatedly removes a vertex with no arcs left coming in, and returns the
/// vertices in removal order. What stays is every vertex on a cycle or reachable from one,
/// so the order covers all of `V` exactly when the graph is acyclic.
///
/// A FIFO queue starts with the vertices of in-degree zero in vertex order; removing `u`
/// walks its arcs in order and appends each target whose in-degree drops to zero. With
/// `ready`, a vertex is removable when `ready(v)` holds instead: the queue starts with the
/// ready vertices in vertex order, and after removing `u` each arc `a` calls
/// `on_remove(u, a)` and then appends `a.to` if it is ready and not yet queued. Each vertex
/// is removed at most once. For a vertexless closure graph, pass the domain: `peel::<V>(u =>
/// ...)`.
///
/// cost: O(n + m) plus the calls of `ready` and `on_remove`
/// empty: an empty `V` gives `[]`
/// ties: FIFO; the initial vertices in vertex order, later ones in the order their last arc is walked
/// order: removal order
/// traps: never
/// example: `domain V = 0..4; let g: [V]List<V> = [[1, 2], [3], [3], []]; peel(g) == [0, 1, 2, 3]`
/// example: `domain V = 0..3; let g: [V]List<V> = [[1], [2], [1]]; peel(g) == [0]`
pub fn peel<V, G: Next<V>>(
    g: G, ready: ?(V -> Bool) = None, on_remove: ?((V, G.Arc) -> ()) = None,
) -> List<V> = todo

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// A topological order (every arc goes forward in it), or `None` if there is a directed
/// cycle: a `peel` that removed every vertex. For a vertexless closure graph, pass the
/// domain: `topo::<V>(u => ...)`.
///
/// cost: O(n + m)
/// empty: an empty `V` gives `Some([])`
/// ties: those of `peel` (FIFO, in-degree-zero vertices in vertex order); `smallest_topo` gives the least order
/// order: topological
/// traps: never
/// example: `domain V = 0..3; let g: [V]List<V> = [[], [0], [0]]; topo(g) == Some([1, 2, 0])`
/// example: `domain V = 0..3; let g: [V]List<V> = [[1], [2], [0]]; topo(g) == None`
pub fn topo<V>(g: Next<V>) -> ?List<V> = todo

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// The strongly connected components, as a fresh domain `C` and each vertex's component.
/// `C` is a topological order of the condensation: every arc goes from a component to the
/// same or a later one.
///
/// Computed by Tarjan's algorithm without recursion, roots in vertex order and arcs in arc
/// order; components are numbered in reverse order of completion.
///
/// cost: O(n + m)
/// ties: components with no path between them come in reverse completion order of that search
/// order: `comp` is keyed by `V`
/// traps: never
/// example: `domain V = 0..3; let g: [V]List<V> = [[1], [0], [0]]; let (domain C, comp) = scc(g); C.count == 2 and comp[0] == comp[1] and comp[2] < comp[0]`
pub fn scc<V>(g: Next<V>) -> (domain C, comp: [V]C) = todo

/// `Graph.directed` on a plain list of `(u, v)` pairs: no edge domain, and each arc is the
/// target vertex itself.
///
/// cost: O(n + m)
/// empty: `[]` gives every vertex no arcs
/// order: each vertex's arcs in input order
/// traps: never
/// example: `domain V = 0..3; directed_of_pairs([(V(0), V(2)), (V(0), V(1))])[0] == [2, 1]`
pub fn directed_of_pairs<V>(edges: List<(V, V)>) -> [V]List<V>:
    var adj: [V]List<V> = []
    for (u, v) in edges: adj[u].push(v)
    adj

/// `Graph.undirected` on a plain list of `(u, v)` pairs: each pair gives arcs both ways,
/// each arc the target vertex itself.
///
/// cost: O(n + m)
/// empty: `[]` gives every vertex no arcs
/// order: each vertex's arcs in the order of their pairs
/// traps: never
/// example: `domain V = 0..3; undirected_of_pairs([(V(0), V(1)), (V(1), V(2))])[1] == [0, 2]`
pub fn undirected_of_pairs<V>(edges: List<(V, V)>) -> [V]List<V>:
    var adj: [V]List<V> = []
    for (u, v) in edges:
        adj[u].push(v)
        adj[v].push(u)
    adj

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// The connected components of an undirected graph, as a fresh domain `C` and each vertex's
/// component.
///
/// Components are numbered in order of their smallest vertex. Precisely: scanning `V` in
/// order, each vertex not yet labelled gets the next number and labels everything it
/// reaches through unlabelled vertices; on a directed graph that is what it computes.
///
/// cost: O(n + m)
/// order: `C` by smallest vertex; `comp` keyed by `V`
/// traps: never
/// example: `domain V = 0..4; let g: [V]List<V> = [[2], [], [0], []]; let (domain C, comp) = components(g); C.count == 3 and comp[0] == comp[2] and Int(comp[3]) == 2`
pub fn components<V>(g: Next<V>) -> (domain C, comp: [V]C) = todo

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// The arcs between components: `out[comp[u]]` gets `(to: comp[a.to])` for every arc `a`
/// of `u` that leaves its component. Arcs inside a component are dropped; repeats are kept.
///
/// cost: O(n + m)
/// empty: an empty `V` gives every component no arcs
/// order: keyed by `C`; each list by source vertex in vertex order, then arc order
/// traps: never
/// example: `domain V = 0..3; let g: [V]List<V> = [[1], [0], [0, 1]]; let (domain C, comp) = scc(g); let h = condense(g, comp); h[comp[2]] == [(to: comp[0]), (to: comp[0])] and h[comp[0]] == []`
pub fn condense<V, C>(g: Next<V>, comp: [V]C) -> [C]List<(to: C)> = todo

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// The two-edge-connected components (bridge blocks) of an undirected graph, as a fresh
/// domain `C` and each vertex's component. Arcs carry `e`, so parallel edges are never
/// bridges. Every edge between two components is a bridge, so `condense(g, comp)` is the
/// bridge tree.
///
/// cost: O(n + m), iterative Tarjan low-link
/// order: `C` by smallest vertex; `comp` keyed by `V`
/// traps: in checked builds, when the graph is not undirected (an arc without its reverse on the same `e`)
/// example: `domain V = 0..3; domain E = 0..3; let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 1, v: 2), (u: 2, v: 1)]; let (domain C, comp) = two_edge_components(Graph.undirected(es)); comp[1] == comp[2] and comp[0] != comp[1]`
pub fn two_edge_components<V, E>(g: EdgedNext<V, E>) -> (domain C, comp: [V]C) = todo

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// Which vertices are cut vertices: removing one (and its arcs) leaves more components than
/// before. Works per component, so disconnected graphs are fine; a root is a cut vertex
/// exactly when it has two or more DFS children. Skips the arc's own edge, not its source
/// vertex, so parallel edges are right.
///
/// cost: O(n + m), iterative
/// order: keyed by `V`
/// traps: in checked builds, when the graph is not undirected (an arc without its reverse on the same `e`)
/// example: `domain V = 0..3; domain E = 0..2; let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 1, v: 2)]; cut_vertices(Graph.undirected(es)) == [false, true, false]`
pub fn cut_vertices<V, E>(g: EdgedNext<V, E>) -> [V]Bool = todo

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// A 2-colouring of an undirected graph (no arc joins two vertices of one side), or `None`
/// when an odd cycle exists.
///
/// A BFS from each uncoloured vertex in vertex order gives it `false` and alternates along
/// arcs, so the smallest vertex of every component is `false`.
///
/// cost: O(n + m)
/// order: keyed by `V`
/// traps: never
/// example: `domain V = 0..3; let g: [V]List<V> = [[1], [0, 2], [1]]; two_color(g) == Some([false, true, false])`
/// example: `domain V = 0..3; let g: [V]List<V> = [[1, 2], [0, 2], [0, 1]]; two_color(g) == None`
pub fn two_color<V>(g: Next<V>) -> ?[V]Bool = todo

@test
fn csr_keeps_edge_ids_and_order():
    domain V = 1..=4
    domain E = 1..=4
    let edges = [E] e => [
        (u: V(1), v: V(2)),
        (u: V(1), v: V(3)),
        (u: V(3), v: V(2)),
        (u: V(4), v: V(1)),
    ][e.to_idx]
    let g = Graph.directed(edges)
    assert [a.to.label for a in g.out(V(1))] == [2, 3]
    assert [a.e.label for a in g.out(V(1))] == [1, 2]
    assert g.deg(V(2)) == 0 and g.deg(V(4)) == 1
    let t = g.transpose()
    assert [a.to.label for a in t.out(V(2))] == [1, 3]
    assert [a.e.label for a in t.out(V(2))] == [1, 3]
    let u = Graph.undirected(edges)
    assert [a.to.label for a in u.out(V(1))] == [2, 3, 4]
    assert sum(u.deg(v) for v in V) == 8

@test
fn bfs_over_a_stored_graph_and_over_a_lambda() = todo

@test
fn dijkstra_matches_floyd_warshall() = todo

@test
fn topo_orders_a_dag_and_rejects_a_cycle():
    domain V = 0..6
    let out = [V] v => [V(x) for x in [[1, 2], [3], [3], [4], [], [0]][v.to_idx]]
    let order = topo::<V>(v => out[v])!
    var pos: [V]Int = 0
    for (i, v) in [(i, order[i]) for i in 0..order.len]: pos[v] = i
    for u in V:
        for v in out[u]: assert pos[u] < pos[v]
    assert topo::<V>(v => [v.cyc(1)]) == None
    let stuck = [V] v => [V(x) for x in [[1, 2], [3], [3], [4], [3], [0]][v.to_idx]]
    assert topo::<V>(v => stuck[v]) == None

@test
fn peel_leaves_what_a_cycle_reaches():
    // 0 -> 1 -> 2 -> 3 -> 2 (a cycle) -> 4, and 5 -> 0
    domain V = 0..6
    let out = [V] v => [V(x) for x in [[1], [2], [3], [2, 4], [], [0]][v.to_idx]]
    assert peel::<V>(v => out[v]) == [V(5), V(0), V(1)]
    let dag = [V] v => [V(x) for x in [[1, 2], [3], [3], [4], [], [0]][v.to_idx]]
    assert peel::<V>(v => dag[v]) == topo::<V>(v => dag[v])!
    domain E = 0..6
    let edges = [E] e => [
        (u: V(0), v: V(1)),
        (u: V(1), v: V(2)),
        (u: V(2), v: V(3)),
        (u: V(3), v: V(2)),
        (u: V(3), v: V(4)),
        (u: V(5), v: V(0)),
    ][e.to_idx]
    assert peel(Graph.directed(edges)) == [V(5), V(0), V(1)]

@test
fn scc_agrees_with_reachability():
    domain V = 0..8
    for _ in 0..25:
        let out = [V] v => [V.nth(rand(0, 7)) for _ in 0..rand(0, 2)]
        var reach: [V, V]Bool = false
        for s in V:
            let d = bfs(v => out[v], from: [s])
            for t in V: reach[s, t] = d[t] != None
        let (C, comp) = scc::<V>(v => out[v])
        for a in V:
            for b in V:
                assert (comp[a] == comp[b]) == (reach[a, b] and reach[b, a])
                if reach[a, b]: assert comp[a] <= comp[b]
        let dag = condense(v => out[v], comp)
        assert topo::<C>(c => dag[c]) != None

@test
fn components_of_an_undirected_graph() = todo

fn random_edges<V, E>() -> [E](u: V, v: V):
    [E] e => (u: V.nth(rand(0, V.K - 1)), v: V.nth(rand(0, V.K - 1)))

/// Are `a` and `b` connected when edge `skip` and vertex `gone` are removed?
fn linked<V, E>(edges: [E](u: V, v: V), a: V, b: V, skip: ?E, gone: ?V) -> Bool:
    var d = Dsu<V>()
    for e in E:
        if e != skip and edges[e].u != gone and edges[e].v != gone: d.unite(edges[e].u, edges[e].v)
    d.same(a, b)

@test
fn bridges_and_cut_vertices_match_removing_them() = todo

@test
fn blocks_of_a_small_graph() = todo

@test
fn two_color_finds_a_colouring_or_an_odd_cycle() = todo

@test
fn vertexless_closure_graphs_with_turbofish() = todo

@test
fn a_plain_edge_list_needs_no_edge_domain():
    domain V = 1..=4
    let g = Graph.undirected([(V(1), V(2)), (V(2), V(3))])
    assert [[v.to_idx for v in g[u]] for u in V] == [[1], [0, 2], [1], []]
    let d = Graph.directed([(V(1), V(2), 5), (V(1), V(4), 7)])
    assert [(v.to_idx, w) for (v, w) in d[V(1)]] == [(1, 5), (3, 7)]
    assert d[V(2)].is_empty

/// O(n) memory.
/// What a search found: `p[v]` is the distance to `v` (`None` if unreachable),
/// `back[v]` is `(from, via)`, the vertex before `v` on one shortest path and the
/// arc or move that reached it (`None` at a source), and `order` the reached
/// vertices in settling order, so every vertex comes after its `from` and
/// distances never decrease. A search given `until: t` stops once `t` is settled:
/// distances in `order` are exact, and any other vertex holds `None` or an upper
/// bound. Cost: the witnesses take one slot per vertex and one write per improving
/// relaxation, paid even when only `p[v]` is read.
/// Storage is spelled like `@memo(array)` / `@memo(map)`: by default `dist` and `back`
/// are dense tables over `V`, which must be a bounded type (a domain, range, enum or a
/// tuple of them); `hashed: true` on the search makes them hash maps over the states
/// reached, for state spaces like `Int` values or unbounded tuples. An unbounded `V`
/// without it is a compile error that names `hashed: true`.
pub struct Paths<V, W, A>:
    dist: [V]?W
    back: [V]?(from: V, via: A)
    order: List<V>

    /// `p[v]`: the distance to `v`, or `None` if no source reaches it (or the search stopped
    /// at `until` first).
    ///
    /// cost: O(1) dense, O(1) expected `hashed`
    /// traps: never
    /// example: `domain V = 0..4; let g: [V]List<V> = [[1, 2], [3], [3], []]; let p = bfs(g, from: [V(0)]); p[3] == Some(2) and p[0] == Some(0)`
    pub fn get(self, v: V) -> ?W = dist[v]

    /// The reached vertices with their distances, in settling order: `for (v, d) in bfs(g,
    /// from: [s])`.
    ///
    /// cost: O(1) to make, O(1) per item
    /// order: settling order (the `order` list), so distances never decrease for `bfs`, `bfs01`, `dial`, `dijkstra` and `fixed_cost_paths`; for `bellman_ford` each vertex comes after its parent
    /// traps: never
    /// example: `domain V = 0..3; let g: [V]List<V> = [[1], [2], []]; bfs(g, from: [V(0)]).iter().collect() == [(v: 0, d: 0), (v: 1, d: 1), (v: 2, d: 2)]`
    pub fn iter(self) -> yields (v: V, d: W) = todo

    /// The arcs on some shortest path: for each reached `u`, the arcs `a` of `g.out(u)` with
    /// `p[a.to] == p[u] + w(a)`, where `w(a)` is `a.w` for `Weighted` arcs and `1` otherwise.
    /// Walking `order` over this DAG counts or lists shortest paths.
    ///
    /// cost: O(n + m)
    /// order: keyed by `V`; each list in arc order; unreached vertices get `[]`
    /// traps: never
    /// example: `domain V = 0..4; let g: [V]List<V> = [[1, 2], [3], [3], []]; let t = bfs(g, from: [V(0)]).tight(g); t[0] == [1, 2] and t[2] == [3] and t[3] == []`
    pub fn tight<>(self, g: Next<V>) -> [V]List<A> = todo

    /// Does some source reach `v`? `p[v] != None`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..3; let g: [V]List<V> = [[1], [], []]; let p = bfs(g, from: [V(0)]); p.reached(1) and not p.reached(2)`
    pub fn reached(self, v: V) -> Bool = dist[v] != None

    /// The vertices of one shortest path from a source to `t`, both included, or `None` if
    /// `t` is unreachable; `[t]` when `t` is a source. Built from the back pointers, each set
    /// by the arc that first gave its vertex the final distance.
    ///
    /// cost: O(path length)
    /// ties: the path through the arc that first reached each vertex at its final distance
    /// order: from the source to `t`
    /// traps: never
    /// example: `domain V = 0..4; let g: [V]List<V> = [[1, 2], [3], [3], []]; bfs(g, from: [V(0)]).path(3) == Some([0, 1, 3])`
    /// example: `domain V = 0..2; let g: [V]List<V> = [[], []]; let p = bfs(g, from: [V(0)]); p.path(0) == Some([0]) and p.path(1) == None`
    pub fn path(self, t: V) -> ?List<V>:
        if dist[t] == none: return None
        var out = [t]
        var at = t
        while let b = back[at]:
            out.push(b.from)
            at = b.from
        reverse(mut out)
        out

    /// The arcs of `path(t)`, in walking order: `[]` when `t` is a source, `None` if it is
    /// unreachable.
    ///
    /// cost: O(path length)
    /// ties: as `path`
    /// order: from the source to `t`
    /// traps: never
    /// example: `domain V = 0..4; let g: [V]List<V> = [[1, 2], [3], [3], []]; bfs(g, from: [V(0)]).arcs(3) == Some([1, 3])`
    pub fn arcs(self, t: V) -> ?List<A>:
        if dist[t] == none: return None
        var out = List<A>()
        var at = t
        while let b = back[at]:
            out.push(b.via)
            at = b.from
        reverse(mut out)
        out

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// Fewest arcs from any vertex of `from` to each vertex, by breadth-first search.
///
/// The sources start at distance 0 in the order given (repeats ignored). A vertex is
/// entered at distance `d` only when `enter(v, d)` holds, sources included, so "the cell
/// must still be safe when you get there" is a filter, not a second search; a rejected
/// vertex may still be entered later through another arc. `until: t` stops as soon as `t`
/// gets its distance; vertices not reached by then read `None`. `hashed: true` keeps the
/// tables in hash maps for unbounded `V`.
///
/// cost: O(n + m) dense; O(reached + arcs scanned) expected `hashed`
/// empty: an empty `from` reaches nothing
/// ties: FIFO; a vertex's back pointer is the first arc to discover it
/// traps: never
/// example: `domain V = 0..4; let g: [V]List<V> = [[1, 2], [3], [3], []]; bfs(g, from: [V(0)])[3] == Some(2)`
/// example: `domain V = 0..4; let g: [V]List<V> = [[1, 2], [3], [3], []]; bfs(g, from: [V(0)], enter: (v, d) => v != 1).path(3) == Some([0, 2, 3])`
/// example: `domain V = 0..4; let g: [V]List<V> = [[1, 2], [3], [3], []]; let p = bfs(g, from: [V(0)], until: V(1)); p[1] == Some(1) and p[2] == None`
pub fn bfs<V, G: Next<V>>(g: G, from: Seq<V>, until: ?V = None, enter: (V, Int) -> Bool = (v, d) => true, hashed: Bool = false) -> Paths<V, Int, G.Arc> = todo

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// Shortest distances when every weight is 0 or 1, from every vertex of `from` at distance
/// 0: a deque, with 0-arcs pushed to the front. `until: t` stops once `t` is settled
/// (popped with its final distance).
///
/// cost: O(n + m)
/// empty: an empty `from` reaches nothing
/// ties: a vertex's back pointer is the first arc to give it its final distance
/// traps: on an arc whose weight is not 0 or 1, when it is read
/// example: `domain V = 0..3; domain E = 0..3; let es: [E](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 1), (u: 0, v: 2, w: 0), (u: 2, v: 1, w: 0)]; let p = bfs01(Graph.directed(es), from: [V(0)]); p[1] == Some(0) and p.path(1) == Some([0, 2, 1])`
pub fn bfs01<V, G: WeightedNext<V, Int>>(g: G, from: Seq<V>, until: ?V = None, hashed: Bool = false) -> Paths<V, Int, G.Arc> = todo

/// Shortest distances for non-negative weights, from `(vertex, starting distance)` pairs (a
/// vertex given twice keeps the smaller start). A binary heap with lazy deletion; `until:
/// t` stops once `t` is settled.
///
/// cost: O((n + m) log m)
/// empty: an empty `from` reaches nothing
/// ties: a vertex's back pointer is the first arc to give it its final distance (only strict improvements replace it)
/// traps: on a negative weight, when it is read
/// example: `domain V = 0..3; domain E = 0..3; let es: [E](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 5), (u: 0, v: 2, w: 1), (u: 2, v: 1, w: 2)]; let p = dijkstra(Graph.directed(es), from: [(V(0), 0)]); p[1] == Some(3) and p.path(1) == Some([0, 2, 1])`
/// example: `domain V = 0..2; domain E = 0..1; let es: [E](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 4)]; dijkstra(Graph.directed(es), from: [(V(0), 10)])[1] == Some(14)`
pub fn dijkstra<V, W, G: WeightedNext<V, W>>(
    g: G,
    from: Seq<(V, W)>,
    until: ?V = None,
    hashed: Bool = false,
) -> Paths<V, W, G.Arc> = todo

/// Shortest distances over a state space `S` whose moves have fixed non-negative costs,
/// from every state of `from` at distance 0. `step(mv, s)` is where move `mv` leads from
/// `s` (`None`: not allowed there), at cost `cost(mv)`.
///
/// A cursor per move into `order` replaces the heap: settled states come in non-decreasing
/// distance, so each move's first settled state whose step is still unsettled is its
/// cheapest candidate. `until: t` stops once `t` is settled, and only settled states get a
/// distance. The back pointer is `(from: s, via: mv)`.
///
/// cost: O(|S| |moves|) time, O(|S|) memory
/// empty: an empty `from` reaches nothing; no moves settles only the sources
/// ties: equal candidates go to the earlier move in `moves`
/// traps: on a negative cost
/// example: `domain S = 0..5; let p = fixed_cost_paths([S(0)], [1, 2], cost: m => if m == 1: 1 else: 3, step: (m, s) => if Int(s) + m < 5: Some(S(Int(s) + m)) else: None); p[4] == Some(4) and p[2] == Some(2)`
pub fn fixed_cost_paths<S, M, W>(
    from: List<S>,
    moves: List<M>,
    cost: M -> W,
    step: (M, S) -> ?S,
    until: ?S = None,
) -> Paths<S, W, M>:
    let price = [cost(mv) for mv in moves]
    for p in price: assert p >= W.zero
    var dist: [S]?W = None
    var back: [S]?(from: S, via: M) = None
    var order = List<S>()
    for s in from:
        if dist[s] == none:
            dist[s] = W.zero
            order.push(s)
            if until == s: return Paths(dist, back, order)
    var at = List.filled(moves.len, 0)
    while true:
        var best: ?(d: W, j: Int, t: S) = None
        for j in 0..moves.len:
            while at[j] < order.len:
                let s = order[at[j]]
                if let t = step(moves[j], s) and dist[t] == none:
                    let d = dist[s]! + price[j]
                    if let b = best:
                        if d < b.d: best = (d:, j:, t:)
                    else:
                        best = (d:, j:, t:)
                    break
                at[j] += 1
        if best == none: break
        let (d, j, t) = best!
        dist[t] = d
        back[t] = (from: order[at[j]], via: moves[j])
        order.push(t)
        at[j] += 1
        if until == t: break
    Paths(dist, back, order)

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// Shortest distances with any weights, from `(vertex, starting distance)` pairs, or `None`
/// when a source reaches a negative cycle (cycles no source reaches are ignored).
///
/// Rounds relax every arc of every reached vertex, vertices in vertex order and arcs in
/// arc order, until a round changes nothing; a change in round `n` means a negative cycle.
/// `order` lists each reached vertex after its parent.
///
/// cost: O(n m)
/// empty: an empty `from` gives `Some` of a search that reaches nothing
/// ties: a vertex's back pointer is the arc of its last strict improvement
/// traps: never
/// example: `domain V = 0..3; domain E = 0..3; let es: [E](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 4), (u: 0, v: 2, w: 5), (u: 2, v: 1, w: -3)]; bellman_ford(Graph.directed(es), from: [(V(0), 0)])![1] == Some(2)`
/// example: `domain V = 0..2; domain E = 0..2; let es: [E](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 1), (u: 1, v: 0, w: -2)]; bellman_ford(Graph.directed(es), from: [(V(0), 0)]) == None`
pub fn bellman_ford<V, W, G: WeightedNext<V, W>>(g: G, from: Seq<(V, W)>) -> ?Paths<V, W, G.Arc> = todo

// blocked: operation-value defaults typed as `Int` (GAPS.md ntt `convolve`)
/// All-pairs shortest distances, in place: `d[u, v]` starts as the direct weight (`None`:
/// no edge, usually from `g.matrix()`) and ends as the distance. Returns `false` if there
/// is a negative cycle, and `d` is then unspecified.
///
/// The diagonal starts at the semiring's one (0), so a vertex is at distance 0 from itself
/// unless a cheaper cycle exists. `over:` swaps the path algebra: `Semiring(max, min)`
/// gives widest (bottleneck) paths, `Semiring(min, max)` minimax paths, `Semiring(|, &)`
/// transitive closure; `None` is the semiring's zero (no path).
///
/// cost: O(n^3) time, no extra memory
/// traps: never
/// example: `domain V = 0..3; var d: [V, V]?Int = None; d[0, 1] = Some(4); d[1, 2] = Some(1); d[0, 2] = Some(7); floyd_warshall(mut d) and d[0, 2] == Some(5) and d[2, 0] == None and d[1, 1] == Some(0)`
/// example: `domain V = 0..2; var d: [V, V]?Int = None; d[0, 1] = Some(1); d[1, 0] = Some(-3); not floyd_warshall(mut d)`
pub fn floyd_warshall<V, W, R>(mut d: [V, V]?W, over: R = Semiring(min, +)) -> Bool = todo

// blocked: operation-value defaults typed as `Int` (GAPS.md ntt `convolve`)
/// The value form of `floyd_warshall`, as `sorted` is to `sort`: a new matrix closed under
/// `over:`, or `None` on a negative cycle. A matrix is already closed when
/// `d.closure(over: r) == Some(d)`.
///
/// cost: O(n^3) time, O(n^2) memory
/// traps: never
/// example: `domain V = 0..2; var d: [V, V]?Int = None; d[0, 1] = Some(2); let c = d.closure()!; c[0, 1] == Some(2) and c[0, 0] == Some(0) and c[1, 0] == None`
pub fn closure<V, W, R>(self: [V, V]?W, over: R = Semiring(min, +)) -> ?[V, V]?W = todo

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// One directed cycle, as its vertices in walking order (each is followed by the next, and
/// the last by the first), or `None` if the graph is acyclic.
///
/// A DFS without recursion, roots in vertex order and arcs in arc order, stops at the first
/// arc into a vertex still on the DFS path; the cycle is the path from that vertex on. A
/// loop gives a one-vertex cycle.
///
/// cost: O(n + m)
/// ties: the first cycle that DFS closes
/// order: walking order, starting at the vertex the closing arc enters
/// traps: never
/// example: `domain V = 0..4; let g: [V]List<V> = [[1], [2], [3], [1]]; find_cycle(g) == Some([1, 2, 3])`
/// example: `domain V = 0..3; let g: [V]List<V> = [[1, 2], [2], []]; find_cycle(g) == None`
pub fn find_cycle<V>(g: Next<V>) -> ?List<V> = todo

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// Which edges are bridges: removing one disconnects its ends. Arcs carry `e` (a
/// `Graph.undirected`), so a parallel edge is not mistaken for the way back and is never a
/// bridge. A loop is never a bridge.
///
/// cost: O(n + m)
/// order: keyed by `E`
/// traps: in checked builds, when the graph is not undirected (an arc without its reverse on the same `e`)
/// example: `domain V = 0..3; domain E = 0..3; let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 1, v: 2), (u: 2, v: 1)]; bridges(Graph.undirected(es)) == [true, false, false]`
pub fn bridges<V, E>(g: EdgedNext<V, E>) -> [E]Bool = todo

/// O(m) memory.
/// A walk: where it starts, then each arc taken (`to` is where the arc leads, `e` the
/// edge it uses), the same arcs a `Graph` yields.
pub struct Walk<V, E>:
    start: V
    arcs: List<(to: V, e: E)>

    /// The vertices visited, `start` first, then each arc's `to`: `arcs.len + 1` of them.
    ///
    /// cost: O(1) to make, O(1) per vertex
    /// order: walking order
    /// traps: never
    /// example: `domain V = 0..3; domain E = 0..2; let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 1, v: 2)]; euler_walk(es, directed: false)!.vertices.collect() == [0, 1, 2]`
    @get pub fn vertices(self) -> yields V = todo

/// A walk that uses every edge exactly once (Hierholzer's algorithm), or `None` if there is
/// None. `directed` says whether `(u, v)` may only be walked from `u` to `v`.
///
/// A closed walk starts at the smallest vertex with an edge; an open one at the vertex with
/// an extra way out (directed) or the smaller odd vertex. The walk is the one Hierholzer's
/// algorithm builds without recursion, taking each vertex's unused arcs in edge order
/// (the arcs of `Graph.directed` / `Graph.undirected` on `edges`).
///
/// cost: O(n + m)
/// empty: no edges gives `Some` walk at `V.first` with no arcs
/// traps: when `edges` is empty and `V` is empty
/// example: `domain V = 0..3; domain E = 0..3; let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 1, v: 2), (u: 2, v: 0)]; euler_walk(es, directed: false)!.vertices.collect() == [0, 1, 2, 0]`
/// example: `domain V = 0..3; domain E = 0..2; let es: [E](u: V, v: V) = [(u: 2, v: 1), (u: 1, v: 0)]; euler_walk(es, directed: false)!.vertices.collect() == [0, 1, 2]`
/// example: `domain V = 0..3; domain E = 0..2; let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 0, v: 2)]; euler_walk(es, directed: true) == None`
pub fn euler_walk<V, E>(edges: [E](u: V, v: V), directed: Bool) -> ?Walk<V, E>:
    let g = if directed: Graph.directed(edges) else: Graph.undirected(edges)
    var surplus: [V]Int = 0
    for e in E:
        surplus[edges[e].u] += 1
        if directed: surplus[edges[e].v] -= 1
        else: surplus[edges[e].v] += 1
    var start: ?V = None
    var opens = 0
    for v in V:
        let extra = if directed: surplus[v] else: surplus[v] % 2
        if extra > 1 or extra < -1: return None
        if extra == 1:
            opens += 1
            if start == none: start = v
    if opens > (if directed: 1 else: 2): return None
    if start == none: start = first(v for v in V if g.deg(v) > 0)
    if start == none: return Walk(V.first, List<(to: V, e: E)>())
    let out = [V] v => [a for a in g.out(v)]
    var at: [V]Int = 0
    var used: [E]Bool = false
    var stack: List<(v: V, came: ?(to: V, e: E))> = [(v: start!, came: None)]
    var arcs = List<(to: V, e: E)>()
    while not stack.is_empty:
        let v = stack.last!.v
        while at[v] < out[v].len and used[out[v][at[v]].e]: at[v] += 1
        if at[v] < out[v].len:
            let a = out[v][at[v]]
            used[a.e] = true
            stack.push((v: a.to, came: a))
        else:
            if let a = stack.pop()!.came: arcs.push(a)
    if arcs.len != E.K: return None
    reverse(mut arcs)
    Walk(start!, arcs)

/// O(1).
/// A variable or its negation: `Lit(x, true)` is "x holds".
pub struct Lit<X>(x: X, value: Bool):
    /// The opposite literal: `Lit(x, v).flip == Lit(x, not v)`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain X = 0..2; Lit(X(1), true).flip == Lit(X(1), false)`
    @get pub fn flip(self) -> Lit<X> = todo

/// A conjunction of two-literal clauses over the variables `X`. Every clause is O(1)
/// to add; `solve` is O(|X| + clauses).
pub struct TwoSat<X>:
    /// Implications between literal nodes: node `2 * i` is "variable i is false",
    /// `2 * i + 1` "is true"; variables past `X.K` are helpers of `at_most_one`.
    imp: List<List<Int>>

    /// An instance with no clauses over the variables of `X`; every assignment satisfies it.
    ///
    /// cost: O(|X|)
    /// traps: never
    /// example: `domain X = 0..2; TwoSat<X>().solve() != None`
    pub fn new() -> TwoSat<X>:
        TwoSat([[] for _ in 0..2 * X.K])

    /// Adds the clause "at least one of `a`, `b` holds".
    ///
    /// cost: O(1) amortized
    /// traps: never
    /// example: `domain X = 0..2; var s = TwoSat<X>(); s.add_or(Lit(X(0), true), Lit(X(1), true)); s.must(Lit(X(0), false)); s.solve() == Some([false, true])`
    pub fn add_or(mut self, a: Lit<X>, b: Lit<X>):
        self.clause(lit_node(a), lit_node(b))

    /// Adds the clause "`a` holds" (`add_or(a, a)`).
    ///
    /// cost: O(1) amortized
    /// traps: never
    /// example: `domain X = 0..1; var s = TwoSat<X>(); s.must(Lit(X(0), true)); s.must(Lit(X(0), false)); s.solve() == None`
    pub fn must(mut self, a: Lit<X>):
        self.add_or(a, a)

    /// Adds "if `a` holds then `b` holds" (`add_or(a.flip, b)`).
    ///
    /// cost: O(1) amortized
    /// traps: never
    /// example: `domain X = 0..2; var s = TwoSat<X>(); s.implies(Lit(X(0), true), Lit(X(1), true)); s.must(Lit(X(0), true)); s.solve() == Some([true, true])`
    pub fn implies(mut self, a: Lit<X>, b: Lit<X>):
        self.add_or(a.flip, b)

    /// Adds "at most one of `lits` holds". One helper variable per literal ("some literal so
    /// far holds") keeps it at O(k) clauses instead of O(k^2); helpers never appear in
    /// `solve`'s result.
    ///
    /// cost: O(k) amortized
    /// empty: `[]` adds nothing
    /// traps: never
    /// example: `domain X = 0..2; var s = TwoSat<X>(); s.at_most_one([Lit(X(0), true), Lit(X(1), true)]); s.must(Lit(X(0), true)); s.solve() == Some([true, false])`
    pub fn at_most_one(mut self, lits: List<Lit<X>>):
        var prev: ?Int = None
        for l in lits:
            let x = lit_node(l)
            let p = imp.len + 1
            imp.push([])
            imp.push([])
            self.clause(x ^ 1, p)
            if let q = prev:
                self.clause(q ^ 1, p)
                self.clause(q ^ 1, x ^ 1)
            prev = p

    /// Adds "exactly one of `a`, `b` holds" (two clauses).
    ///
    /// cost: O(1) amortized
    /// traps: never
    /// example: `domain X = 0..2; var s = TwoSat<X>(); s.add_xor(Lit(X(0), true), Lit(X(1), true)); s.must(Lit(X(0), true)); s.solve() == Some([true, false])`
    pub fn add_xor(mut self, a: Lit<X>, b: Lit<X>) = todo

    /// Adds "`a` and `b` hold together or not at all" (two clauses).
    ///
    /// cost: O(1) amortized
    /// traps: never
    /// example: `domain X = 0..2; var s = TwoSat<X>(); s.add_eq(Lit(X(0), true), Lit(X(1), true)); s.must(Lit(X(1), false)); s.solve() == Some([false, false])`
    pub fn add_eq(mut self, a: Lit<X>, b: Lit<X>) = todo

    /// An assignment satisfying every clause, or `None` if there is None.
    ///
    /// Runs `scc` on the implication graph (node `2i` is "variable `i` is false", `2i + 1` "is
    /// true"; clause `a or b` adds `not a -> b` then `not b -> a`, in the order added). A
    /// variable is true exactly when its true node's component comes after its false node's in
    /// `scc`'s topological order, so the assignment is a function of the clauses and their order.
    ///
    /// cost: O(|X| + clauses)
    /// order: keyed by `X`
    /// traps: never
    /// example: `domain X = 0..1; var s = TwoSat<X>(); s.implies(Lit(X(0), true), Lit(X(0), false)); s.solve() == Some([false])`
    pub fn solve(self) -> ?[X]Bool:
        domain N = 0..imp.len
        let (C, comp) = scc::<N>(u => [N(v) for v in imp[u.to_idx]])
        // components are in topological order: a literal is true when it comes after
        // its negation
        for i in 0..imp.len / 2:
            if comp[N(2 * i)] == comp[N(2 * i + 1)]: return None
        [X] x => comp[N(2 * x.to_idx + 1)] > comp[N(2 * x.to_idx)]

    fn clause(mut self, a: Int, b: Int):
        imp[a ^ 1].push(b)
        imp[b ^ 1].push(a)

fn lit_node<X>(l: Lit<X>) -> Int = 2 * l.x.to_idx + Int(l.value)

@test
fn paths_witness_their_distances() = todo

@test
fn fixed_cost_paths_match_dijkstra_on_the_same_moves() = todo

@test
fn until_stops_at_the_target_with_its_exact_distance() = todo

@test
fn bellman_ford_matches_floyd_warshall_and_finds_negative_cycles() = todo

@test
fn find_cycle_exists_exactly_when_topo_fails():
    domain V = 0..7
    for _ in 0..60:
        let out = [V] v => [V.nth(rand(0, 6)) for _ in 0..rand(0, 2) if rand(0, 2) > 0]
        let cyc = find_cycle::<V>(v => out[v])
        assert (cyc == None) == (topo::<V>(v => out[v]) != None)
        if let c = cyc:
            for i in 0..c.len: assert out[c[i]].contains(c[(i + 1) % c.len])
            assert sorted(c).len == c.len

@test
fn bridges_are_the_edges_whose_removal_disconnects():
    domain V = 0..6
    domain E = 0..7
    for _ in 0..40:
        let edges = [E] e => (u: V.nth(rand(0, 5)), v: V.nth(rand(0, 5)))
        let bridge = bridges(Graph.undirected(edges))
        for e in E:
            let rest = [V] v => [x.v for f in E if f != e for x in [edges[f]] if x.u == v] + [
                x.u for f in E if f != e for x in [edges[f]] if x.v == v
            ]
            let far = bfs::<V>(v => rest[v], from: [edges[e].u])
            assert bridge[e] == (not far.reached(edges[e].v))

@test
fn euler_walks_use_every_edge_once_and_exist_when_brute_force_finds_one():
    domain V = 0..4
    for _ in 0..200:
        let directed = rand(0, 1) == 1
        let k = rand(0, 6)
        domain E = 0..k
        let edges = [E] _ => (u: V.nth(rand(0, 3)), v: V.nth(rand(0, 3)))
        let got = euler_walk(edges, directed:)
        assert (got != None) == has_walk([(edges[e].u, edges[e].v) for e in E], directed)
        if let w = got:
            assert sorted([a.e for a in w.arcs]) == List(E)
            var at = w.start
            for a in w.arcs:
                let x = edges[a.e]
                assert (x.u == at and x.v == a.to) or (not directed and x.v == at and x.u == a.to)
                at = a.to
            assert w.vertices.len == w.arcs.len + 1
/// Does `walk` trace connected edges, starting at `start`?
fn follows<V>(edges: List<(V, V)>, walk: List<Int>, directed: Bool, start: V) -> Bool:
    var at = start
    for i in walk:
        let (u, v) = edges[i]
        if u == at: at = v
        elif not directed and v == at: at = u
        else: return false
    true

fn has_walk<V>(edges: List<(V, V)>, directed: Bool) -> Bool = todo

fn extend<V>(edges: List<(V, V)>, directed: Bool, mut used: List<Bool>, at: V, done: Int) -> Bool:
    if done == edges.len: return true
    for i in 0..edges.len:
        if used[i]: continue
        let (u, v) = edges[i]
        var to: ?V = None
        if u == at: to = v
        elif not directed and v == at: to = u
        if let w = to:
            used[i] = true
            if extend(edges, directed, mut used, w, done + 1): return true
            used[i] = false
    false

@test
fn at_most_one_forbids_every_pair_and_allows_each_single() = todo


// ---- Spec surface without implementations yet ----------------------------

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// A depth-first walk from each vertex of `from` in turn, yielding `Enter(v, via:)` when `v`
/// is discovered, `Again(from, via)` for an arc into an already discovered vertex, and
/// `Exit(v, via:)` when `v` is finished. `via` is the arc that discovered `v`, `None` for a
/// source; on `[V]List<V>` an arc is the vertex it reaches.
///
/// Arcs are taken in arc order and the walk descends at once (recursive order, run
/// without recursion). Discovered vertices are shared across sources, so a source already
/// discovered is skipped; pass `seen:` to share them with other walks (it is read and
/// updated in place). `step.prune()` right after `Enter(v)` skips `v`'s arcs (its `Exit`
/// still comes); `break` and `return` end the walk. `Again` events give custom low-link.
///
/// cost: O(n + m) total, O(1) per step
/// empty: an empty `from` yields nothing
/// order: `Enter` in preorder, `Exit` in postorder, `Again` when its arc is scanned, interleaved as the walk goes
/// traps: never
/// example: `domain V = 0..3; let g: [V]List<V> = [[1, 2], [], []]; dfs(g, from: [V(0)]).collect() == [Enter(0, None), Enter(1, Some(1)), Exit(1, Some(1)), Enter(2, Some(2)), Exit(2, Some(2)), Exit(0, None)]`
/// example: `domain V = 0..3; let g: [V]List<V> = [[1, 2], [2], []]; dfs(g, from: [V(0)]).collect() == [Enter(0, None), Enter(1, Some(1)), Enter(2, Some(2)), Exit(2, Some(2)), Exit(1, Some(1)), Again(0, 2), Exit(0, None)]`
pub fn dfs<V, G: Next<V>>(g: G, from: Seq<V>, seen: ?mut [V]Bool = None) -> yields Visit<V, G.Arc> = todo

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// Every vertex reachable from `from`, each once, sources included. `reachable(g, from:
/// [s]).count()`, `List(...)` or `.contains(v)` ask the usual questions; to avoid some
/// arcs, pass `g` filtered (`v => g.next(v).filter(a => ok(a))`).
///
/// cost: O(n + m) total, O(1) per step
/// empty: an empty `from` yields nothing
/// order: discovery order, the order of `dfs`'s `Enter` events
/// traps: never
/// example: `domain V = 0..4; let g: [V]List<V> = [[1, 2], [3], [3], []]; reachable(g, from: [V(0)]).collect() == [0, 1, 3, 2]`
/// example: `domain V = 0..3; let g: [V]List<V> = [[], [0], []]; reachable(g, from: [V(1), V(2)]).collect() == [1, 0, 2]`
pub fn reachable<V>(g: Next<V>, from: Seq<V>) -> yields V = todo

/// O(1): one event of `dfs`, the same shape as `Rooted.tour` yields: `via` is the arc that
/// discovered `v` (`None` for a source); `Again(from, via)` is an arc into a discovered vertex.
pub enum Visit<V, A>: Enter(v: V, via: ?A), Exit(v: V, via: ?A), Again(from: V, via: A)

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// Shortest distances for integer weights in `0..=max_w` with a bucket queue (Dial), from
/// every vertex of `from` at distance 0. Same result shape as `dijkstra`; `until: t` stops
/// once `t` is settled.
///
/// `max_w + 1` buckets are reused cyclically, so memory is O(n + max_w) and the scan over
/// distances is O(d_max), the largest distance reached (at most `(n - 1) * max_w`).
///
/// cost: O(n + m + max_w + d_max)
/// empty: an empty `from` reaches nothing
/// ties: FIFO within a bucket; a vertex's back pointer is the first arc to give it its final distance
/// traps: on a weight outside `0..=max_w`, when it is read
/// example: `domain V = 0..3; domain E = 0..3; let es: [E](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 5), (u: 0, v: 2, w: 1), (u: 2, v: 1, w: 2)]; dial(Graph.directed(es), from: [V(0)], max_w: 5)[1] == Some(3)`
pub fn dial<V, G: WeightedNext<V, Int>>(g: G, from: Seq<V>, max_w: Int, until: ?V = None) -> Paths<V, Int, G.Arc> = todo

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// All-pairs shortest distances with negative arcs: one Bellman-Ford from a virtual source
/// joined to every vertex, then a Dijkstra per vertex on the reweighted arcs. `d[u, v]` is
/// the distance, `None` when unreachable, `0` on the diagonal. `None` if any negative cycle
/// exists, reachable or not.
///
/// cost: O(n m log m)
/// traps: never
/// example: `domain V = 0..3; domain E = 0..3; let es: [E](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 4), (u: 0, v: 2, w: 5), (u: 2, v: 1, w: -3)]; let d = johnson(Graph.directed(es))!; d[0, 1] == Some(2) and d[1, 0] == None and d[1, 1] == Some(0)`
/// example: `domain V = 0..2; domain E = 0..2; let es: [E](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 1), (u: 1, v: 0, w: -2)]; johnson(Graph.directed(es)) == None`
pub fn johnson<V, W>(g: WeightedNext<V, W>) -> ?[V, V]?W = todo

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// The lengths of the `k` shortest walks from any vertex of `from` to `to` (Eppstein).
/// Walks may repeat vertices and arcs; the empty walk counts when `to` is a source. Fewer
/// than `k` when fewer walks exist.
///
/// cost: O((n + m) log n + k log k)
/// empty: an empty `from` or `k <= 0` gives `[]`
/// order: ascending, repeats kept
/// traps: on a negative weight
/// example: `domain V = 0..2; domain E = 0..2; let es: [E](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 1), (u: 1, v: 0, w: 1)]; k_shortest(Graph.directed(es), from: [V(0)], to: V(1), k: 3) == [1, 3, 5]`
/// example: `domain V = 0..2; domain E = 0..1; let es: [E](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 7)]; k_shortest(Graph.directed(es), from: [V(0)], to: V(1), k: 3) == [7]`
pub fn k_shortest<V, W>(g: WeightedNext<V, W>, from: Seq<V>, to: V, k: Int) -> List<W> = todo

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// Some cycle of negative total weight, as its arcs in walking order, or `None`.
///
/// Bellman-Ford from every vertex at distance 0 (vertices in vertex order, arcs in arc
/// order); the first vertex relaxed in round `n`, followed back `n` times, lands on a
/// cycle, which is returned starting with the arc leaving that vertex.
///
/// cost: O(n m)
/// ties: the cycle that search lands on
/// order: walking order
/// traps: never
/// example: `domain V = 0..3; domain E = 0..3; let es: [E](u: V, v: V, w: Int) = [(u: 2, v: 0, w: 1), (u: 0, v: 1, w: 1), (u: 1, v: 0, w: -2)]; negative_cycle(Graph.directed(es))!.len == 2`
/// example: `domain V = 0..2; domain E = 0..2; let es: [E](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 1), (u: 1, v: 0, w: -1)]; negative_cycle(Graph.directed(es)) == None`
pub fn negative_cycle<V, W, G: WeightedNext<V, W>>(g: G) -> ?List<G.Arc> = todo

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// The least mean weight over all directed cycles, as `(sum, len)` of a cycle achieving it
/// (Karp), or `None` on an acyclic graph. Among cycles of least mean the shortest is
/// reported, so `(sum, len)` is unique.
///
/// cost: O(n m)
/// ties: the fewest arcs among cycles of least mean
/// traps: never
/// example: `domain V = 0..2; domain E = 0..3; let es: [E](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 1), (u: 1, v: 0, w: 3), (u: 1, v: 1, w: 5)]; min_mean_cycle(Graph.directed(es)) == Some((sum: 4, len: 2))`
/// example: `domain V = 0..2; domain E = 0..3; let es: [E](u: V, v: V, w: Int) = [(u: 0, v: 0, w: 2), (u: 0, v: 1, w: 1), (u: 1, v: 0, w: 3)]; min_mean_cycle(Graph.directed(es)) == Some((sum: 2, len: 1))`
pub fn min_mean_cycle<V>(g: WeightedNext<V, Int>) -> ?(sum: Int, len: Int) = todo

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// The length of a shortest cycle (the girth), or `None` if there is None. With `directed:
/// false` the graph is read as undirected (every arc has its reverse) and a cycle counts
/// only if it uses at least three edges, so loops and an arc's way back are not cycles.
///
/// cost: O(n m), a BFS from every vertex
/// traps: never
/// example: `domain V = 0..3; let g: [V]List<V> = [[1], [2], [0]]; girth(g, directed: true) == Some(3)`
/// example: `domain V = 0..3; let g: [V]List<V> = [[1], [0, 2], [1]]; girth(g, directed: false) == None`
pub fn girth<V>(g: Next<V>, directed: Bool) -> ?Int = todo

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// The lexicographically smallest topological order (vertex order), by a min-heap instead
/// of `topo`'s queue; `None` if there is a cycle.
///
/// cost: O((n + m) log n)
/// order: topological, least first
/// traps: never
/// example: `domain V = 0..3; let g: [V]List<V> = [[], [0], []]; smallest_topo(g) == Some([1, 0, 2])`
/// example: `domain V = 0..2; let g: [V]List<V> = [[1], [0]]; smallest_topo(g) == None`
pub fn smallest_topo<V>(g: Next<V>) -> ?List<V> = todo

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// DP over a DAG in topological order. Each vertex starts at `init(v)`; then, for `u` in
/// `topo` order and each arc `a` of `u` in arc order, `val[a.to] = op(val[a.to],
/// step(val[u], a))`. `op: max` gives longest paths, `op: +` counts. `None` on a cycle.
///
/// cost: O(n + m)
/// order: keyed by `V`
/// traps: never
/// example: `domain V = 0..4; let g: [V]List<V> = [[1, 2], [3], [3], []]; dag_fold(g, init: v => 0, step: (x, a) => x + 1, op: max) == Some([0, 1, 1, 2])`
/// example: `domain V = 0..2; let g: [V]List<V> = [[1], [0]]; dag_fold(g, init: v => 0, step: (x, a) => x + 1, op: max) == None`
pub fn dag_fold<V, T, M, G: Next<V>>(g: G, init: V -> T, step: (T, G.Arc) -> T, op: M) -> ?[V]T = todo

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// The number of paths from a vertex of `from` to every `v` of a DAG, in `T` (a modular
/// type keeps it exact). A source has its empty path; a repeated source counts once.
/// `None` if the graph has a cycle anywhere.
///
/// cost: O(n + m)
/// empty: an empty `from` gives all zeros
/// order: keyed by `V`
/// traps: on overflow of `T`, as its arithmetic does
/// example: `domain V = 0..4; let g: [V]List<V> = [[1, 2], [3], [3], []]; count_paths::<Int>(g, from: [V(0)]) == Some([1, 1, 1, 2])`
pub fn count_paths<T, V>(g: Next<V>, from: Seq<V>) -> ?[V]T = todo

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// Reachability of every pair as bitset rows: `r[u].contains(v)` when some path of zero or
/// more arcs leads from `u` to `v`, so every `u` reaches itself. Computed through the
/// condensation in reverse topological order.
///
/// cost: O(n m / 64) time, O(n^2 / 64) memory
/// order: keyed by `V`
/// traps: never
/// example: `domain V = 0..3; let g: [V]List<V> = [[1], [2], []]; let r = closure(g); r[0].contains(2) and not r[2].contains(0) and r[1].contains(1)`
pub fn closure<V>(g: Next<V>) -> [V]Bitset<V> = todo

/// Offline incremental SCC: the arcs of `arcs` are added in order, and `t[e]` is the first
/// index `i >= e` such that after adding arcs `0..=i` the ends of `e` share a strongly
/// connected component, or `None` if they never do.
///
/// cost: O(m log m), divide and conquer over time
/// traps: never
/// example: `domain V = 0..3; domain E = 0..4; let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 1, v: 2), (u: 2, v: 0), (u: 2, v: 1)]; scc_times(es) == [Some(2), Some(2), Some(2), Some(3)]`
/// example: `domain V = 0..2; domain E = 0..1; let es: [E](u: V, v: V) = [(u: 0, v: 1)]; scc_times(es) == [None]`
pub fn scc_times<V, E>(arcs: [E](u: V, v: V)) -> [E]?Int = todo

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// The block-cut tree of an undirected simple graph: `B` numbers the blocks as the DFS (roots
/// in vertex order, arcs in arc order) closes them, deepest first, as `blocks` does, and `links` joins each block to the cut vertices it contains.
///
/// cost: O(n + m)
/// order: `links` by block, then the block's vertex order
/// traps: never
/// example: `domain V = 0..3; let g: [V]List<V> = [[1], [0, 2], [1]]; let (domain B, links) = block_cut_tree(g); B.count == 2 and links == [(b: B(0), v: V(1)), (b: B(1), v: V(1))]`
pub fn block_cut_tree<V>(g: Next<V>) -> (domain B, links: List<(b: B, v: V)>) = todo

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// The biconnected block of every edge: edges, not vertices, are partitioned, so a cut
/// vertex lies in several blocks and parallel edges share one. Blocks are numbered as the
/// DFS (roots in vertex order, arcs in arc order) closes them; each loop is a block of its
/// own, numbered after those in edge order.
///
/// cost: O(n + m)
/// order: `block` keyed by `E`
/// traps: in checked builds, when the graph is not undirected (an arc without its reverse on the same `e`)
/// example: `domain V = 0..4; domain E = 0..4; let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 1, v: 2), (u: 2, v: 0), (u: 2, v: 3)]; let (domain B, block) = blocks(Graph.undirected(es)); B.count == 2 and block[0] == block[1] and block[1] == block[2] and block[3] != block[0]`
pub fn blocks<V, E>(g: EdgedNext<V, E>) -> (domain B, block: [E]B) = todo

/// The complement of `missing` over a domain `V`: an arc `u -> v` for every `v != u` that
/// `missing.out(u)` does not reach. Its own `out(u)` is O(|V|), but `bfs`, `dfs`,
/// `reachable` and `components` over a `Complement` run in O(|V| + m) by keeping the set of
/// unvisited vertices and scanning only it, so "BFS in the complement graph" and its
/// components are the ordinary searches: `components(complement(g))`.
pub struct Complement<V, G>(missing: G):
    /// Every `v != u` that `missing.out(u)` does not reach.
    ///
    /// cost: O(|V| + deg u)
    /// order: ascending
    /// traps: never
    /// example: `domain V = 0..4; let g: [V]List<V> = [[1], [0], [], []]; complement(g).out(0).collect() == [2, 3]`
    pub fn out(self, u: V) -> yields V = todo

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// The `Complement` of `missing`, as a view; see there.
///
/// cost: O(1)
/// traps: never
/// example: `domain V = 0..4; let g: [V]List<V> = [[1], [0], [], []]; let (domain C, comp) = components(complement(g)); C.count == 1`
pub fn complement<V, G: Next<V>>(missing: G) -> Complement<V, G> = todo

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// An odd cycle witnessing that an undirected graph is not bipartite, or `None` when it is
/// (exactly when `two_color` is `None`).
///
/// The BFS of `two_color` stops at the first arc `u -> v` joining two vertices of one side;
/// the cycle runs from the BFS-tree LCA `x` of `u` and `v` down to `u`, then from `v` back
/// up to the child of `x`. A loop gives `[u]`.
///
/// cost: O(n + m)
/// order: walking order, starting at `x`
/// traps: never
/// example: `domain V = 0..3; let g: [V]List<V> = [[1, 2], [0, 2], [0, 1]]; odd_cycle(g) == Some([0, 1, 2])`
/// example: `domain V = 0..3; let g: [V]List<V> = [[1], [0, 2], [1]]; odd_cycle(g) == None`
pub fn odd_cycle<V>(g: Next<V>) -> ?List<V> = todo

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// The immediate dominator of every vertex reachable from `root` (Lengauer-Tarjan): the
/// last vertex other than `v` that every path from `root` to `v` passes. `None` for `root`
/// and unreachable vertices.
///
/// cost: O((n + m) alpha)
/// traps: never
/// example: `domain V = 0..4; let g: [V]List<V> = [[1, 2], [3], [3], []]; dominators(g, root: V(0)) == [None, Some(0), Some(0), Some(0)]`
/// example: `domain V = 0..3; let g: [V]List<V> = [[1], [2], []]; dominators(g, root: V(1)) == [None, None, Some(1)]`
pub fn dominators<V>(g: Next<V>, root: V) -> [V]?V = todo

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// Every triangle of an undirected simple graph once.
///
/// Each edge is oriented from lower to higher rank, rank being `(degree, vertex)`; for `u`
/// in vertex order, each out-neighbour `v` of `u` in arc order and each out-neighbour `w`
/// of `v` in arc order that is also one of `u`'s gives `(u, v, w)`, so the three come in
/// increasing rank.
///
/// cost: O(m sqrt m) total, O(1) per item
/// order: by `u` in vertex order, then `v` and `w` in arc order
/// traps: never
/// example: `domain V = 0..3; let g: [V]List<V> = [[1, 2], [0, 2], [0, 1]]; triangles(g).collect() == [(0, 1, 2)]`
/// example: `domain V = 0..4; let g: [V]List<V> = [[1, 2, 3], [0, 2, 3], [0, 1, 3], [0, 1, 2]]; triangles(g).count() == 4`
pub fn triangles<V>(g: Next<V>) -> yields (V, V, V) = todo

fn clique_rec<V>(adj: [V]Bitset<V>, mut r: List<V>, p: Bitset<V>, x: Bitset<V>, mut best: List<V>):
    if p.is_empty and x.is_empty:
        var c = [y for y in r]
        sort(mut c)
        if c.len > best.len or (c.len == best.len and c < best): best = c
        return
    if r.len + p.count < best.len: return
    let px = p.bitor(x)
    var pivot = px.first!
    for u in px.each():
        if p.bitand(adj[u]).count > p.bitand(adj[pivot]).count: pivot = u
    var pp = p.bitand(adj[pivot].complement())
    var xx = x.bitand(Bitset<V>.full())
    var rest = p.bitand(Bitset<V>.full())
    for v in pp.each():
        r.push(v)
        clique_rec(adj, mut r, rest.bitand(adj[v]), xx.bitand(adj[v]), mut best)
        r.pop()
        rest.remove(v)
        xx.add(v)

/// A maximum clique of the undirected graph given as bitset rows (Bron-Kerbosch with
/// pivoting); `n` up to about 64 in contests.
///
/// cost: O(3^(n/3)) worst case
/// empty: an empty `V` gives `[]`
/// ties: the lexicographically least of the maximum cliques, each read ascending
/// order: ascending
/// traps: never
/// example: `domain V = 0..4; let adj: [V]Bitset<V> = [Bitset.of([V(1), V(2)]), Bitset.of([V(0), V(2)]), Bitset.of([V(0), V(1), V(3)]), Bitset.of([V(2)])]; max_clique(adj) == [0, 1, 2]`
pub fn max_clique<V>(adj: [V]Bitset<V>) -> List<V>:
    var best = List<V>()
    var r = List<V>()
    clique_rec(adj, mut r, Bitset<V>.full(), Bitset<V>.empty(), mut best)
    best

/// The fewest colours such that no edge joins two vertices of one colour, by
/// inclusion-exclusion over independent sets.
///
/// cost: O(2^n n)
/// empty: an empty `V` gives `0`
/// traps: never
/// example: `domain V = 0..3; let adj: [V]Bitset<V> = [Bitset.of([V(1), V(2)]), Bitset.of([V(0), V(2)]), Bitset.of([V(0), V(1)])]; chromatic_number(adj) == 3`
/// example: `domain V = 0..3; let adj: [V]Bitset<V> = [Bitset.empty(), Bitset.empty(), Bitset.empty()]; chromatic_number(adj) == 1`
pub fn chromatic_number<V>(adj: [V]Bitset<V>) -> Int:
    let n = V.K
    if n == 0: return 0
    assert n <= 30, "`chromatic_number` needs at most 30 vertices"
    var nb = List.filled(n, 0)
    for v in V:
        for u in adj[v].each(): nb[v.to_idx] |= 1 << u.to_idx
    let full = (1 << n) - 1
    var ind = List.filled(1 << n, 0)
    ind[0] = 1
    for s in 1..(1 << n):
        let v = ctz(s)
        ind[s] = ind[s & (s - 1)] + ind[s & (s - 1) & ~nb[v]]
    let ps = [1_000_000_007, 998_244_353]
    for k in 1..n:
        for p in ps:
            var sum = 0
            for s in 0..(1 << n):
                var t = 1
                let b = ind[s] % p
                for _ in 0..k: t = t * b % p
                if (n - popcount(s)) % 2 == 1: sum = (sum - t + p) % p else: sum = (sum + t) % p
            if sum != 0: return k
    n

/// The de Bruijn sequence over `0..k` of order `n`: a cyclic sequence of length `k^n` in
/// which every length-`n` word appears once. Returns the lexicographically least one (the
/// Lyndon words of length dividing `n`, in order).
///
/// cost: O(k^n)
/// order: the cyclic sequence, cut so it starts with `n` zeros
/// traps: when `k < 1` or `n < 1`
/// example: `de_bruijn(2, 2) == [0, 0, 1, 1]`
/// example: `de_bruijn(2, 3) == [0, 0, 0, 1, 0, 1, 1, 1]`
/// example: `de_bruijn(3, 1) == [0, 1, 2]`
pub fn de_bruijn(k: Int, n: Int) -> List<Int>:
    assert k >= 1 and n >= 1, "`de_bruijn` needs `k >= 1` and `n >= 1`"
    var out = List<Int>()
    var w = [-1]
    while w.len > 0:
        w[w.len - 1] += 1
        let m = w.len
        if n % m == 0:
            for x in w: out.push(x)
        while w.len < n: w.push(w[w.len - m])
        while w.len > 0 and w[w.len - 1] == k - 1: w.pop()
    out

/// A minimum spanning forest by Kruskal, as its total weight and the chosen edges. Edges
/// are taken in `by:` order (`by` reads the edge record) with a stable sort, so `by: e => desc(e.w)` gives a maximum
/// one; loops are never taken.
///
/// cost: O(m log m)
/// empty: no edges give `(total: W.zero, take: [])`
/// ties: equal `by` keys go in edge key order
/// order: `take` keyed by `E`
/// traps: never
/// example: `domain V = 0..3; domain E = 0..3; let es: [E](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 1), (u: 1, v: 2, w: 2), (u: 0, v: 2, w: 2)]; kruskal(es) == (total: 3, take: [true, true, false])`
/// example: `domain V = 0..3; domain E = 0..3; let es: [E](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 1), (u: 1, v: 2, w: 2), (u: 0, v: 2, w: 2)]; kruskal(es, by: e => desc(e.w)).take == [false, true, true]`
pub fn kruskal<V, E, W, R: (u: V, v: V, w: W, ..), K: Ord>(edges: [E]R, by: R -> K = e => e.w) -> (total: W, take: [E]Bool) = todo

/// A minimum spanning tree of the dense graph `w` (Prim without a heap), rooted at
/// `V.first`; `None` if it is disconnected. `w[u, v]` is read with `u` in the tree.
///
/// cost: O(n^2)
/// empty: an empty `V` gives `Some((total: 0, parent: []))`
/// ties: the smallest vertex among equally cheap ones joins first; its parent is the tree vertex that first offered that cost
/// traps: never
/// example: `domain V = 0..3; var w: [V, V]?Int = None; w[0, 1] = Some(1); w[1, 0] = Some(1); w[1, 2] = Some(2); w[2, 1] = Some(2); w[0, 2] = Some(5); w[2, 0] = Some(5); prim(w) == Some((total: 3, parent: [None, Some(0), Some(1)]))`
pub fn prim<V, W>(w: [V, V]?W) -> ?(total: W, parent: [V]?V):
    var parent: [V]?V = None
    var best: [V]?W = None
    var done: [V]Bool = false
    var total = W.zero
    if V.K == 0: return Some((total:, parent:))
    var at = V.first!
    for _ in 0..V.K:
        done[at] = true
        for v in V:
            if done[v]: continue
            if let c = w[at, v]:
                if best[v] == None or c < best[v]!:
                    best[v] = Some(c)
                    parent[v] = Some(at)
        var next: ?V = None
        for v in V:
            if done[v] or best[v] == none: continue
            if next == None or best[v]! < best[next!]!: next = Some(v)
        if next == none: break
        at = next!
        total = total + best[at]!
    for v in V:
        if not done[v]: return None
    Some((total:, parent:))

/// A minimum spanning forest by Boruvka rounds, for implicit complete graphs (xor MST,
/// Manhattan MST). Each round passes `comp`, the component label of every vertex
/// (`0..c`, numbered by smallest vertex), and `cheapest(comp)` returns per label its
/// lightest edge leaving it, or `None`. Every returned edge joining two different
/// components is taken; rounds end when one component remains or no edge is taken.
///
/// cost: O(log n) rounds, each O(n) plus the call of `cheapest`
/// empty: an empty `V` gives total 0 and no edges
/// traps: when `cheapest` returns a list whose length is not the number of components
/// example: `domain V = 0..2; let r = boruvka(comp => [Some((u: V(0), v: V(1), w: 5)), Some((u: V(0), v: V(1), w: 5))]); r.total == 5 and r.edges == [(u: 0, v: 1)]`
pub fn boruvka<V, C, W>(cheapest: ([V]Int) -> List<?(u: V, v: V, w: W)>) -> (total: W, edges: List<(u: V, v: V)>) = todo

/// A minimum spanning arborescence rooted at `root` (Chu-Liu / Edmonds with a skew heap):
/// `parent[v]` is the edge entering `v`, `None` at `root`. `None` if some vertex is
/// unreachable from `root`. Loops are ignored.
///
/// cost: O(m log n)
/// ties: `total` is unique; when several arborescences reach it, which one `parent` describes is left open
/// traps: never
/// example: `domain V = 0..3; domain E = 0..4; let es: [E](u: V, v: V, w: Int) = [(u: 0, v: 1, w: 5), (u: 0, v: 2, w: 1), (u: 2, v: 1, w: 1), (u: 1, v: 2, w: 1)]; arborescence(es, root: V(0)) == Some((total: 2, parent: [None, Some(2), Some(1)]))`
/// example: `domain V = 0..2; domain E = 0..1; let es: [E](u: V, v: V, w: Int) = [(u: 1, v: 0, w: 1)]; arborescence(es, root: V(0)) == None`
pub fn arborescence<V, E, W>(edges: [E](u: V, v: V, w: W, ..), root: V) -> ?(total: W, parent: [V]?E) = todo

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// A spanning forest of an undirected graph as chosen edges: a BFS from each unvisited
/// vertex in vertex order takes the arc that discovers each vertex.
///
/// cost: O(n + m)
/// order: keyed by `E`
/// traps: never
/// example: `domain V = 0..3; domain E = 0..3; let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 1, v: 2), (u: 2, v: 0)]; spanning_forest(Graph.undirected(es)) == [true, false, true]`
pub fn spanning_forest<V, E>(g: EdgedNext<V, E>) -> [E]Bool = todo

/// O(n) build: the functional graph `v -> next[v]` (every vertex one out-arc).
pub struct Functional<V>:
    next: [V]V
    _nx: List<Int>
    _root: List<Int>
    _depth: List<Int>
    _cyc: List<Int>
    _pos: List<Int>
    _cycles: List<List<Int>>
    _up: List<List<Int>>

impl<V> Functional<V>:
    /// The functional graph `v -> next[v]` (every vertex has one out-arc), with its cycles,
    /// tails and binary-lifting tables built.
    ///
    /// cost: O(n log n)
    /// empty: an empty `V` gives an empty graph
    /// traps: never
    /// example: `domain V = 0..5; let nx: [V]V = [1, 2, 0, 0, 3]; Functional(nx).cycle_len(4) == 3`
    pub fn new(next: [V]V) -> Functional<V>:
        let n = V.count
        let nx = [next[V.at(i)].ord for i in 0..n]
        var root = List.filled(n, -1)
        var depth = List.filled(n, 0)
        var cyc = List.filled(n, -1)
        var pos = List.filled(n, 0)
        var cycles: List<List<Int>> = []
        var state = List.filled(n, 0)
        for s in 0..n:
            if state[s] != 0: continue
            var path: List<Int> = []
            var x = s
            while state[x] == 0:
                state[x] = 1
                path.push(x)
                x = nx[x]
            if state[x] == 1:
                var c: List<Int> = []
                var y = x
                while true:
                    cyc[y] = cycles.len
                    pos[y] = c.len
                    root[y] = y
                    state[y] = 2
                    c.push(y)
                    y = nx[y]
                    if y == x: break
                cycles.push(c)
            var k = path.len
            while k > 0:
                k -= 1
                let u = path[k]
                if state[u] == 2 and root[u] == u: continue
                root[u] = root[nx[u]]
                depth[u] = depth[nx[u]] + 1
                cyc[u] = cyc[nx[u]]
                state[u] = 2
        var up: List<List<Int>> = [nx]
        while (1 << up.len) <= n:
            let prev = up[up.len - 1]
            up.push([prev[prev[i]] for i in 0..n])
        Functional(next, nx, root, depth, cyc, pos, cycles, up)

    /// Does `v` lie on a cycle?
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..5; let nx: [V]V = [1, 2, 0, 0, 3]; let f = Functional(nx); f.on_cycle(1) and not f.on_cycle(3)`
    pub fn on_cycle(self, v: V) -> Bool = _depth[v.ord] == 0

    /// The first cycle vertex reached from `v` (`v` itself on a cycle): the root of the
    /// in-tree hanging off the cycle that holds `v`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..5; let nx: [V]V = [1, 2, 0, 0, 3]; let f = Functional(nx); f.root_of(4) == 0 and f.root_of(1) == 1`
    pub fn root_of(self, v: V) -> V = V.at(_root[v.ord])

    /// The steps from `v` to `root_of(v)`; `0` on a cycle.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..5; let nx: [V]V = [1, 2, 0, 0, 3]; let f = Functional(nx); f.depth(4) == 2 and f.depth(0) == 0`
    pub fn depth(self, v: V) -> Int = _depth[v.ord]

    /// The length of the cycle `v` eventually reaches.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..5; let nx: [V]V = [1, 2, 0, 0, 3]; Functional(nx).cycle_len(4) == 3`
    pub fn cycle_len(self, v: V) -> Int = _cycles[_cyc[v.ord]].len

    /// The vertex `k` steps after `v`, by binary lifting: `after(v, 0) == v`. Never `None`,
    /// unlike `Rooted.ancestor`, since every vertex has an out-arc.
    ///
    /// cost: O(log k)
    /// traps: when `k < 0`
    /// example: `domain V = 0..5; let nx: [V]V = [1, 2, 0, 0, 3]; let f = Functional(nx); f.after(4, 5) == 0 and f.after(4, 0) == 4`
    pub fn after(self, v: V, k: Int) -> V:
        assert k >= 0, "`after` needs `k >= 0`"
        let x = v.ord
        if k < _depth[x]: return V.at(self.lift(x, k))
        let r = _root[x]
        let c = _cycles[_cyc[r]]
        V.at(c[(_pos[r] + (k - _depth[x]) % c.len) % c.len])

    /// The first vertex `u` on the walk from `v`, `v` included, with `pred(u)`, or `None` if
    /// None appears. `pred` must be monotone along every walk (once true, true onward).
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `domain V = 0..5; let nx: [V]V = [1, 2, 0, 0, 3]; let f = Functional(nx); f.jump_while(4, u => f.on_cycle(u)) == Some(0)`
    /// example: `domain V = 0..5; let nx: [V]V = [1, 2, 0, 0, 3]; let f = Functional(nx); f.jump_while(0, u => u == 4) == None`
    pub fn jump_while(self, v: V, pred: V -> Bool) -> ?V:
        if pred(v): return Some(v)
        var x = v.ord
        var k = _up.len
        while k > 0:
            k -= 1
            let y = _up[k][x]
            if not pred(V.at(y)): x = y
        let c = V.at(_nx[x])
        if pred(c): Some(c) else: None

    /// The fewest steps from `a` to `b`, or `None` if the walk from `a` never reaches `b`;
    /// `dist(a, a) == Some(0)`.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `domain V = 0..5; let nx: [V]V = [1, 2, 0, 0, 3]; let f = Functional(nx); f.dist(4, 1) == Some(3) and f.dist(2, 0) == Some(1) and f.dist(1, 4) == None`
    pub fn dist(self, a: V, b: V) -> ?Int:
        let x = a.ord
        let y = b.ord
        if _cyc[x] != _cyc[y]: return None
        if _depth[y] == 0:
            let len = _cycles[_cyc[y]].len
            return Some(_depth[x] + (_pos[y] - _pos[_root[x]] + len) % len)
        if _depth[x] < _depth[y]: return None
        let d = _depth[x] - _depth[y]
        if self.lift(x, d) == y: Some(d) else: None

    fn lift(self, x: Int, k: Int) -> Int:
        var y = x
        for b in 0.._up.len:
            if (k >> b) & 1 == 1: y = _up[b][y]
        y

    /// Every cycle once, each in walk order from its smallest vertex.
    ///
    /// cost: O(n)
    /// order: cycles by smallest vertex
    /// traps: never
    /// example: `domain V = 0..5; let nx: [V]V = [1, 2, 0, 0, 3]; Functional(nx).cycles() == [[0, 1, 2]]`
    /// example: `domain V = 0..3; let nx: [V]V = [0, 2, 1]; Functional(nx).cycles() == [[0], [1, 2]]`
    pub fn cycles(self) -> List<List<V>>:
        var done = List.filled(_cycles.len, false)
        var out: List<List<V>> = []
        for i in 0.._nx.len:
            if _depth[i] != 0 or done[_cyc[i]]: continue
            done[_cyc[i]] = true
            var c: List<V> = [V.at(i)]
            var y = _nx[i]
            while y != i:
                c.push(V.at(y))
                y = _nx[y]
            out.push(c)
        out

/// O(log n) amortized per edge: 2-edge-connectivity under edge insertions.
pub struct IncrementalBridges<V>:
    bridges: Int

impl<V> IncrementalBridges<V>:
    /// No edges yet: every vertex alone.
    ///
    /// cost: O(n)
    /// traps: never
    /// example: `domain V = 0..2; var b = IncrementalBridges<V>(); not b.same(V(0), V(1))`
    pub fn new() -> IncrementalBridges<V> = todo

    /// Adds the undirected edge `u - v`. A loop changes nothing; a parallel edge makes its
    /// ends two-edge-connected.
    ///
    /// cost: O(log n) amortized
    /// traps: never
    /// example: `domain V = 0..2; var b = IncrementalBridges<V>(); b.add(V(0), V(1)); b.add(V(0), V(1)); b.two_edge_same(V(0), V(1))`
    pub fn add(mut self, u: V, v: V) = todo

    /// Are `u` and `v` joined by two edge-disjoint paths? True for `u == v`.
    ///
    /// cost: O(alpha)
    /// traps: never
    /// example: `domain V = 0..3; var b = IncrementalBridges<V>(); b.add(V(0), V(1)); b.add(V(1), V(2)); let before = b.two_edge_same(V(0), V(2)); b.add(V(2), V(0)); not before and b.two_edge_same(V(0), V(2))`
    pub fn two_edge_same(mut self, u: V, v: V) -> Bool = todo

    /// Are `u` and `v` connected?
    ///
    /// cost: O(alpha)
    /// traps: never
    /// example: `domain V = 0..3; var b = IncrementalBridges<V>(); b.add(V(0), V(1)); b.add(V(1), V(2)); b.same(V(0), V(2))`
    pub fn same(mut self, u: V, v: V) -> Bool = todo

/// O(1): one edge of a `Graph`: its id and its ends as given.
pub struct Edge<V, E>(e: E, u: V, v: V)

/// `directed_of_pairs` on named `(u, v)` arcs: neighbour lists indexed by vertex.
///
/// cost: O(n + m)
/// empty: `[]` gives every vertex no arcs
/// order: each vertex's arcs in input order
/// traps: never
/// example: `domain V = 0..3; adjacency([(u: V(0), v: V(2)), (u: V(0), v: V(1))])[0] == [2, 1]`
pub fn adjacency<V>(arcs: List<(u: V, v: V)>) -> [V]List<V>:
    var adj: [V]List<V> = []
    for a in arcs: adj[a.u].push(a.v)
    adj

// blocked: generic `V` of a `Next<V>` free fn is bound to `Int` (GAPS.md graph free fns)
/// The graph restricted to the vertices `keep` accepts: arcs from or to a dropped vertex
/// are skipped as `out` walks. `from` drops only the arcs leaving a vertex it rejects, so
/// the vertex can be reached but not left. A view; vertices keep their domain.
///
/// cost: O(1) to make; `keep` and `from` run per arc walked
/// traps: never
/// example: `domain V = 0..3; let g: [V]List<V> = [[1, 2], [2], []]; g.only(keep: v => v != 1).out(0).collect() == [2]`
/// example: `domain V = 0..3; let g: [V]List<V> = [[1, 2], [2], []]; reachable(g.only(from: v => v != 1), from: [V(0)]).collect() == [0, 1, 2]`
pub fn only<V, G: Next<V>>(self: G, keep: V -> Bool = v => true, from: V -> Bool = v => true) -> Only<V, G> = todo

/// `g.only(keep)`; see `only`. Implements every arc capability `G` has.
pub struct Only<V, G>(g: G, keep: V -> Bool, from: V -> Bool):
    /// The arcs out of `u` the view keeps: None when `u` is dropped, else those whose `to` is
    /// kept.
    ///
    /// cost: O(1) to make, O(1) per arc of `g` walked
    /// order: arc order of `g`
    /// traps: never
    /// example: `domain V = 0..3; let g: [V]List<V> = [[1, 2], [2], []]; g.only(keep: v => v != 1).out(0).collect() == [2]`
    pub fn out(self, u: V) -> yields G.Arc = todo

    /// The number of arcs out of `u` the view keeps.
    ///
    /// cost: O(deg u in `g`)
    /// traps: never
    /// example: `domain V = 0..3; let g: [V]List<V> = [[1, 2], [2], []]; g.only(keep: v => v != 1).deg(0) == 1`
    pub fn deg(self, u: V) -> Int = todo

    /// The kept arcs reversed, built as a `Graph` (record arcs as `(..a, to: u)`, a plain
    /// vertex arc as `u`).
    ///
    /// cost: O(n + m)
    /// traps: never
    /// example: `domain V = 0..3; let g: [V]List<V> = [[1, 2], [2], []]; g.only(keep: v => v != 1).transpose().next(2).collect() == [0]`
    pub fn transpose(self) -> Graph<V, G.Arc> = todo
