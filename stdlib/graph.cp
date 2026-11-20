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

/// What an algorithm walks: a `Graph` gives its arcs out of `u`, and any fn `u => ...` is
/// simply called.
pub trait Neighbours:
    fn out<V>(self, u: V) -> Iter<V> = self(u)

/// What a neighbour list holds: an arc says where it leads with its `to` field, and a plain
/// vertex leads to itself.
pub trait Target:
    @get fn to(self) -> Self = self

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

    /// The arcs out of `u`, in edge order.
    pub fn adj(self, u: V) -> yields (to: V, e: E):
        for i in start[u.to_idx]..start[u.to_idx + 1]:
            yield arcs[i]

    /// The arcs out of `u`: what makes a `Graph` usable wherever `next` is expected.
    pub fn out(self, u: V) -> yields (to: V, e: E):
        for i in start[u.to_idx]..start[u.to_idx + 1]:
            yield arcs[i]

    /// `g[u]`: the arcs out of `u`, as a list.
    pub fn get(self, u: V) -> List<(to: V, e: E)> = [arcs[i] for i in start[u.to_idx]..start[u.to_idx + 1]]

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
            for a in self.adj(u):
                out[at[a.to.to_idx]] = (to: u, e: a.e)
                at[a.to.to_idx] += 1
        Graph(back, out)

/// Fewest arcs from any of `from` to each vertex. O(n + m).
pub fn bfs<V, N>(next: N, from: List<V>) -> [V]?Int:
    var dist: [V]?Int = none
    var queue = List<V>()
    for s in from:
        if dist[s] == none:
            dist[s] = 0
            queue.push(s)
    var head = 0
    while head < queue.len:
        let u = queue[head]
        head += 1
        for a in next.out(u):
            if dist[a.to] == none:
                dist[a.to] = dist[u]! + 1
                queue.push(a.to)
    dist

/// Shortest distances for non-negative weights, from `(vertex, starting distance)` pairs.
/// O((n + m) log m) with a binary heap.
pub fn dijkstra<V, N, A, W>(next: N, from: List<(V, W)>, w: A -> W) -> [V]?W:
    shortest(next, from, w).dist

/// `dijkstra` that also records how each vertex was reached: `via[v]` is the
/// vertex before `v` on a shortest path, `none` at a start. See `walk_back`.
pub fn shortest<V, N, A, W>(next: N, from: List<(V, W)>, w: A -> W) -> (dist: [V]?W, via: [V]?V):
    var dist: [V]?W = none
    var via: [V]?V = none
    var heap = MinHeap<(W, V)>()
    for (s, d) in from:
        if dist[s] == none or d < dist[s]!:
            dist[s] = d
            heap.push((d, s))
    while let (d, u) = heap.pop():
        if dist[u]! < d: continue
        for a in next.out(u):
            let nd = d + w(a)
            if dist[a.to] == none or nd < dist[a.to]!:
                dist[a.to] = nd
                via[a.to] = u
                heap.push((nd, a.to))
    (dist: dist, via: via)

/// The vertices from a start to `t`, following `via` backwards. O(path length).
pub fn walk_back<V>(via: [V]?V, t: V) -> List<V>:
    var path = [t]
    while let p = via[path[path.len - 1]]: path.push(p)
    path.reverse()
    path

/// A topological order, or `none` if there is a cycle. O(n + m).
pub fn topo<V, N>(next: N) -> ?List<V>:
    let order = topo_prefix::<V>(next)
    if order.len == V.K: order else: none

/// As much of a topological order as exists: every vertex that is not on a cycle and not
/// reachable from one, each after everything that leads to it. O(n + m).
pub fn topo_prefix<V, N>(next: N) -> List<V>:
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

