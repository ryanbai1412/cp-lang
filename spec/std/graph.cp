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
    fn out<V>(self, u: V) -> Iter<V> = self(u)

/// O(1).
/// What a neighbour list holds: an arc says where it leads with its `to` field, and a plain
/// vertex leads to itself.
pub trait Target:
    @get fn to(self) -> Self = self

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
    pub fn directed(edges: [E](u: V, v: V)) -> Graph<V, E> = Graph<V, E>.build(edges, false)

    /// O(n + m).
    /// Two arcs per edge, `u -> v` and `v -> u`, sharing the edge id.
    pub fn undirected(edges: [E](u: V, v: V)) -> Graph<V, E> = Graph<V, E>.build(edges, true)

    fn build(edges: [E](u: V, v: V), both: Bool) -> Graph<V, E>:
        var start: List<0..=2_000_000_000> = List.filled(V.K + 1, 0)
        unchecked:
            for e in E:
                start[edges[e].u.to_idx + 1] += 1
                if both: start[edges[e].v.to_idx + 1] += 1
            for i in 0..V.K: start[i + 1] += start[i]
            if E.K == 0: return Graph(start, [])
            var arcs = List.filled(start[V.K], (to: edges[E.first].v, e: E.first))
            var at: List<0..=2_000_000_000> = copy(start)
            for e in E:
                let (u, v) = (edges[e].u, edges[e].v)
                arcs[at[u.to_idx]] = (to: v, e: e)
                at[u.to_idx] += 1
                if both:
                    arcs[at[v.to_idx]] = (to: u, e: e)
                    at[v.to_idx] += 1
            return Graph(start, arcs)

    /// O(deg u) total, O(1) per arc.
    /// The arcs out of `u`, in edge order: what makes a `Graph` usable wherever `next` is
    /// expected. O(deg u).
    pub fn out(self, u: V) -> yields (to: V, e: E):
        unchecked:
            for i in start[u.to_idx]..start[u.to_idx + 1]:
                yield arcs[i]

    /// O(deg u): copies.
    /// `g[u]`: the arcs out of `u`, as a list.
    pub fn get(self, u: V) -> List<(to: V, e: E)> = [
        arcs[i] for i in start[u.to_idx]..start[u.to_idx + 1]
    ]

    /// The number of arcs out of `u`. O(1).
    pub fn deg(self, u: V) -> Int = start[u.to_idx + 1] - start[u.to_idx]

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

    /// O(n + m): `peel(g, ...)` as a method, with the same options.
    pub fn peel(
        self, ready: ?(V -> Bool) = none, on_remove: ?((V, (to: V, e: E)) -> ()) = none,
    ) -> List<V> = peel(self, ready: ready, on_remove: on_remove)

    /// The graph with every arc reversed. A rebuild, O(n + m).
    pub fn transpose(self) -> Graph<V, E>:
        var back: List<0..=2_000_000_000> = List.filled(V.K + 1, 0)
        unchecked:
            for a in arcs: back[a.to.to_idx + 1] += 1
            for i in 0..V.K: back[i + 1] += back[i]
            var out = copy(arcs)
            var at: List<0..=2_000_000_000> = copy(back)
            for u in V:
                for a in self.out(u):
                    out[at[a.to.to_idx]] = (to: u, e: a.e)
                    at[a.to.to_idx] += 1
            return Graph(back, out)

/// O(n + m).
/// Kahn's peeling: repeatedly remove a vertex with no arcs left coming in, and return the
/// vertices in the order they were removed. What stays is every vertex on a cycle or
/// reachable from one, so the order covers all of `V` exactly when the graph is acyclic.
/// For a vertexless closure graph, pass the vertex domain via turbofish: `peel::<V>(u => ...)`.
/// O(n + m).
pub fn peel<V, N, A>(
    next: N, ready: ?(V -> Bool) = none, on_remove: ?((V, A) -> ()) = none,
) -> List<V>:
    if ready != none or on_remove != none: return todo
    var indeg: [V]Int = 0
    for u in V:
        for a in next.out(u): indeg[a.to] += 1
    var order = [v for v in V if indeg[v] == 0]
    var head = 0
    while head < order.len:
        let u = order[head]
        head += 1
        for a in next.out(u):
            indeg[a.to] -= 1
            if indeg[a.to] == 0: order.push(a.to)
    order

/// O(n + m).
/// A topological order, or `none` if there is a cycle: a `peel` that removed every vertex.
/// O(n + m). For a vertexless closure graph, pass the vertex domain via turbofish:
/// `topo::<V>(u => ...)`.
pub fn topo<V, N>(next: N) -> ?List<V>:
    let order = peel::<V>(next)
    if order.len == V.K: order else: none

/// O(n + m).
/// Strongly connected components. `C` is a fresh domain, numbered so that
/// every arc goes from a smaller or equal component to a larger or equal
/// one (a topological order of the condensation). O(n + m), no recursion.
pub fn scc<V, N>(next: N) -> (domain C, comp: [V]C):
    let out = [V] v => [a.to for a in next.out(v)]
    var seen: [V]?Int = none  // discovery time
    var low: [V]Int = 0
    var done: [V]?Int = none  // component, in order of completion
    var stack = List<V>()
    var path = List<(V, Int)>()  // the dfs: a vertex and its next arc
    var (timer, k) = (0, 0)
    for root in V:
        if seen[root] != none: continue
        path.push((root, 0))
        while not path.is_empty:
            let (u, i) = path.last!
            if i == 0:
                seen[u] = timer
                low[u] = timer
                timer += 1
                stack.push(u)
            if i < out[u].len:
                path.last! = (u, i + 1)
                let v = out[u][i]
                if seen[v] == none: path.push((v, 0))
                elif done[v] == none: low[u] min= seen[v]!
            else:
                path.pop()
                if low[u] == seen[u]!:
                    while true:
                        let v = stack.pop()
                        done[v] = k
                        if v == u: break
                    k += 1
                if not path.is_empty: low[path.last!.0] min= low[u]
    domain C = 0..k
    (C: C, comp: [V] v => C(k - 1 - done[v]!))

