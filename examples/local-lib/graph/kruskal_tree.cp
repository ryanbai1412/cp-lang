//! Kruskal reconstruction tree (`KruskalTree<V>`) and spanning forest helper (`spanning`).

/// Pick a spanning forest from unweighted edges `edges: List<(V, V)>`: returns the subset of
/// edges that connect components without creating cycles. O(m alpha(n)).
use std.dsu.{Dsu}
use std.graph.{bfs}
pub fn spanning<V>(edges: List<(V, V)>) -> List<(V, V)>:
    var dsu = Dsu<V>()
    var out: List<(V, V)> = []
    for (u, v) in edges:
        if dsu.unite(u, v): out.push((u, v))
    out

/// Kruskal reconstruction (reachability / bottleneck) tree over weighted undirected edges.
///
/// Original vertices occupy node ids `0..V.K`; each merged MST edge creates an internal node
/// `V.K + k` with weight `w` whose children are the two merged component roots.
/// Because internal node weights increase upward, all vertices reachable from `v` using only
/// edges of weight `<= limit` form the leaves of a single subtree, found in O(log n) by lifting.
pub struct KruskalTree<V>:
    /// Total nodes (`V.K` leaves + `<= V.K - 1` internal edge nodes).
    n_nodes: Int
    /// Weight of internal node `id` (0 for leaf `0..V.K`).
    weight: List<Int>
    /// Binary-lifting ancestor table `up[k][id]`.
    up: List<List<Int>>
    /// Euler tour leaf intervals `[tin[id], tout[id])` in `leaf_order`.
    tin: List<Int>
    tout: List<Int>
    /// Leaves in Euler-tour order (so `leaf_order[tin[id]..tout[id]]` are all vertices in `id`'s component).
    leaf_order: List<V>

    pub fn new(edges: List<(Int, V, V)>) -> KruskalTree<V>:
        let n = V.K
        let sorted_e = sorted(edges, by: e => e.0)
        var dsu = Dsu<V>()
        var comp_root = [i for i in 0..n]
        var weight = List.filled(n, 0)
        var children: List<List<Int>> = [[] for _ in 0..n]
        var par = [i for i in 0..n]
        for (w, u, v) in sorted_e:
            let ru = dsu.find(u).to_idx
            let rv = dsu.find(v).to_idx
            if ru == rv: continue
            let id = weight.len
            weight.push(w)
            children.push([comp_root[ru], comp_root[rv]])
            par.push(id)
            par[comp_root[ru]] = id
            par[comp_root[rv]] = id
            dsu.unite(u, v)
            comp_root[dsu.find(u).to_idx] = id
        let tot = weight.len
        // Build binary lifting table
        var up = [par]
        var step = 1
        while step < tot:
            let prev = up.last!
            up.push([prev[prev[i]] for i in 0..tot])
            step *= 2
        // Iterative DFS over every component root to assign contiguous leaf ranges
        var tin = List.filled(tot, 0)
        var tout = List.filled(tot, 0)
        var leaf_order: List<V> = []
        for r in 0..tot:
            if par[r] != r: continue
            var stack = [(r, 0)]
            while not stack.is_empty:
                let (u, idx) = stack.last!
                if idx == 0: tin[u] = leaf_order.len
                if idx < children[u].len:
                    stack.last! = (u, idx + 1)
                    stack.push((children[u][idx], 0))
                else:
                    stack.pop()
                    if u < n: leaf_order.push(V.nth(u))
                    tout[u] = leaf_order.len
        KruskalTree(tot, weight, up, tin, tout, leaf_order)

    /// Highest ancestor of `v` whose edge weight is `<= limit`. O(log n).
    pub fn lift(self, v: V, limit: Int) -> Int:
        var u = v.to_idx
        for k in (0..up.len).rev:
            let p = up[k][u]
            if weight[p] <= limit: u = p
        u

    /// Half-open slice range `lo..hi` in `leaf_order` containing every vertex reachable from `v`
    /// using only edges with weight `<= limit`. O(log n).
    pub fn reachable_range(self, v: V, limit: Int) -> Range<Int>:
        let u = self.lift(v, limit)
        tin[u]..tout[u]

    /// All vertices reachable from `v` using only edges with weight `<= limit`.
    pub fn reachable(self, v: V, limit: Int) -> List<V>:
        let r = self.reachable_range(v, limit)
        [leaf_order[i] for i in r]

    /// Minimum possible maximum edge weight (bottleneck) on any path between `a` and `b`,
    /// or `none` if disconnected. O(log n).
    pub fn bottleneck(self, a: V, b: V) -> ?Int:
        if a == b: return 0
        var u = a.to_idx
        let v = b.to_idx
        let top = up.len - 1
        if up[top][u] != up[top][v]: return none
        // Climb to the LCA of u and v (the first ancestor whose leaf range contains both)
        for k in (0..up.len).rev:
            let p = up[k][u]
            if not (tin[p] <= tin[v] and tout[v] <= tout[p]): u = p
        weight[up[0][u]]

@test
fn spanning_and_kruskal_tree_match_bfs_and_minimax_paths():
    for _ in 0..20:
        let n = rand(2, 10)
        domain V = 0..n
        var raw_edges: List<(V, V)> = []
        var edges: List<(Int, V, V)> = []
        for _ in 0..rand(0, n * 2):
            let u = V.nth(rand(0, n - 1))
            let v = V.nth(rand(0, n - 1))
            let w = rand(1, 20)
            raw_edges.push((u, v))
            edges.push((w, u, v))
        // spanning preserves exact connectivity with zero cycles
        let forest = spanning::<V>(raw_edges)
        var d1 = Dsu<V>()
        for (u, v) in raw_edges: d1.unite(u, v)
        var d2 = Dsu<V>()
        for (u, v) in forest: assert d2.unite(u, v)
        for u in V:
            for v in V: assert d1.same(u, v) == d2.same(u, v)
        // KruskalTree reachable & bottleneck match BFS on weight-filtered graph
        let kt = KruskalTree<V>(edges)
        for limit in [0, 5, 10, 15, 25]:
            var adj: [V]List<V> = []
            for (w, u, v) in edges:
                if w <= limit:
                    adj[u].push(v)
                    adj[v].push(u)
            for s in V:
                let dist = bfs(u => adj[u], [s])
                let want = sorted([u for u in V if dist[u] != none])
                let got = sorted(kt.reachable(s, limit))
                assert got == want
        for a in V:
            for b in V:
                let bn = kt.bottleneck(a, b)
                if not d1.same(a, b):
                    assert bn == none
                else:
                    assert bn != none
                    assert b in kt.reachable(a, bn!)
                    if bn! > 0: assert b not in kt.reachable(a, bn! - 1)
