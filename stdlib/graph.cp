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

/// What an algorithm walks: a `Graph` gives its arcs out of `u`, and any fn `u => ...` is
/// simply called.
pub trait Neighbours:
    fn out<V>(self, u: V) -> Iter<V> = self(u)

/// What a neighbour list holds: an arc says where it leads with its `to` field, and a plain
/// vertex leads to itself.
pub trait Target:
    @get fn to(self) -> Self = self

/// The `V.len - 1` edges of a tree on `V`, one `u v` pair per line, as an input
/// gives them. `cpl stress` makes and shrinks it as a tree.
pub type Tree<V> = [0..V.len - 1](u: V, v: V)

/// One edge on `V` for each value of `E`, `u v` per line, with no loop and no
/// pair twice. `cpl stress` makes and shrinks it so.
pub type SimpleGraph<V, E> = [E](u: V, v: V)

/// A `SimpleGraph` in which every vertex reaches every other.
pub type ConnectedGraph<V, E> = [E](u: V, v: V)

/// One arc `u -> v` on `V` for each value of `E`, with no arc twice and no
/// directed cycle. `cpl stress` makes and shrinks it so.
pub type Dag<V, E> = [E](u: V, v: V)

/// One arc `u -> v` on `V` for each value of `E`, with no loop and no arc
/// twice; `u v` and `v u` are two arcs. `cpl stress` makes and shrinks it so.
pub type Digraph<V, E> = [E](u: V, v: V)

/// A fixed graph in compressed sparse row form: one offsets list and one arcs list.
/// Built in O(n + m); memory is n + 1 offsets and one `(to, e)` per arc.
/// An undirected edge is two arcs with the same `e`.
pub struct Graph<V, E>:
    start: List<Int>
    arcs: List<(to: V, e: E)>

    /// One arc `u -> v` per edge.
    pub fn directed(edges: [E](u: V, v: V)) -> Graph<V, E> = Graph<V, E>.build(edges, false)

    /// Two arcs per edge, `u -> v` and `v -> u`, sharing the edge id.
    pub fn undirected(edges: [E](u: V, v: V)) -> Graph<V, E> = Graph<V, E>.build(edges, true)

    fn build(edges: [E](u: V, v: V), both: Bool) -> Graph<V, E>:
        var start = List.filled(V.K + 1, 0)
        for e in E:
            start[edges[e].u.to_idx + 1] += 1
            if both: start[edges[e].v.to_idx + 1] += 1
        for i in 0..V.K: start[i + 1] += start[i]
        if E.K == 0: return Graph(start, [])
        var arcs = List.filled(start[V.K], (to: edges[E.first].v, e: E.first))
        var at = copy(start)
        for e in E:
            let (u, v) = (edges[e].u, edges[e].v)
            arcs[at[u.to_idx]] = (to: v, e: e)
            at[u.to_idx] += 1
            if both:
                arcs[at[v.to_idx]] = (to: u, e: e)
                at[v.to_idx] += 1
        Graph(start, arcs)

    /// The arcs out of `u`, in edge order: what makes a `Graph` usable wherever `next` is
    /// expected. O(deg u).
    pub fn out(self, u: V) -> yields (to: V, e: E):
        for i in start[u.to_idx]..start[u.to_idx + 1]:
            yield arcs[i]

    /// `g[u]`: the arcs out of `u`, as a list.
    pub fn get(self, u: V) -> List<(to: V, e: E)> = [
        arcs[i] for i in start[u.to_idx]..start[u.to_idx + 1]
    ]

    /// The number of arcs out of `u`. O(1).
    pub fn deg(self, u: V) -> Int = start[u.to_idx + 1] - start[u.to_idx]

    /// The graph with every arc reversed. A rebuild, O(n + m).
    pub fn transpose(self) -> Graph<V, E>:
        var back = List.filled(V.K + 1, 0)
        for a in arcs: back[a.to.to_idx + 1] += 1
        for i in 0..V.K: back[i + 1] += back[i]
        var out = copy(arcs)
        var at = copy(back)
        for u in V:
            for a in self.out(u):
                out[at[a.to.to_idx]] = (to: u, e: a.e)
                at[a.to.to_idx] += 1
        Graph(back, out)

/// Kahn's peeling: repeatedly remove a vertex with no arcs left coming in, and return the
/// vertices in the order they were removed. What stays is every vertex on a cycle or
/// reachable from one, so the order covers all of `V` exactly when the graph is acyclic.
/// For a vertexless closure graph, pass the vertex domain via turbofish: `peel::<V>(u => ...)`.
/// O(n + m).
pub fn peel<V, N>(next: N) -> List<V>:
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