/// O(n + m) memory and build.
/// Compressed sparse row adjacency table over `V` built from pairs or triples.
/// O(n + m) time; memory is n + 1 32-bit offsets and one arc per directed edge.
pub struct Adj<V, A>:
    start: List<0..=2_000_000_000>
    arcs: List<A>

    /// The arcs out of `u`. O(deg u).
    pub fn out(self, u: V) -> yields A:
        unchecked:
            for i in start[u.to_idx]..start[u.to_idx + 1]:
                yield arcs[i]

    /// `g[u]`: the arcs out of `u`, as a list. O(deg u).
    pub fn get(self, u: V) -> List<A> = [arcs[i] for i in start[u.to_idx]..start[u.to_idx + 1]]

    /// The number of arcs out of `u`. O(1).
    pub fn deg(self, u: V) -> Int = start[u.to_idx + 1] - start[u.to_idx]

    /// O(n + m): the table with every arc reversed.
    pub fn transpose(self) -> Adj<V, A> = todo

    /// O(deg u): is `v` among `u`'s neighbours? A scan, not a lookup.
    pub fn has(self, u: V, v: V) -> Bool = todo

/// O(n + m).
/// `Graph.directed(pairs)` on a plain list of `(u, v)`: CSR adjacency of each vertex.
/// No edge domain, and `for v in g[u]` yields the vertex itself. O(n + m).
pub fn directed_of_pairs<V>(edges: List<(V, V)>) -> Adj<V, V>:
    var start: List<0..=2_000_000_000> = List.filled(V.K + 1, 0)
    unchecked:
        for (u, _) in edges: start[u.to_idx + 1] += 1
        for i in 0..V.K: start[i + 1] += start[i]
        if edges.is_empty: return Adj(start, [])
        var arcs = List.filled(start[V.K], edges[0].1)
        var at: List<0..=2_000_000_000> = copy(start)
        for (u, v) in edges:
            arcs[at[u.to_idx]] = v
            at[u.to_idx] += 1
        return Adj(start, arcs)

/// `Graph.undirected(pairs)` on a plain list of `(u, v)`: each edge both ways in CSR. O(n + m).
pub fn undirected_of_pairs<V>(edges: List<(V, V)>) -> Adj<V, V>:
    var start: List<0..=2_000_000_000> = List.filled(V.K + 1, 0)
    unchecked:
        for (u, v) in edges:
            start[u.to_idx + 1] += 1
            start[v.to_idx + 1] += 1
        for i in 0..V.K: start[i + 1] += start[i]
        if edges.is_empty: return Adj(start, [])
        var arcs = List.filled(start[V.K], edges[0].1)
        var at: List<0..=2_000_000_000> = copy(start)
        for (u, v) in edges:
            arcs[at[u.to_idx]] = v
            at[u.to_idx] += 1
            arcs[at[v.to_idx]] = u
            at[v.to_idx] += 1
        return Adj(start, arcs)

/// `Graph.directed(triples)` on a list of `(u, v, w)`: `for (v, w) in g[u]` in CSR. O(n + m).
pub fn directed_of_triples<V, W>(edges: List<(V, V, W)>) -> Adj<V, (V, W)>:
    var start: List<0..=2_000_000_000> = List.filled(V.K + 1, 0)
    unchecked:
        for (u, _, _) in edges: start[u.to_idx + 1] += 1
        for i in 0..V.K: start[i + 1] += start[i]
        if edges.is_empty: return Adj(start, [])
        var arcs = List.filled(start[V.K], (edges[0].1, edges[0].2))
        var at: List<0..=2_000_000_000> = copy(start)
        for (u, v, w) in edges:
            arcs[at[u.to_idx]] = (v, w)
            at[u.to_idx] += 1
        return Adj(start, arcs)

/// O(n + m).
/// `Graph.undirected(triples)` on a list of `(u, v, w)`: each edge both ways in CSR, `w` riding
/// along. O(n + m).
pub fn undirected_of_triples<V, W>(edges: List<(V, V, W)>) -> Adj<V, (V, W)>:
    var start: List<0..=2_000_000_000> = List.filled(V.K + 1, 0)
    unchecked:
        for (u, v, _) in edges:
            start[u.to_idx + 1] += 1
            start[v.to_idx + 1] += 1
        for i in 0..V.K: start[i + 1] += start[i]
        if edges.is_empty: return Adj(start, [])
        var arcs = List.filled(start[V.K], (edges[0].1, edges[0].2))
        var at: List<0..=2_000_000_000> = copy(start)
        for (u, v, w) in edges:
            arcs[at[u.to_idx]] = (v, w)
            at[u.to_idx] += 1
            arcs[at[v.to_idx]] = (u, w)
            at[v.to_idx] += 1
        return Adj(start, arcs)

/// Connected components of an undirected graph, as a fresh domain `C`. O(n + m).
pub fn components<V, N>(next: N) -> (domain C, comp: [V]C):
    var id: [V]?Int = none
    var k = 0
    for root in V:
        if id[root] != none: continue
        id[root] = k
        var queue = [root]
        while not queue.is_empty:
            let u = queue.pop()
            for a in next.out(u):
                if id[a.to] == none:
                    id[a.to] = k
                    queue.push(a.to)
        k += 1
    domain C = 0..k
    (C: C, comp: [V] v => C(id[v]!))

