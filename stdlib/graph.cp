//! Graphs: one stored representation and the algorithms over "arcs out of v".
//!
//! Vertices and edges are domains, so per-vertex data is `[V]T`, per-edge
//! data is `[E]T`, and mixing the two does not type-check. An algorithm
//! takes `adj`, any fn from a vertex to its arcs, where an arc is a record
//! with a `to` field. A stored `Graph`, a `[V]List<...>` and a lambda over
//! implicit states all fit:
//!
//!     bfs(g.adj, from: [s])
//!     bfs(u => lists[u], from: [s])
//!     bfs(u => [(to: v) for v in g.adj(u) if v.to != blocked], from: [s])
//!
//! Weights are read through a fn of the arc (`w: a => edges[a.e].w`), so
//! the same graph serves several weightings. Unreachable is `none`.
use std.algebra.{Order}
use std.heap.{Heap}

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
pub fn bfs<V, A>(adj: V -> Iter<A>, from: List<V>) -> [V]?Int:
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
        for a in adj(u):
            if dist[a.to] == none:
                dist[a.to] = dist[u]! + 1
                queue.push(a.to)
    dist

/// Shortest distances for non-negative weights, from `(vertex, starting distance)` pairs.
/// O((n + m) log m) with a binary heap.
pub fn dijkstra<V, A, W>(adj: V -> Iter<A>, from: List<(V, W)>, w: A -> W) -> [V]?W:
    shortest(adj, from, w).dist

/// `dijkstra` that also records how each vertex was reached: `via[v]` is the
/// vertex before `v` on a shortest path, `none` at a start. See `walk_back`.
pub fn shortest<V, A, W>(adj: V -> Iter<A>, from: List<(V, W)>, w: A -> W) -> (dist: [V]?W, via: [V]?V):
    var dist: [V]?W = none
    var via: [V]?V = none
    var heap = Heap<(W, V)>(Order<(W, V)>((a, b) => a.0 < b.0))
    for (s, d) in from:
        if dist[s] == none or d < dist[s]!:
            dist[s] = d
            heap.push((d, s))
    while let (d, u) = heap.pop():
        if dist[u]! < d: continue
        for a in adj(u):
            let nd = d + w(a)
            if dist[a.to] == none or nd < dist[a.to]!:
                dist[a.to] = nd
                via[a.to] = u
                heap.push((nd, a.to))
    (dist: dist, via: via)

/// The vertices from a start to `t`, following `via` backwards. O(path length).
pub fn walk_back<V>(via: [V]?V, t: V) -> List<V>:
    var path = [t]
    while let p = via[path.last]: path.push(p)
    path.reverse()
    path

/// A topological order, or `none` if there is a cycle. O(n + m).
pub fn topo<V, A>(adj: V -> Iter<A>) -> ?List<V>:
    var indeg: [V]Int = 0
    for u in V:
        for a in adj(u): indeg[a.to] += 1
    var order = [v for v in V if indeg[v] == 0]
    var head = 0
    while head < order.len:
        let u = order[head]
        head += 1
        for a in adj(u):
            indeg[a.to] -= 1
            if indeg[a.to] == 0: order.push(a.to)
    if order.len == V.K: order else: none

/// Strongly connected components. `C` is a fresh domain, numbered so that
/// every arc goes from a smaller or equal component to a larger or equal
/// one (a topological order of the condensation). O(n + m), no recursion.
pub fn scc<V, A>(adj: V -> Iter<A>) -> (domain C, comp: [V]C):
    let out = [V] v => [a.to for a in adj(v)]
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
            let (u, i) = path.last
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
                if not path.is_empty: low[path.last.0] min= low[u]
    domain C = 0..k
    (C: C, comp: [V] v => C(k - 1 - done[v]!))

/// Connected components of an undirected graph, as a fresh domain `C`. O(n + m).
pub fn components<V, A>(adj: V -> Iter<A>) -> (domain C, comp: [V]C):
    var id: [V]?Int = none
    var k = 0
    for root in V:
        if id[root] != none: continue
        id[root] = k
        var queue = [root]
        while not queue.is_empty:
            let u = queue.pop()
            for a in adj(u):
                if id[a.to] == none:
                    id[a.to] = k
                    queue.push(a.to)
        k += 1
    domain C = 0..k
    (C: C, comp: [V] v => C(id[v]!))

/// The condensation: the arcs between components, as adjacency lists (duplicates kept).
pub fn condense<V, A, C>(adj: V -> Iter<A>, comp: [V]C) -> [C]List<(to: C)>:
    var out: [C]List<(to: C)> = []
    for u in V:
        for a in adj(u):
            if comp[u] != comp[a.to]: out[comp[u]].push((to: comp[a.to]))
    out

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
    let d = bfs(g.adj, from: [V(0)])
    assert [d[v] for v in V] == [0, 1, 2, 1, 3, none]
    // an implicit graph: the same vertices on a ring
    let ring = bfs(u => [(to: u.cyc(1))], from: [V(4), V(1)])
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
            let r = shortest(g.adj, from: [(s, 0)], w: a => edges[a.e].w)
            for t in V:
                assert r.dist[t] == best[s, t]
                if r.dist[t] != none:
                    let path = walk_back(r.via, t)
                    assert path.first == s and path.last == t

@test
fn topo_orders_a_dag_and_rejects_a_cycle():
    domain V = 0..6
    let out = [V] v => [(to: V(x)) for x in [[1, 2], [3], [3], [4], [], [0]][v.to_idx]]
    let order = topo::<V>(v => out[v])!
    var pos: [V]Int = 0
    for (i, v) in [(i, order[i]) for i in 0..order.len]: pos[v] = i
    for u in V:
        for a in out[u]: assert pos[u] < pos[a.to]
    let cyc = [V] v => [(to: v.cyc(1))]
    assert topo::<V>(v => cyc[v]) == none

@test
fn scc_agrees_with_reachability():
    domain V = 0..8
    for _ in 0..25:
        let out = [V] v => [(to: V.nth(rand(0, 7))) for _ in 0..rand(0, 2)]
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
    let (C, comp) = components::<V>(g.adj)
    assert C.K == 3
    var size: [C]Int = 0
    for v in V: size[comp[v]] += 1
    assert sorted(size[c] for c in C) == [1, 2, 3]
    assert comp[V(1)] == comp[V(3)] and comp[V(4)] != comp[V(5)]
