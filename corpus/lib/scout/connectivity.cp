//! Undirected connectivity helpers: 2-edge-connected components (bridges),
//! vertex-biconnected components (blocks), articulation points, bipartite
//! and 2-coloring; the rollback bipartite DSU is
//! `std.dsu.ParityDsu`. Proposed additions to `std.graph` and `std.tree`.
use std.dsu.{ParityDsu}

/// Two-edge-connected components (bridge blocks) of an undirected graph.
/// Each arc in `adj(u)` carries `(to: V, e: E)` so parallel edges are handled
/// properly (parallel edges are never bridges). Returns a fresh domain `C`
/// for the components and `comp: [V]C`. Every edge between distinct components
/// is a bridge, so `std.graph.condense(adj, comp)` yields the bridge tree.
/// O(n + m), iterative Tarjan low-link.
pub fn two_edge_components<V, A>(adj: V -> Iter<A>) -> (domain C, comp: [V]C):
    let out = [V] v => [a for a in adj(v)]
    var tin: [V]?Int = None
    var low: [V]Int = 0
    var cid: [V]?Int = None
    var stack = List<V>()
    // (u, next arc index, parent edge id or none)
    var frames = List<(V, Int, ?Int)>()
    var (timer, k) = (0, 0)
    for root in V:
        if tin[root] != None: continue
        frames.push((root, 0, None))
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
                let eid = a.e.ord
                if pe == Some(eid): continue
                let v = a.to
                if tin[v] == None:
                    frames.push((v, 0, eid))
                else:
                    low[u] min= tin[v]!
            else:
                frames.pop()
                if not frames.is_empty:
                    let p = frames.last.0
                    low[p] min= low[u]
                if low[u] == tin[u]!:
                    while true:
                        let v = stack.pop()!
                        cid[v] = k
                        if v == u: break
                    k += 1
    domain C = 0..k
    (C:, comp: [V] v => C(cid[v]!))

/// Articulation points (cut vertices) of an undirected simple graph.
/// Handles disconnected graphs and root child count. O(n + m), iterative.
pub fn articulation_points<V, A>(adj: V -> Iter<A>) -> [V]Bool:
    let out = [V] v => [a.to for a in adj(v)]
    var tin: [V]?Int = None
    var low: [V]Int = 0
    var is_cut: [V]Bool = false
    var children: [V]Int = 0
    // (u, next neighbor index, parent vertex or none)
    var frames = List<(V, Int, ?V)>()
    var timer = 0
    for root in V:
        if tin[root] != None: continue
        frames.push((root, 0, None))
        while not frames.is_empty:
            let (u, i, p) = frames.last!
            if i == 0:
                tin[u] = timer
                low[u] = timer
                timer += 1
            if i < out[u].len:
                frames.last! = (u, i + 1, p)
                let v = out[u][i]
                if p == Some(v): continue
                if tin[v] == None:
                    children[u] += 1
                    frames.push((v, 0, u))
                else:
                    low[u] min= tin[v]!
            else:
                frames.pop()
                if p == None:
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
pub fn biconnected_components<V, A>(adj: V -> Iter<A>) -> List<List<V>>:
    let out = [V] v => [a.to for a in adj(v)]
    var tin: [V]?Int = None
    var low: [V]Int = 0
    var stack = List<V>()
    var blocks = List<List<V>>()
    // (u, next neighbor index, parent vertex or none)
    var frames = List<(V, Int, ?V)>()
    var timer = 0
    for root in V:
        if tin[root] != None: continue
        frames.push((root, 0, None))
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
                if p == Some(v): continue
                if tin[v] == None:
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
                            let w = stack.pop()!
                            comp.push(w)
                            if w == u: break
                        comp.push(par)
                        blocks.push(comp)
                else:
                    stack.pop()
    blocks

/// Bipartite 2-coloring of an undirected graph. Returns `none` if an odd cycle
/// exists, otherwise a coloring `side: [V]Bool` with `side[u] != side[v]` on
/// every edge. O(n + m).
pub fn two_color<V, A>(adj: V -> Iter<A>) -> ?[V]Bool:
    var side: [V]?Bool = None
    for root in V:
        if side[root] != None: continue
        side[root] = false
        var q = [root]
        var head = 0
        while head < q.len:
            let u = q[head]
            head += 1
            let next_c = not side[u]!
            for a in adj(u):
                let v = a.to
                if side[v] == None:
                    side[v] = next_c
                    q.push(v)
                elif side[v]! != next_c:
                    return None
    [V] v => side[v]!

@test
fn bridges_and_parallel_edges():
    domain V = 0..5
    domain E = 0..6
    // 0-1-2 triangle, bridge 2-3, double edge 3-4
    let raw = [(0, 1), (1, 2), (2, 0), (2, 3), (3, 4), (3, 4)]
    let edges = [E] e => (u: V(raw[e.ord].0), v: V(raw[e.ord].1))
    let g = Graph<V, E>.undirected(edges)
    let (C, comp) = two_edge_components<V>(u => g.out(u))
    assert C.count == 2
    assert comp[V(0)] == comp[V(1)] and comp[V(1)] == comp[V(2)]
    assert comp[V(3)] == comp[V(4)] and comp[V(2)] != comp[V(3)]
    let tree = condense(u => g.out(u), comp)
    assert tree[comp[V(2)]].len == 1 and tree[comp[V(3)]].len == 1

@test
fn cut_vertices_with_root_and_leaf():
    domain V = 0..5
    domain E = 0..6
    // bowtie: 0-1-2-0 and 2-3-4-2; only vertex 2 is an articulation point
    let raw = [(0, 1), (1, 2), (2, 0), (2, 3), (3, 4), (4, 2)]
    let edges = [E] e => (u: V(raw[e.ord].0), v: V(raw[e.ord].1))
    let g = Graph<V, E>.undirected(edges)
    let cut = articulation_points<V>(u => g.out(u))
    assert [cut[v] for v in V] == [false, false, true, false, false]

@test
fn coloring_and_bipartite_dsu_rollback():
    domain V = 0..4
    domain E = 0..3
    let edges = [E] e => (
        u: V([(0, 1), (1, 2), (2, 3)][e.ord].0),
        v: V([(0, 1), (1, 2), (2, 3)][e.ord].1),
    )
    let g = Graph<V, E>.undirected(edges)
    let col = two_color<V>(u => g.out(u))!
    assert col[V(0)] != col[V(1)] and col[V(1)] != col[V(2)] and col[V(2)] != col[V(3)]

    var d = ParityDsu<V>()
    d.unite(V(0), V(1))
    d.unite(V(1), V(2))
    let mark = d.mark()
    assert d.consistent
    d.unite(V(2), V(0))
    assert not d.consistent
    d.undo_to(mark)
    assert d.consistent
    d.unite(V(2), V(3))
    d.unite(V(3), V(0))
    assert d.consistent

@test
fn bcc_sizes():
    domain V = 0..6
    domain E = 0..7
    // 0-1-2 triangle, bridge 2-3, triangle 3-4-5
    let raw = [(0, 1), (1, 2), (2, 0), (2, 3), (3, 4), (4, 5), (5, 3)]
    let edges = [E] e => (u: V(raw[e.ord].0), v: V(raw[e.ord].1))
    let g = Graph<V, E>.undirected(edges)
    let blocks = biconnected_components<V>(u => g.out(u))
    var sizes = [b.len for b in blocks]
    sizes.sort()
    assert sizes == [2, 3, 3]