/// O(n + m).
/// The condensation: the arcs between components, as adjacency lists (duplicates kept).
pub fn condense<V, N, C>(next: N, comp: [V]C) -> [C]List<(to: C)>:
    var out: [C]List<(to: C)> = []
    for u in V:
        for a in next.out(u):
            if comp[u] != comp[a.to]: out[comp[u]].push((to: comp[a.to]))
    out

/// O(n + m).
/// Two-edge-connected components (bridge blocks) of an undirected graph.
/// Each arc in `next.out(u)` carries `(to: V, e: E)` so parallel edges are handled
/// properly (parallel edges are never bridges). Returns a fresh domain `C`
/// for the components and `comp: [V]C`. Every edge between distinct components
/// is a bridge, so `condense(next, comp)` yields the bridge tree.
/// O(n + m), iterative Tarjan low-link.
pub fn two_edge_components<V, N>(next: N) -> (domain C, comp: [V]C):
    let out = [V] v => [a for a in next.out(v)]
    var tin: [V]?Int = none
    var low: [V]Int = 0
    var cid: [V]?Int = none
    var stack = List<V>()
    // (u, next arc index, parent edge id or none)
    var frames = List<(V, Int, ?Int)>()
    var (timer, k) = (0, 0)
    for root in V:
        if tin[root] != none: continue
        frames.push((root, 0, none))
        while not frames.is_empty:
            let (u, i, pe) = frames.last!
            if i == 0:
                tin[u] = timer
                low[u] = timer
                timer += 1
                stack.push(u)
            if i < out[u].len:
                frames.last! = (u, i + 1, pe)
                let a = out[u][i]
                let eid = a.e.to_idx
                if pe != none and eid == pe!: continue
                let v = a.to
                if tin[v] == none:
                    frames.push((v, 0, eid))
                else:
                    low[u] min= tin[v]!
            else:
                frames.pop()
                if not frames.is_empty:
                    let p = frames.last!.0
                    low[p] min= low[u]
                if low[u] == tin[u]!:
                    while true:
                        let v = stack.pop()
                        cid[v] = k
                        if v == u: break
                    k += 1
    domain C = 0..k
    (C: C, comp: [V] v => C(cid[v]!))

/// Articulation points (cut vertices) of an undirected simple graph.

/// Handles disconnected graphs and root child count. O(n + m), iterative.
pub fn articulation_points<V, N>(next: N) -> [V]Bool:
    let out = [V] v => [a.to for a in next.out(v)]
    var tin: [V]?Int = none
    var low: [V]Int = 0
    var is_cut: [V]Bool = false
    var children: [V]Int = 0
    // (u, next neighbor index, parent vertex or none)
    var frames = List<(V, Int, ?V)>()
    var timer = 0
    for root in V:
        if tin[root] != none: continue
        frames.push((root, 0, none))
        while not frames.is_empty:
            let (u, i, p) = frames.last!
            if i == 0:
                tin[u] = timer
                low[u] = timer
                timer += 1
            if i < out[u].len:
                frames.last! = (u, i + 1, p)
                let v = out[u][i]
                if p != none and v == p!: continue
                if tin[v] == none:
                    children[u] += 1
                    frames.push((v, 0, u))
                else:
                    low[u] min= tin[v]!
            else:
                frames.pop()
                if p == none:
                    if children[u] >= 2: is_cut[u] = true
                else:
                    let par = p!
                    low[par] min= low[u]
                    if frames.len > 1 and low[u] >= tin[par]!:
                        is_cut[par] = true
    is_cut

/// Vertex-biconnected components (blocks) of an undirected simple graph.
/// Each returned list contains the vertices of one maximal 2-vertex-connected
/// subgraph (or the two endpoints of a bridge). Articulation points appear in
/// every block that shares them. O(n + m), iterative Tarjan.

/// Vertex-biconnected components (blocks) of an undirected graph via Tarjan DFS. O(|V| + |E|).
pub fn biconnected_components<V, N>(next: N) -> List<List<V>>:
    let out = [V] v => [a.to for a in next.out(v)]
    var tin: [V]?Int = none
    var low: [V]Int = 0
    var stack = List<V>()
    var blocks = List<List<V>>()
    // (u, next neighbor index, parent vertex or none)
    var frames = List<(V, Int, ?V)>()
    var timer = 0
    for root in V:
        if tin[root] != none: continue
        frames.push((root, 0, none))
        while not frames.is_empty:
            let (u, i, p) = frames.last!
            if i == 0:
                tin[u] = timer
                low[u] = timer
                timer += 1
                stack.push(u)
            if i < out[u].len:
                frames.last! = (u, i + 1, p)
                let v = out[u][i]
                if p != none and v == p!: continue
                if tin[v] == none:
                    frames.push((v, 0, u))
                else:
                    low[u] min= tin[v]!
            else:
                frames.pop()
                if let par = p:
                    low[par] min= low[u]
                    if low[u] >= tin[par]!:
                        var comp = List<V>()
                        while true:
                            let w = stack.pop()
                            comp.push(w)
                            if w == u: break
                        comp.push(par)
                        blocks.push(comp)
                else:
                    stack.pop()
    blocks

/// 2-coloring of an undirected graph: returns a side for each vertex, or `none` if an odd cycle exists. O(n + m).
pub fn two_color<V, N>(next: N) -> ?[V]Bool:
    var side: [V]?Bool = none
    for root in V:
        if side[root] != none: continue
        side[root] = false
        var q = [root]
        var head = 0
        while head < q.len:
            let u = q[head]
            head += 1
            let next_c = not side[u]!
            for a in next.out(u):
                let v = a.to
                if side[v] == none:
                    side[v] = next_c
                    q.push(v)
                elif side[v]! != next_c:
                    return none
    [V] v => side[v]!

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
fn bfs_over_a_stored_graph_and_over_a_lambda():
    domain V = 0..6
    domain E = 0..5
    let edges = [E] e => [
        (u: V(0), v: V(1)),
        (u: V(1), v: V(2)),
        (u: V(0), v: V(3)),
        (u: V(3), v: V(2)),
        (u: V(2), v: V(4)),
    ][e.to_idx]
    let g = Graph.undirected(edges)
    let d = bfs(g, from: [V(0)])
    assert [d[v] for v in V] == [0, 1, 2, 1, 3, none]
    // an implicit graph: the same vertices on a ring
    let ring = bfs(from: [V(4), V(1)], next: u => [u.cyc(1)])
    assert [ring[v]! for v in V] == [2, 0, 1, 2, 0, 1]