/// A topological order, or `none` if there is a cycle: a `peel` that removed every vertex.
/// O(n + m). For a vertexless closure graph, pass the vertex domain via turbofish:
/// `topo::<V>(u => ...)`.
pub fn topo<V, N>(next: N) -> ?List<V>:
    let order = peel::<V>(next)
    if order.len == V.K: order else: none

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

/// `Graph.directed(pairs)` on a plain list of `(u, v)`: the neighbours of each vertex.
/// No edge domain, and `for v in g[u]` yields the vertex itself. O(n + m).
pub fn directed_of_pairs<V>(edges: List<(V, V)>) -> [V]List<V>:
    var g: [V]List<V> = []
    for (u, v) in edges: g[u].push(v)
    g

/// `Graph.undirected(pairs)` on a plain list of `(u, v)`: each edge both ways. O(n + m).
pub fn undirected_of_pairs<V>(edges: List<(V, V)>) -> [V]List<V>:
    var g: [V]List<V> = []
    for (u, v) in edges:
        g[u].push(v)
        g[v].push(u)
    g

/// `Graph.directed(triples)` on a list of `(u, v, w)`: `for (v, w) in g[u]`. O(n + m).
pub fn directed_of_triples<V, W>(edges: List<(V, V, W)>) -> [V]List<(V, W)>:
    var g: [V]List<(V, W)> = []
    for (u, v, w) in edges: g[u].push((v, w))
    g

/// `Graph.undirected(triples)` on a list of `(u, v, w)`: each edge both ways, `w` riding
/// along. O(n + m).
pub fn undirected_of_triples<V, W>(edges: List<(V, V, W)>) -> [V]List<(V, W)>:
    var g: [V]List<(V, W)> = []
    for (u, v, w) in edges:
        g[u].push((v, w))
        g[v].push((u, w))
    g

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

/// The condensation: the arcs between components, as adjacency lists (duplicates kept).
pub fn condense<V, N, C>(next: N, comp: [V]C) -> [C]List<(to: C)>:
    var out: [C]List<(to: C)> = []
    for u in V:
        for a in next.out(u):
            if comp[u] != comp[a.to]: out[comp[u]].push((to: comp[a.to]))
    out

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

/// What a search found: `p[v]` is the distance to `v` (`none` if unreachable),
/// `parent[v]` the vertex before `v` on one shortest path, `via[v]` the arc or
/// move that reached it, and `order` the reached vertices in settling order, so
/// every vertex comes after its parent and distances never decrease. A search
/// given `until: t` stops once `t` is settled: distances in `order` are exact,
/// and any other vertex holds `none` or an upper bound.
pub struct Paths<V, W, A>:
    dist: [V]?W
    parent: [V]?V
    via: [V]?A
    order: List<V>

    /// `p[v]`: the distance to `v`, or `none` if no source reaches it. O(1).
    pub fn get(self, v: V) -> ?W = dist[v]

    /// Does some source reach `v`? O(1).
    pub fn reached(self, v: V) -> Bool = dist[v] != none

    /// The vertices of one shortest path from a source to `t`, both included, or
    /// `none` if `t` is unreachable. O(length).
    pub fn path(self, t: V) -> ?List<V>:
        if dist[t] == none: return none
        var out = [t]
        var at = t
        while let u = parent[at]:
            out.push(u)
            at = u
        reverse(mut out)
        out

    /// The arcs of `path(t)`, in walking order. O(length).
    pub fn arcs(self, t: V) -> ?List<A>:
        if dist[t] == none: return none
        var out = List<A>()
        var at = t
        while let a = via[at]:
            out.push(a)
            at = parent[at]!
        reverse(mut out)
        out

/// Fewest arcs from any of `from` to each vertex; `until: t` stops once `t` is
/// reached. O(n + m).
pub fn bfs<V, N, A>(next: N, from: List<V>, until: ?V = none) -> Paths<V, Int, A>:
    var dist: [V]?Int = none
    var parent: [V]?V = none
    var via: [V]?A = none
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
                parent[a.to] = u
                via[a.to] = a
                queue.push(a.to)
    Paths(dist, parent, via, queue)

/// Shortest distances when every weight `w(a)` is 0 or 1, from every vertex of `from`
/// at distance 0: a deque, with 0-arcs pushed to the front; `until: t` stops once
/// `t` is settled. O(n + m); traps on any other weight.
pub fn bfs01<V, N, A>(next: N, from: List<V>, w: A -> Int, until: ?V = none) -> Paths<V, Int, A>:
    var dist: [V]?Int = none
    var parent: [V]?V = none
    var via: [V]?A = none
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
                parent[v] = u
                via[v] = a
                if c == 0: queue.push_front(v) else: queue.push(v)
    Paths(dist, parent, via, order)

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
    var parent: [V]?V = none
    var via: [V]?A = none
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
                parent[v] = u
                via[v] = a
                heap.push((nd, v))
    Paths(dist, parent, via, order)

