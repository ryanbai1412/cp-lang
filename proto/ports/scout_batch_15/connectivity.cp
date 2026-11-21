//! Undirected connectivity helpers: 2-edge-connected components (bridges),
//! vertex-biconnected components (blocks), articulation points, bipartite
//! 2-coloring, rollback bipartite DSU, and virtual trees on `Rooted<V>`.
//! Proposed additions to `std.graph`, `std.tree`, and `std.dsu`.

/// Two-edge-connected components (bridge blocks) of an undirected graph.
/// Each arc in `adj(u)` carries `(to: V, e: E)` so parallel edges are handled
/// properly (parallel edges are never bridges). Returns a fresh domain `C`
/// for the components and `comp: [V]C`. Every edge between distinct components
/// is a bridge, so `std.graph.condense(adj, comp)` yields the bridge tree.
/// O(n + m), iterative Tarjan low-link.
pub fn two_edge_components<V, A>(adj: V -> Iter<A>) -> (domain C, comp: [V]C):
    let out = [V] v => [a for a in adj(v)]
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
            let (u, i, pe) = frames.last
            if i == 0:
                tin[u] = timer
                low[u] = timer
                timer += 1
                stack.push(u)
            if i < out[u].len:
                frames.last! = (u, i + 1, pe)
                let a = out[u][i]
                let eid = a.e.to_idx
                if pe == some(eid): continue
                let v = a.to
                if tin[v] == none:
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
                        let v = stack.pop()
                        cid[v] = k
                        if v == u: break
                    k += 1
    domain C = 0..k
    (C: C, comp: [V] v => C(cid[v]!))

/// Articulation points (cut vertices) of an undirected simple graph.
/// Handles disconnected graphs and root child count. O(n + m), iterative.
pub fn articulation_points<V, A>(adj: V -> Iter<A>) -> [V]Bool:
    let out = [V] v => [a.to for a in adj(v)]
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
            let (u, i, p) = frames.last
            if i == 0:
                tin[u] = timer
                low[u] = timer
                timer += 1
            if i < out[u].len:
                frames.last! = (u, i + 1, p)
                let v = out[u][i]
                if p == some(v): continue
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
pub fn biconnected_components<V, A>(adj: V -> Iter<A>) -> List<List<V>>:
    let out = [V] v => [a.to for a in adj(v)]
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
            let (u, i, p) = frames.last
            if i == 0:
                tin[u] = timer
                low[u] = timer
                timer += 1
                stack.push(u)
            if i < out[u].len:
                frames.last! = (u, i + 1, p)
                let v = out[u][i]
                if p == some(v): continue
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
pub fn virtual_tree<V>(tree: Rooted<V>, keys: List<V>) -> (
    nodes: List<V>,
    edges: List<(u: V, v: V)>,
):
    if keys.is_empty: return (nodes: [], edges: [])
    var by_tin = [(tree.tin[v].ord, v) for v in keys]
    by_tin.sort()
    var uniq = List<V>()
    for p in by_tin:
        if uniq.is_empty or uniq.last != p.1: uniq.push(p.1)
    var all = [(tree.tin[v].ord, v) for v in uniq]
    for i in 0..uniq.len - 1:
        let l = tree.lca(uniq[i], uniq[i + 1])
        all.push((tree.tin[l].ord, l))
    all.sort()
    var nodes = List<V>()
    for p in all:
        if nodes.is_empty or nodes.last != p.1: nodes.push(p.1)
    var edges: List<(u: V, v: V)> = []
    var st = [nodes[0]]
    for i in 1..nodes.len:
        let u = nodes[i]
        while not tree.is_anc(st.last, u): st.pop()
        edges.push((u: st.last, v: u))
        st.push(u)
    (nodes: nodes, edges: edges)