/// Strongly connected components. `C` is a fresh domain, numbered so that
/// every arc goes from a smaller or equal component to a larger or equal
/// one (a topological order of the condensation). O(n + m), no recursion.
pub fn scc<V, N>(next: N) -> (domain C, comp: [V]C):
    let out = [V] v => [a.to for a in next.out(v)]
    var seen: [V]?Int = none        // discovery time
    var low: [V]Int = 0
    var done: [V]?Int = none        // component, in order of completion
    var stack = List<V>()
    var path = List<(V, Int)>()     // the dfs: a vertex and its next arc
    var (timer, k) = (0, 0)
    for root in V:
        if seen[root] != none: continue
        path.push((root, 0))
        while not path.is_empty:
            let (u, i) = path[path.len - 1]
            if i == 0:
                seen[u] = timer
                low[u] = timer
                timer += 1
                stack.push(u)
            if i < out[u].len:
                path[path.len - 1] = (u, i + 1)
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
                if not path.is_empty: low[path[path.len - 1].0] min= low[u]
    domain C = 0..k
    (C: C, comp: [V] v => C(k - 1 - done[v]!))

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
            let (u, i, pe) = frames[frames.len - 1]
            if i == 0:
                tin[u] = timer
                low[u] = timer
                timer += 1
                stack.push(u)
            if i < out[u].len:
                frames[frames.len - 1] = (u, i + 1, pe)
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
                    let p = frames[frames.len - 1].0
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
            let (u, i, p) = frames[frames.len - 1]
            if i == 0:
                tin[u] = timer
                low[u] = timer
                timer += 1
            if i < out[u].len:
                frames[frames.len - 1] = (u, i + 1, p)
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
            let (u, i, p) = frames[frames.len - 1]
            if i == 0:
                tin[u] = timer
                low[u] = timer
                timer += 1
                stack.push(u)
            if i < out[u].len:
                frames[frames.len - 1] = (u, i + 1, p)
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

/// Compressed (virtual / auxiliary) tree induced by `keys` inside `tree`:
/// contains every vertex in `keys` together with all pairwise LCAs, linked by
/// directed parent-to-child edges in `tree`. O(k log k + k log n).

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

/// Rollback union-find that tracks bipartiteness (whether any odd cycle exists).
/// `unite` is O(log n), `undo` is O(1). Each `unite` pushes one log record so
/// `rewind(mark)` restores the exact earlier state, including `is_bipartite`.

/// A minimum spanning forest by Kruskal's algorithm: the edges kept, in increasing weight,
/// and their total. O(m log m).
pub fn mst<V, E, W>(edges: [E](u: V, v: V, w: W)) -> (total: W, used: List<E>):
    var order = [e for e in E]
    sort(mut order, by: e => edges[e].w)
    var d = Dsu<V>()
    var used: List<E> = []
    var total = W.zero
    for e in order:
        if d.unite(edges[e].u, edges[e].v):
            used.push(e)
            total = total + edges[e].w
    (total: total, used: used)

/// A variable or its negation: `Lit(x, true)` is "x holds".
pub struct Lit<X>(x: X, value: Bool):
    /// The opposite literal.
    pub @get fn flip(self) -> Lit<X> = Lit(x, not value)

/// A conjunction of two-literal clauses over the variables `X`.
/// `solve` is O(variables + clauses).
pub struct TwoSat<X>:
    /// Implications between literal nodes: node `2 * i` is "variable i is false", `2 * i + 1` "is true".
    imp: List<List<Int>>

    pub fn new() -> TwoSat<X>:
        TwoSat([[] for _ in 0..2 * X.K])

    /// At least one of `a`, `b` holds.
    pub fn add_or(mut self, a: Lit<X>, b: Lit<X>):
        imp[lit_node(a.flip)].push(lit_node(b))
        imp[lit_node(b.flip)].push(lit_node(a))

    /// `a` holds.
    pub fn must(mut self, a: Lit<X>):
        self.add_or(a, a)

    /// If `a` holds then `b` holds.
    pub fn implies(mut self, a: Lit<X>, b: Lit<X>):
        self.add_or(a.flip, b)

    /// An assignment satisfying every clause, or `none` if there is none.
    pub fn solve(self) -> ?[X]Bool:
        domain N = 0..2 * X.K
        let (C, comp) = scc::<N>(u => [N(v) for v in imp[u.to_idx]])
        // components are in topological order: a literal is true when it comes after its negation
        for x in X:
            if comp[N(2 * x.to_idx)] == comp[N(2 * x.to_idx + 1)]: return none
        [X] x => comp[N(2 * x.to_idx + 1)] > comp[N(2 * x.to_idx)]

fn lit_node<X>(l: Lit<X>) -> Int = 2 * l.x.to_idx + (if l.value: 1 else: 0)

@test
fn csr_keeps_edge_ids_and_order():
    domain V = 1..=4
    domain E = 1..=4
    let edges = [E] e => [(u: V(1), v: V(2)), (u: V(1), v: V(3)), (u: V(3), v: V(2)), (u: V(4), v: V(1))][e.to_idx]
    let g = Graph.directed(edges)
    assert [a.to.label for a in g.adj(V(1))] == [2, 3]
    assert [a.e.label for a in g.adj(V(1))] == [1, 2]
    assert g.deg(V(2)) == 0 and g.deg(V(4)) == 1
    let t = g.transpose()
    assert [a.to.label for a in t.adj(V(2))] == [1, 3]
    assert [a.e.label for a in t.adj(V(2))] == [1, 3]
    let u = Graph.undirected(edges)
    assert [a.to.label for a in u.adj(V(1))] == [2, 3, 4]
    assert sum(u.deg(v) for v in V) == 8