/// Shortest distances over a state space `S` whose moves have fixed non-negative
/// costs: from every state of `from` at distance 0, `step(mv, s)` is where move `mv`
/// leads from `s` (`none`: not allowed there), at cost `cost(mv)`. One FIFO per move
/// replaces the heap: a move's queue receives states in non-decreasing distance, so
/// the cheapest head is always next; `until: t` stops once `t` is settled.
/// O(|S| * |moves|).
pub fn fixed_cost_paths<S, M, W>(
    from: List<S>,
    moves: List<M>,
    cost: M -> W,
    step: (M, S) -> ?S,
    until: ?S = none,
) -> Paths<S, W, M>:
    let price = [cost(mv) for mv in moves]
    var dist: [S]?W = none
    var parent: [S]?S = none
    var via: [S]?M = none
    var done: [S]Bool = false
    var order = List<S>()
    var queues: List<List<(W, S)>> = [[] for _ in 0..=moves.len]
    var heads = List.filled(moves.len + 1, 0)
    for s in from:
        if dist[s] == none:
            let zero = W.zero
            dist[s] = zero
            queues[0].push((zero, s))
    while true:
        var best: ?Int = none
        for i in 0..=moves.len:
            if heads[i] < queues[i].len:
                if let b = best:
                    if queues[i][heads[i]].0 < queues[b][heads[b]].0: best = i
                else:
                    best = i
        if best == none: break
        let i = best!
        let (d, s) = queues[i][heads[i]]
        heads[i] += 1
        if done[s]: continue
        done[s] = true
        order.push(s)
        if until == s: break
        for j in 0..moves.len:
            if let t = step(moves[j], s):
                let nd = d + price[j]
                assert nd >= d
                if dist[t] == none or nd < dist[t]!:
                    dist[t] = nd
                    parent[t] = s
                    via[t] = moves[j]
                    queues[j + 1].push((nd, t))
    Paths(dist, parent, via, order)

/// Shortest distances with any weights, from `(vertex, starting distance)` pairs, or
/// `none` when a source reaches a negative cycle. `order` lists each reached vertex
/// after its parent. O(n * m).
pub fn bellman_ford<V, N, A, W>(next: N, from: List<(V, W)>, w: A -> W) -> ?Paths<V, W, A>:
    var dist: [V]?W = none
    var parent: [V]?V = none
    var via: [V]?A = none
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
                        parent[a.to] = u
                        via[a.to] = a
                        changed = true
    var kids: [V]List<V> = []
    var order = List<V>()
    for v in V:
        if let p = parent[v]: kids[p].push(v)
        elif dist[v] != none: order.push(v)
    var head = 0
    while head < order.len:
        let u = order[head]
        head += 1
        for v in kids[u]: order.push(v)
    Paths(dist, parent, via, order)

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

/// Which edges are bridges: removing one disconnects its ends. Needs arcs with edge ids
/// (a `Graph.undirected`), so a parallel edge is not mistaken for the way back. O(n + m).
pub fn bridges<V, E, N>(next: N) -> [E]Bool:
    let (_, comp) = two_edge_components::<V>(next)
    var bridge: [E]Bool = false
    for u in V:
        for a in next.out(u):
            if comp[u] != comp[a.to]: bridge[a.e] = true
    bridge

/// A walk: where it starts, then each arc taken (`to` is where the arc leads, `e` the
/// edge it uses), the same arcs a `Graph` yields.
pub struct Walk<V, E>:
    start: V
    arcs: List<(to: V, e: E)>

    /// The vertices visited, `start` first. O(length).
    pub @get fn vertices(self) -> List<V> = [start] + [a.to for a in arcs]

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

/// A variable or its negation: `Lit(x, true)` is "x holds".
pub struct Lit<X>(x: X, value: Bool):
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

    /// At least one of `a`, `b` holds.
    pub fn add_or(mut self, a: Lit<X>, b: Lit<X>):
        self.clause(lit_node(a), lit_node(b))

    /// `a` holds.
    pub fn must(mut self, a: Lit<X>):
        self.add_or(a, a)

    /// If `a` holds then `b` holds.
    pub fn implies(mut self, a: Lit<X>, b: Lit<X>):
        self.add_or(a.flip, b)

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