@test
fn dijkstra_matches_floyd_warshall():
    domain V = 1..=7
    domain E = 1..=14
    for _ in 0..15:
        let edges = [E] e => (u: V.nth(rand(0, 6)), v: V.nth(rand(0, 6)), w: rand(0, 9))
        let g = Graph.directed(edges)
        var best: [V, V]?Int = none
        for v in V: best[v, v] = 0
        for e in E:
            let x = edges[e]
            if best[x.u, x.v] == none or x.w < best[x.u, x.v]!: best[x.u, x.v] = x.w
        for k in V:
            for i in V:
                for j in V:
                    if best[i, k] != none and best[k, j] != none:
                        let via = best[i, k]! + best[k, j]!
                        if best[i, j] == none or via < best[i, j]!: best[i, j] = via
        for s in V:
            let dist = dijkstra(g, from: [(s, 0)], w: a => edges[a.e].w)
            for t in V:
                assert dist[t] == best[s, t]

@test
fn topo_orders_a_dag_and_rejects_a_cycle():
    domain V = 0..6
    let out = [V] v => [V(x) for x in [[1, 2], [3], [3], [4], [], [0]][v.to_idx]]
    let order = topo::<V>(v => out[v])!
    var pos: [V]Int = 0
    for (i, v) in [(i, order[i]) for i in 0..order.len]: pos[v] = i
    for u in V:
        for v in out[u]: assert pos[u] < pos[v]
    assert topo::<V>(v => [v.cyc(1)]) == none
    let stuck = [V] v => [V(x) for x in [[1, 2], [3], [3], [4], [3], [0]][v.to_idx]]
    assert topo::<V>(v => stuck[v]) == none

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
            for t in V: reach[s, t] = d[t] != none
        let (C, comp) = scc::<V>(v => out[v])
        for a in V:
            for b in V:
                assert (comp[a] == comp[b]) == (reach[a, b] and reach[b, a])
                if reach[a, b]: assert comp[a] <= comp[b]
        let dag = condense(v => out[v], comp)
        assert topo::<C>(c => dag[c]) != none

@test
fn components_of_an_undirected_graph():
    domain V = 1..=6
    domain E = 1..=3
    let edges = [E] e => [(u: V(1), v: V(2)), (u: V(2), v: V(3)), (u: V(5), v: V(6))][e.to_idx]
    let g = Graph.undirected(edges)
    let (C, comp) = components(g)
    assert C.K == 3
    var size: [C]Int = 0
    for v in V: size[comp[v]] += 1
    assert sorted(size[c] for c in C) == [1, 2, 3]
    assert comp[V(1)] == comp[V(3)] and comp[V(4)] != comp[V(5)]

fn random_edges<V, E>() -> [E](u: V, v: V):
    [E] e => (u: V.nth(rand(0, V.K - 1)), v: V.nth(rand(0, V.K - 1)))

/// Are `a` and `b` connected when edge `skip` and vertex `gone` are removed?
fn linked<V, E>(edges: [E](u: V, v: V), a: V, b: V, skip: ?E, gone: ?V) -> Bool:
    var d = Dsu<V>()
    for e in E:
        if e != skip and edges[e].u != gone and edges[e].v != gone: d.unite(edges[e].u, edges[e].v)
    d.same(a, b)

@test
fn bridges_and_cut_vertices_match_removing_them():
    domain V = 0..6
    domain E = 0..7
    for _ in 0..12:
        let edges = random_edges::<V, E>()
        let g = Graph.undirected(edges)
        let (C, comp) = two_edge_components(g)
        let cut = articulation_points(g)
        for a in V:
            for b in V:
                // same 2-edge-connected component: no single edge separates them
                let robust = all(
                    linked(edges, a, b, e, none) for e in E
                ) and linked(edges, a, b, none, none)
                assert (comp[a] == comp[b]) == robust
        for v in V:
            var splits = false
            for a in V:
                for b in V:
                    if a != v and b != v and linked(
                        edges,
                        a,
                        b,
                        none,
                        none,
                    ) and not linked(edges, a, b, none, v): splits = true
            assert cut[v] == splits

@test
fn blocks_of_a_small_graph():
    // two triangles sharing vertex 2, and a pendant edge 4 - 5
    domain V = 0..6
    domain E = 0..7
    let edges = [E] e => [
        (u: V(0), v: V(1)),
        (u: V(1), v: V(2)),
        (u: V(2), v: V(0)),
        (u: V(2), v: V(3)),
        (u: V(3), v: V(4)),
        (u: V(4), v: V(2)),
        (u: V(4), v: V(5)),
    ][e.to_idx]
    let g = Graph.undirected(edges)
    let blocks = sorted(sorted(v.to_idx for v in b) for b in biconnected_components(g))
    assert blocks == [[0, 1, 2], [2, 3, 4], [4, 5]]
    assert [articulation_points(g)[v] for v in V] == [false, false, true, false, true, false]

