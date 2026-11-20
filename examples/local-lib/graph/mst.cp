//! Minimum spanning forest by Kruskal's algorithm.
use std.dsu.{Dsu}

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