/// Bipartite 2-coloring of an undirected graph. Returns `none` if an odd cycle
/// exists, otherwise a coloring `side: [V]Bool` with `side[u] != side[v]` on
/// every edge. O(n + m).
pub fn two_color<V, A>(adj: V -> Iter<A>) -> ?[V]Bool:
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
            for a in adj(u):
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
pub struct BipartiteDsu<V>:
    up: [V]V
    sz: [V]Int
    diff: [V]Bool
    ok: Bool
    log: List<(prev_ok: Bool, child: ?V, root: V)>

    pub fn new() -> BipartiteDsu<V>:
        BipartiteDsu([V] v => v, [V] v => 1, [V] v => false, true, [])

    pub @get fn is_bipartite(self) -> Bool = ok

    pub @get fn depth(self) -> Int = log.len

    /// Returns `(root, parity_to_root)` without path compression.
    pub fn find(self, v: V) -> (V, Bool):
        var x = v
        var p = false
        while up[x] != x:
            p = p != diff[x]
            x = up[x]
        (x, p)

    /// Add an undirected edge `(a, b)` requiring `a` and `b` to have opposite colors.
    pub fn unite(mut self, a: V, b: V):
        let (rx, px) = self.find(a)
        let (ry, py) = self.find(b)
        if rx == ry:
            log.push((prev_ok: ok, child: none, root: rx))
            if px == py: ok = false
            return
        var (big, small) = (rx, ry)
        if sz[big] < sz[small]: swap(mut big, mut small)
        log.push((prev_ok: ok, child: small, root: big))
        up[small] = big
        sz[big] += sz[small]
        diff[small] = px == py

    pub fn undo(mut self):
        let step = log.pop()
        ok = step.prev_ok
        if let c = step.child:
            up[c] = c
            sz[step.root] -= sz[c]
            diff[c] = false

    pub fn rewind(mut self, mark: Int):
        while log.len > mark: self.undo()

@test
fn bridges_and_parallel_edges():
    domain V = 0..5
    domain E = 0..6
    // 0-1-2 triangle, bridge 2-3, double edge 3-4
    let raw = [(0, 1), (1, 2), (2, 0), (2, 3), (3, 4), (3, 4)]
    let edges = [E] e => (u: V(raw[e.to_idx].0), v: V(raw[e.to_idx].1))
    let g = Graph<V, E>.undirected(edges)
    let (C, comp) = two_edge_components::<V>(u => g.adj(u))
    assert C.K == 2
    assert comp[V(0)] == comp[V(1)] and comp[V(1)] == comp[V(2)]
    assert comp[V(3)] == comp[V(4)] and comp[V(2)] != comp[V(3)]
    let tree = condense(u => g.adj(u), comp)
    assert tree[comp[V(2)]].len == 1 and tree[comp[V(3)]].len == 1

@test
fn cut_vertices_with_root_and_leaf():
    domain V = 0..5
    domain E = 0..6
    // bowtie: 0-1-2-0 and 2-3-4-2; only vertex 2 is an articulation point
    let raw = [(0, 1), (1, 2), (2, 0), (2, 3), (3, 4), (4, 2)]
    let edges = [E] e => (u: V(raw[e.to_idx].0), v: V(raw[e.to_idx].1))
    let g = Graph<V, E>.undirected(edges)
    let cut = articulation_points::<V>(u => g.adj(u))
    assert [cut[v] for v in V] == [false, false, true, false, false]

@test
fn coloring_and_bipartite_dsu_rollback():
    domain V = 0..4
    domain E = 0..3
    let edges = [E] e => (
        u: V([(0, 1), (1, 2), (2, 3)][e.to_idx].0),
        v: V([(0, 1), (1, 2), (2, 3)][e.to_idx].1),
    )
    let g = Graph<V, E>.undirected(edges)
    let col = two_color::<V>(u => g.adj(u))!
    assert col[V(0)] != col[V(1)] and col[V(1)] != col[V(2)] and col[V(2)] != col[V(3)]

    var d = BipartiteDsu<V>.new()
    d.unite(V(0), V(1))
    d.unite(V(1), V(2))
    let mark = d.depth
    assert d.is_bipartite
    d.unite(V(2), V(0))
    assert not d.is_bipartite
    d.rewind(mark)
    assert d.is_bipartite
    d.unite(V(2), V(3))
    d.unite(V(3), V(0))
    assert d.is_bipartite

@test
fn bcc_and_virtual_tree():
    domain V = 0..6
    domain E = 0..7
    // 0-1-2 triangle, bridge 2-3, triangle 3-4-5
    let raw = [(0, 1), (1, 2), (2, 0), (2, 3), (3, 4), (4, 5), (5, 3)]
    let edges = [E] e => (u: V(raw[e.to_idx].0), v: V(raw[e.to_idx].1))
    let g = Graph<V, E>.undirected(edges)
    let blocks = biconnected_components::<V>(u => g.adj(u))
    var sizes = [b.len for b in blocks]
    sizes.sort()
    assert sizes == [2, 3, 3]

    domain T = 0..6
    domain TE = 0..5
    let traw = [(0, 1), (1, 2), (1, 3), (0, 4), (4, 5)]
    let tedges = [TE] e => (u: T(traw[e.to_idx].0), v: T(traw[e.to_idx].1))
    let tg = Graph<T, TE>.undirected(tedges)
    let tree = Rooted<T>.new(u => tg.adj(u), T(0))
    let vt = virtual_tree(tree, [T(2), T(3), T(5)])
    assert vt.nodes.len == 5  // 0, 1, 2, 3, 5
    assert vt.edges.len == 4