@test
fn two_color_finds_a_colouring_or_an_odd_cycle():
    domain V = 0..7
    domain E = 0..7
    for _ in 0..60:
        let edges = random_edges::<V, E>()
        let g = Graph.undirected(edges)
        let side = two_color(g)
        // brute force over all 2^7 colourings
        var possible = false
        for mask in 0..128:
            if all(
                ((mask >> edges[e].u.to_idx) & 1) != ((mask >> edges[e].v.to_idx) & 1) for e in E
            ): possible = true
        assert (side != none) == possible
        if side != none:
            for e in E: assert side![edges[e].u] != side![edges[e].v]

@test
fn vertexless_closure_graphs_with_turbofish():
    domain V = 0..5
    // DAG 0 -> 1 -> 3, 0 -> 2 -> 3, 3 -> 4 via pure closure (no Graph or adj array)
    let out = [V] u => [V(x) for x in [[1, 2], [3], [3], [4], []][u.to_idx]]
    let order = topo::<V>(u => out[u])!
    assert order.len == 5 and order[0] == V(0) and order[4] == V(4)
    // Cyclic closure graph returns none for topo
    let cyc = topo::<V>(u => [V((u.to_idx + 1) % 5)])
    assert cyc == none
    // scc and components also work with vertexless closures via turbofish
    let (SC, scomp) = scc::<V>(u => if u == V(0): [V(1)] else: if u == V(1): [V(0)] else: [])
    assert SC.K == 4 and scomp[V(0)] == scomp[V(1)]
    let (CC, ccomp) = components::<V>(u => if u == V(0): [V(1)] else: if u == V(1): [V(0)] else: [])
    assert CC.K == 4 and ccomp[V(0)] == ccomp[V(1)]

@test
fn a_plain_edge_list_needs_no_edge_domain():
    domain V = 1..=4
    let g = Graph.undirected([(V(1), V(2)), (V(2), V(3))])
    assert [[v.to_idx for v in g[u]] for u in V] == [[1], [0, 2], [1], []]
    let d = Graph.directed([(V(1), V(2), 5), (V(1), V(4), 7)])
    assert [(v.to_idx, w) for (v, w) in d[V(1)]] == [(1, 5), (3, 7)]
    assert d[V(2)].is_empty

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
    pub fn get(self, v: V) -> ?W = dist[v]

    /// O(1) per item: `for (v, d) in bfs(g, from: [s])` walks the reached
    /// vertices in settling order with their distances.
    pub fn iter(self) -> yields (v: V, d: W):
        for v in order: yield (v: v, d: dist[v]!)

    /// O(n + m): the arcs on some shortest path (`p[a.to] == p[u] + w(a)`), the
    /// DAG that counting or listing shortest paths walks in `order`.
    pub fn tight<N>(self, next: N, w: A -> W) -> [V]List<A> = todo

    /// Does some source reach `v`? O(1).
    pub fn reached(self, v: V) -> Bool = dist[v] != none

    /// O(path length).
    /// The vertices of one shortest path from a source to `t`, both included, or
    /// `none` if `t` is unreachable. O(length).
    pub fn path(self, t: V) -> ?List<V>:
        if dist[t] == none: return none
        var out = [t]
        var at = t
        while let b = back[at]:
            out.push(b.from)
            at = b.from
        reverse(mut out)
        out

    /// The arcs of `path(t)`, in walking order. O(length).
    pub fn arcs(self, t: V) -> ?List<A>:
        if dist[t] == none: return none
        var out = List<A>()
        var at = t
        while let b = back[at]:
            out.push(b.via)
            at = b.from
        reverse(mut out)
        out

/// O(n + m).
/// Fewest arcs from any of `from` to each vertex; `until: t` stops once `t` is
/// reached. O(n + m).
pub fn bfs<V, N, A>(next: N, from: List<V>, until: ?V = none) -> Paths<V, Int, A>:
    var dist: [V]?Int = none
    var back: [V]?(from: V, via: A) = none
    var queue = List<V>()
    for s in from:
        if dist[s] == none:
            dist[s] = 0
            queue.push(s)
    var head = 0
    while head < queue.len:
        if let t = until and dist[t] != none: break
        let u = queue[head]
        head += 1
        let d = dist[u]! + 1
        for a in next.out(u):
            if dist[a.to] == none:
                dist[a.to] = d
                back[a.to] = (from: u, via: a)
                queue.push(a.to)
    Paths(dist, back, queue)

/// O(n + m).
/// Shortest distances when every weight `w(a)` is 0 or 1, from every vertex of `from`
/// at distance 0: a deque, with 0-arcs pushed to the front; `until: t` stops once
/// `t` is settled. O(n + m); traps on any other weight.
pub fn bfs01<V, N, A>(next: N, from: List<V>, w: A -> Int, until: ?V = none) -> Paths<V, Int, A>:
    var dist: [V]?Int = none
    var back: [V]?(from: V, via: A) = none
    var done: [V]Bool = false
    var order = List<V>()
    var queue = Deque<V>()
    for s in from:
        if dist[s] == none:
            dist[s] = 0
            queue.push(s)
    while let u = queue.pop_front():
        if done[u]: continue
        done[u] = true
        order.push(u)
        if until == u: break
        let d = dist[u]!
        for a in next.out(u):
            let c = w(a)
            assert c == 0 or c == 1
            let v = a.to
            if dist[v] == none or d + c < dist[v]!:
                dist[v] = d + c
                back[v] = (from: u, via: a)
                if c == 0: queue.push_front(v) else: queue.push(v)
    Paths(dist, back, order)