@test
fn bfs_over_a_stored_graph_and_over_a_lambda():
    domain V = 0..6
    domain E = 0..5
    let edges = [E] e => [(u: V(0), v: V(1)), (u: V(1), v: V(2)), (u: V(0), v: V(3)), (u: V(3), v: V(2)), (u: V(2), v: V(4))][e.to_idx]
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
            let r = shortest(g, from: [(s, 0)], w: a => edges[a.e].w)
            for t in V:
                assert r.dist[t] == best[s, t]
                if r.dist[t] != none:
                    let path = walk_back(r.via, t)
                    assert path[0] == s and path[path.len - 1] == t

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
    // 5 -> 0 -> {1, 2} -> 3 -> 4 with 4 -> 3 added: only what comes before the cycle is ordered
    let stuck = [V] v => [V(x) for x in [[1, 2], [3], [3], [4], [3], [0]][v.to_idx]]
    assert topo_prefix::<V>(v => stuck[v]) == [V(5), V(0), V(1), V(2)]

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
                let robust = all(linked(edges, a, b, e, none) for e in E) and linked(edges, a, b, none, none)
                assert (comp[a] == comp[b]) == robust
        for v in V:
            var splits = false
            for a in V:
                for b in V:
                    if a != v and b != v and linked(edges, a, b, none, none) and not linked(edges, a, b, none, v): splits = true
            assert cut[v] == splits

@test
fn blocks_of_a_small_graph():
    // two triangles sharing vertex 2, and a pendant edge 4 - 5
    domain V = 0..6
    domain E = 0..7
    let edges = [E] e => [(u: V(0), v: V(1)), (u: V(1), v: V(2)), (u: V(2), v: V(0)), (u: V(2), v: V(3)), (u: V(3), v: V(4)), (u: V(4), v: V(2)), (u: V(4), v: V(5))][e.to_idx]
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
            if all(((mask >> edges[e].u.to_idx) & 1) != ((mask >> edges[e].v.to_idx) & 1) for e in E): possible = true
        assert (side != none) == possible
        if side != none:
            for e in E: assert side![edges[e].u] != side![edges[e].v]

@test
fn mst_matches_trying_every_subset():
    domain V = 0..5
    domain E = 0..7
    for _ in 0..30:
        let edges = [E] e => (u: V.nth(rand(0, 4)), v: V.nth(rand(0, 4)), w: rand(1, 20))
        let got = mst(edges)
        var parts = Dsu<V>()
        for e in E: parts.unite(edges[e].u, edges[e].v)
        var best: ?Int = none
        for mask in 0..128:
            var d = Dsu<V>()
            var (cost, cyclic) = (0, false)
            for e in E:
                if ((mask >> e.to_idx) & 1) == 1:
                    if not d.unite(edges[e].u, edges[e].v): cyclic = true
                    cost += edges[e].w
            if not cyclic and d.components == parts.components: best min= cost
        assert got.total == best and got.used.len == V.K - parts.components

@test
fn two_sat_solutions_satisfy_every_clause_and_unsat_is_detected():
    domain X = 0..5
    for _ in 0..60:
        var sat = TwoSat<X>()
        var clauses: List<(Lit<X>, Lit<X>)> = []
        for _ in 0..rand(1, 12):
            let a = Lit(X.nth(rand(0, 4)), rand(0, 1) == 1)
            let b = Lit(X.nth(rand(0, 4)), rand(0, 1) == 1)
            sat.add_or(a, b)
            clauses.push((a, b))
        var possible = false
        for mask in 0..32:
            if all((((mask >> c.0.x.to_idx) & 1) == 1) == c.0.value or (((mask >> c.1.x.to_idx) & 1) == 1) == c.1.value for c in clauses): possible = true
        let got = sat.solve()
        assert (got != none) == possible
        if got != none:
            for c in clauses: assert got![c.0.x] == c.0.value or got![c.1.x] == c.1.value
    var s = TwoSat<X>()
    s.must(Lit(X(0), true))
    s.implies(Lit(X(0), true), Lit(X(3), false))
    let fixed = s.solve()!
    assert fixed[X(0)] and not fixed[X(3)]