/// O((n + m) log n).
/// Shortest distances for non-negative weights, from `(vertex, starting distance)`
/// pairs; `until: t` stops once `t` is settled. O((n + m) log m) with a binary heap
/// and lazy deletion; traps on a negative weight.
pub fn dijkstra<V, N, A, W>(
    next: N,
    from: List<(V, W)>,
    w: A -> W,
    until: ?V = none,
) -> Paths<V, W, A>:
    var dist: [V]?W = none
    var back: [V]?(from: V, via: A) = none
    var order = List<V>()
    var heap = MinHeap<(W, V)>()
    for (s, d) in from:
        if dist[s] == none or d < dist[s]!:
            dist[s] = d
            heap.push((d, s))
    while let (d, u) = heap.pop():
        if dist[u]! < d: continue
        order.push(u)
        if until == u: break
        for a in next.out(u):
            let c = w(a)
            assert c >= W.zero
            let nd = d + c
            let v = a.to
            if dist[v] == none or nd < dist[v]!:
                dist[v] = nd
                back[v] = (from: u, via: a)
                heap.push((nd, v))
    Paths(dist, back, order)

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
) -> Paths<S, W, M>:
    let price = [cost(mv) for mv in moves]
    for p in price: assert p >= W.zero
    var dist: [S]?W = none
    var back: [S]?(from: S, via: M) = none
    var order = List<S>()
    for s in from:
        if dist[s] == none:
            dist[s] = W.zero
            order.push(s)
            if until == s: return Paths(dist, back, order)
    var at = List.filled(moves.len, 0)
    while true:
        var best: ?(d: W, j: Int, t: S) = none
        for j in 0..moves.len:
            while at[j] < order.len:
                let s = order[at[j]]
                if let t = step(moves[j], s) and dist[t] == none:
                    let d = dist[s]! + price[j]
                    if let b = best:
                        if d < b.d: best = (d: d, j: j, t: t)
                    else:
                        best = (d: d, j: j, t: t)
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

/// O(n m).
/// Shortest distances with any weights, from `(vertex, starting distance)` pairs, or
/// `none` when a source reaches a negative cycle. `order` lists each reached vertex
/// after its parent. O(n * m).
pub fn bellman_ford<V, N, A, W>(next: N, from: List<(V, W)>, w: A -> W) -> ?Paths<V, W, A>:
    var dist: [V]?W = none
    var back: [V]?(from: V, via: A) = none
    for (s, d) in from:
        if dist[s] == none or d < dist[s]!: dist[s] = d
    var rounds = 0
    var changed = true
    while changed:
        if rounds == V.K: return none
        rounds += 1
        changed = false
        for u in V:
            if let d = dist[u]:
                for a in next.out(u):
                    let nd = d + w(a)
                    if dist[a.to] == none or nd < dist[a.to]!:
                        dist[a.to] = nd
                        back[a.to] = (from: u, via: a)
                        changed = true
    var kids: [V]List<V> = []
    var order = List<V>()
    for v in V:
        if let b = back[v]: kids[b.from].push(v)
        elif dist[v] != none: order.push(v)
    var head = 0
    while head < order.len:
        let u = order[head]
        head += 1
        for v in kids[u]: order.push(v)
    Paths(dist, back, order)

/// O(n^3).
/// All-pairs shortest distances from the direct weights `w[u, v]` (`none`: no edge),
/// or `none` if there is a negative cycle. A vertex is at distance 0 from itself
/// unless a cheaper cycle exists. O(n^3) time, O(n^2) memory.
pub fn floyd_warshall<V>(w: [V, V]?Int) -> ?[V, V]?Int:
    var d = copy(w)
    for v in V:
        if d[v, v] == none or d[v, v]! > 0: d[v, v] = 0
    for k in V:
        for i in V:
            if let ik = d[i, k]:
                for j in V:
                    if let kj = d[k, j]:
                        if d[i, j] == none or ik + kj < d[i, j]!: d[i, j] = ik + kj
    if any(d[v, v]! < 0 for v in V): none else: d

/// O(n + m).
/// One directed cycle, as its vertices in walking order (each is followed by the next,
/// and the last by the first), or `none` if the graph is acyclic. O(n + m), no recursion.
pub fn find_cycle<V, N>(next: N) -> ?List<V>:
    let out = [V] v => [a.to for a in next.out(v)]
    var state: [V]Int = 0  // 0 unseen, 1 on the dfs path, 2 finished
    var path = List<(V, Int)>()
    for root in V:
        if state[root] != 0: continue
        state[root] = 1
        path.push((root, 0))
        while not path.is_empty:
            let (u, i) = path.last!
            if i < out[u].len:
                path.last! = (u, i + 1)
                let v = out[u][i]
                if state[v] == 0:
                    state[v] = 1
                    path.push((v, 0))
                elif state[v] == 1:
                    var cyc = [x for (x, _) in path]
                    let at = first(j for j in 0..cyc.len if cyc[j] == v)!
                    return cyc[at..].to_list()
            else:
                state[u] = 2
                path.pop()
    none

/// O(n + m).
/// Which edges are bridges: removing one disconnects its ends. Needs arcs with edge ids
/// (a `Graph.undirected`), so a parallel edge is not mistaken for the way back. O(n + m).
pub fn bridges<V, E, N>(next: N) -> [E]Bool:
    let (_, comp) = two_edge_components::<V>(next)
    var bridge: [E]Bool = false
    for u in V:
        for a in next.out(u):
            if comp[u] != comp[a.to]: bridge[a.e] = true
    bridge

/// O(m) memory.
/// A walk: where it starts, then each arc taken (`to` is where the arc leads, `e` the
/// edge it uses), the same arcs a `Graph` yields.
pub struct Walk<V, E>:
    start: V
    arcs: List<(to: V, e: E)>

    /// The vertices visited, `start` first. O(length).
    pub @get fn vertices(self) -> List<V> = [start] + [a.to for a in arcs]

/// O(n + m).
/// A walk that uses every edge exactly once (Hierholzer's algorithm), or `none` if
/// there is none. `directed` says whether `(u, v)` may only be walked from `u` to `v`.
/// A closed walk starts at the smallest vertex with an edge; an open one at the
/// vertex with an extra way out (directed) or the smaller odd vertex. O(n + m).
pub fn euler_walk<V, E>(edges: [E](u: V, v: V), directed: Bool) -> ?Walk<V, E>:
    let g = if directed: Graph.directed(edges) else: Graph.undirected(edges)
    var surplus: [V]Int = 0
    for e in E:
        surplus[edges[e].u] += 1
        if directed: surplus[edges[e].v] -= 1
        else: surplus[edges[e].v] += 1
    var start: ?V = none
    var opens = 0
    for v in V:
        let extra = if directed: surplus[v] else: surplus[v] % 2
        if extra > 1 or extra < -1: return none
        if extra == 1:
            opens += 1
            if start == none: start = v
    if opens > (if directed: 1 else: 2): return none
    if start == none: start = first(v for v in V if g.deg(v) > 0)
    if start == none: return Walk(V.first, List<(to: V, e: E)>())
    let out = [V] v => [a for a in g.out(v)]
    var at: [V]Int = 0
    var used: [E]Bool = false
    var stack: List<(v: V, came: ?(to: V, e: E))> = [(v: start!, came: none)]
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
    if arcs.len != E.K: return none
    reverse(mut arcs)
    Walk(start!, arcs)

/// O(1).
/// A variable or its negation: `Lit(x, true)` is "x holds".
pub struct Lit<X>(x: X, value: Bool):
    /// O(1).
    /// The opposite literal.
    pub @get fn flip(self) -> Lit<X> = Lit(x, not value)

/// A conjunction of two-literal clauses over the variables `X`. Every clause is O(1)
/// to add; `solve` is O(|X| + clauses).
pub struct TwoSat<X>:
    /// Implications between literal nodes: node `2 * i` is "variable i is false",
    /// `2 * i + 1` "is true"; variables past `X.K` are helpers of `at_most_one`.
    imp: List<List<Int>>

    /// An instance with no clauses over the variables of `X`. O(|X|).
    pub fn new() -> TwoSat<X>:
        TwoSat([[] for _ in 0..2 * X.K])

    /// O(1).
    /// At least one of `a`, `b` holds.
    pub fn add_or(mut self, a: Lit<X>, b: Lit<X>):
        self.clause(lit_node(a), lit_node(b))

    /// O(1).
    /// `a` holds.
    pub fn must(mut self, a: Lit<X>):
        self.add_or(a, a)

    /// O(1).
    /// If `a` holds then `b` holds.
    pub fn implies(mut self, a: Lit<X>, b: Lit<X>):
        self.add_or(a.flip, b)

    /// O(k).
    /// At most one of `lits` holds: one helper variable per literal ("some literal so
    /// far holds") keeps it O(k) clauses instead of O(k^2).
    pub fn at_most_one(mut self, lits: List<Lit<X>>):
        var prev: ?Int = none
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

    /// O(1): exactly one of `a`, `b` holds.
    pub fn add_xor(mut self, a: Lit<X>, b: Lit<X>) = todo

    /// O(1): `a` and `b` hold together or not at all.
    pub fn add_eq(mut self, a: Lit<X>, b: Lit<X>) = todo

    /// O(n + m) over the clauses.
    /// An assignment satisfying every clause, or `none` if there is none.
    pub fn solve(self) -> ?[X]Bool:
        domain N = 0..imp.len
        let (C, comp) = scc::<N>(u => [N(v) for v in imp[u.to_idx]])
        // components are in topological order: a literal is true when it comes after
        // its negation
        for i in 0..imp.len / 2:
            if comp[N(2 * i)] == comp[N(2 * i + 1)]: return none
        [X] x => comp[N(2 * x.to_idx + 1)] > comp[N(2 * x.to_idx)]

    fn clause(mut self, a: Int, b: Int):
        imp[a ^ 1].push(b)
        imp[b ^ 1].push(a)

fn lit_node<X>(l: Lit<X>) -> Int = 2 * l.x.to_idx + Int(l.value)

@test
fn paths_witness_their_distances():
    domain V = 0..7
    domain E = 0..12
    for _ in 0..40:
        let edges = [E] e => (u: V.nth(rand(0, 6)), v: V.nth(rand(0, 6)), w: rand(0, 1))
        let g = Graph.directed(edges)
        let s = V.nth(rand(0, 6))
        let heap = dijkstra(g, from: [(s, 0)], w: a => edges[a.e].w)
        let deque = bfs01(g, from: [s], w: a => edges[a.e].w)
        let unit = bfs(g, from: [s])
        let ones = dijkstra(g, from: [(s, 0)], w: a => 1)
        for v in V:
            assert heap[v] == deque[v] and unit[v] == ones[v]
            assert unit.reached(v) == (unit.path(v) != none)
        for p in [heap, deque]:
            for v in V if p.reached(v):
                let path = p.path(v)!
                let arcs = p.arcs(v)!
                assert path[0] == s and path.last! == v and arcs.len == path.len - 1
                assert sum(edges[a.e].w for a in arcs) == p[v]!
                for i in 0..arcs.len: assert edges[arcs[i].e].u == path[i] and arcs[i].to == path[i + 1]
            for i in 1..p.order.len: assert p[p.order[i - 1]]! <= p[p.order[i]]!
            assert p.order.len == sum(1 for v in V if p.reached(v))

@test
fn fixed_cost_paths_match_dijkstra_on_the_same_moves():
    domain S = 0..30
    let moves = [1, 2, 7]
    fn price(m: Int) -> Int = if m == 1: 3 elif m == 2: 4 else: 9
    fn step(m: Int, s: S) -> ?S:
        let t = if m == 1: s.to_idx + 1 else: s.to_idx * m % 31
        if t < 30: S(t) else: none
    let p = fixed_cost_paths(from: [S(1)], moves: moves, cost: price, step: step)
    let q = dijkstra::<S>(
        s => [(to: t, m: m) for m in moves if let t = step(m, s)],
        from: [(S(1), 0)],
        w: a => price(a.m),
    )
    for s in S:
        assert p[s] == q[s]
        if let path = p.path(s):
            assert path[0] == S(1) and path.last! == s
    for i in 1..p.order.len: assert p[p.order[i - 1]]! <= p[p.order[i]]!

@test
fn until_stops_at_the_target_with_its_exact_distance():
    domain V = 0..40
    domain E = 0..80
    let edges = [E] e => (
        u: V.nth(e.to_idx % 40),
        v: V.nth((e.to_idx * 7 + 3) % 40),
        w: e.to_idx % 3,
    )
    let g = Graph.directed(edges)
    let full = dijkstra(g, from: [(V(0), 0)], w: a => edges[a.e].w)
    let hops = bfs(g, from: [V(0)])
    let cheap = bfs01(g, from: [V(0)], w: a => min(edges[a.e].w, 1))
    for t in V:
        let p = dijkstra(g, from: [(V(0), 0)], w: a => edges[a.e].w, until: t)
        if full[t] == none:
            assert p.order == full.order
            continue
        assert p[t] == full[t] and p.order.last == t and p.path(t) != none
        for v in p.order: assert p[v] == full[v]
        assert bfs(g, from: [V(0)], until: t)[t] == hops[t]
        let q = bfs01(g, from: [V(0)], w: a => min(edges[a.e].w, 1), until: t)
        assert q[t] == cheap[t] and q.order.last == t
    domain S = 0..30
    fn step(m: Int, s: S) -> ?S = if s.to_idx + m < 30: S(s.to_idx + m) else: none
    let all = fixed_cost_paths(from: [S(0)], moves: [1, 4], cost: m => 5 - m, step: step)
    let one = fixed_cost_paths(
        from: [S(0)],
        moves: [1, 4],
        cost: m => 5 - m,
        step: step,
        until: S(17),
    )
    assert one[S(17)] == all[S(17)] and one.order.last == S(17) and one.order.len < all.order.len

@test
fn bellman_ford_matches_floyd_warshall_and_finds_negative_cycles():
    domain V = 0..6
    domain E = 0..9
    for _ in 0..40:
        let edges = [E] e => (u: V.nth(rand(0, 5)), v: V.nth(rand(0, 5)), w: rand(-2, 8))
        var w: [V, V]?Int = none
        for e in E:
            let x = edges[e]
            if w[x.u, x.v] == none or x.w < w[x.u, x.v]!: w[x.u, x.v] = x.w
        let all = floyd_warshall(w)
        let g = Graph.directed(edges)
        for s in V:
            let p = bellman_ford(g, from: [(s, 0)], w: a => edges[a.e].w)
            if let d = all:
                for t in V: assert p![t] == d[s, t]
                for v in V if p!.reached(v):
                    assert sum(edges[a.e].w for a in p!.arcs(v)!) == p![v]!
            elif let q = p:
                // a negative cycle exists, but this source does not reach it
                for e in E if q.reached(edges[e].u):
                    assert q[edges[e].v]! <= q[edges[e].u]! + edges[e].w

@test
fn find_cycle_exists_exactly_when_topo_fails():
    domain V = 0..7
    for _ in 0..60:
        let out = [V] v => [V.nth(rand(0, 6)) for _ in 0..rand(0, 2) if rand(0, 2) > 0]
        let cyc = find_cycle::<V>(v => out[v])
        assert (cyc == none) == (topo::<V>(v => out[v]) != none)
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
        let got = euler_walk(edges, directed: directed)
        assert (got != none) == has_walk([(edges[e].u, edges[e].v) for e in E], directed)
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

fn has_walk<V>(edges: List<(V, V)>, directed: Bool) -> Bool:
    var used = List.filled(edges.len, false)
    edges.is_empty or any(extend(edges, directed, mut used, v, 0) for v in V)

fn extend<V>(edges: List<(V, V)>, directed: Bool, mut used: List<Bool>, at: V, done: Int) -> Bool:
    if done == edges.len: return true
    for i in 0..edges.len:
        if used[i]: continue
        let (u, v) = edges[i]
        var to: ?V = none
        if u == at: to = v
        elif not directed and v == at: to = u
        if let w = to:
            used[i] = true
            if extend(edges, directed, mut used, w, done + 1): return true
            used[i] = false
    false

@test
fn at_most_one_forbids_every_pair_and_allows_each_single():
    domain X = 0..5
    for _ in 0..40:
        var sat = TwoSat<X>()
        var clauses: List<(Lit<X>, Lit<X>)> = []
        for _ in 0..rand(0, 4):
            let a = Lit(X.nth(rand(0, 4)), rand(0, 1) == 1)
            let b = Lit(X.nth(rand(0, 4)), rand(0, 1) == 1)
            sat.add_or(a, b)
            clauses.push((a, b))
        let group = [Lit(X.nth(i), rand(0, 1) == 1) for i in 0..rand(1, 5)]
        sat.at_most_one(group)
        fn holds(mask: Int, l: Lit<X>) -> Bool = (((mask >> l.x.to_idx) & 1) == 1) == l.value
        fn fits(mask: Int) -> Bool = all(holds(mask, a) or holds(mask, b) for (a, b) in clauses)
        let possible = any(fits(m) and sum(1 for l in group if holds(m, l)) <= 1 for m in 0..32)
        let got = sat.solve()
        assert (got != none) == possible
        if let pick = got:
            assert sum(1 for l in group if pick[l.x] == l.value) <= 1
            for (a, b) in clauses: assert pick[a.x] == a.value or pick[b.x] == b.value


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
