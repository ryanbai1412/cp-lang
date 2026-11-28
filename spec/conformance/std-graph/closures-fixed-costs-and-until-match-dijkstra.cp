// expect: ok, output "YES"
use std.graph.{fixed_cost_paths}
fn vertexless_closure_graphs_with_type_args():
    domain V = 0..6
    // arcs u -> u + 1 and u -> u + 2, given only as a closure
    let order = peel<V>(
        u => [v for v in V if v.ord == u.ord + 1 or v.ord == u.ord + 2]
    )
    assert order == [V(0), V(1), V(2), V(3), V(4), V(5)]
    assert peel<V>(u => [V.nth((u.ord + 1) % 6)]) == []
fn fixed_cost_paths_match_dijkstra_on_the_same_moves():
    domain S = 0..20
    let moves = [1, 3, -2, 5]
    for _ in 0..30:
        let price = [rand(0, 6) for _ in 0..4]
        var es: List<(u: S, v: S, w: Int)> = []
        for s in S:
            for j in 0..4:
                let t = s.ord + moves[j]
                if 0 <= t < 20: es.push((u: s, v: S.nth(t), w: price[j]))
        domain E = 0..es.len
        let ea: [E](u: S, v: S, w: Int) = [E] e => es[e.ord]
        let src = S.nth(rand(0, 19))
        let want = dijkstra(Graph.directed(ea), from: [(src, 0)])
        let got = fixed_cost_paths(
            [src],
            [0, 1, 2, 3],
            cost: j => price[j],
            step: (
                j,
                s,
            ) => if s.ord + moves[j] >= 0 and s.ord + moves[j] < 20: Some(S.nth(s.ord + moves[j])) else: None,
        )
        for s in S: assert got[s] == want[s]
fn until_stops_at_the_target_with_its_exact_distance():
    domain V = 0..8
    domain E = 0..14
    fn closer_settled(full: [V]?Int, p: [V]?Int, t: V) -> Bool:
        if p[t] != full[t]: return false
        if let dt = full[t]:
            for v in V:
                if let dv = full[v]:
                    if dv < dt and p[v] != full[v]: return false
        true
    for _ in 0..40:
        let es = [E] _ => (u: V.nth(rand(0, 7)), v: V.nth(rand(0, 7)), w: rand(0, 1))
        let g = Graph.directed(es)
        let ws = [E] e => (u: es[e].u, v: es[e].v, w: es[e].w * rand(1, 5))
        let h = Graph.directed(ws)
        let plain: [V]List<V> = [V] u => [es[e].v for e in E if es[e].u == u]
        let s = V.nth(rand(0, 7))
        let (fb, f01, fd) = (
            bfs(plain, from: [s]),
            bfs01(g, from: [s]),
            dijkstra(h, from: [(s, 0)]),
        )
        for t in V:
            assert closer_settled([V] v => fb[v], [V] v => bfs(plain, from: [s], until: x => x == t)[v], t)
            assert closer_settled([V] v => f01[v], [V] v => bfs01(g, from: [s], until: x => x == t)[v], t)
            assert closer_settled(
                [V] v => fd[v],
                [V] v => dijkstra(h, from: [(s, 0)], until: x => x == t)[v],
                t,
            )
            let fc = fixed_cost_paths(
                [s],
                [e for e in E],
                cost: e => ws[e].w,
                step: (e, x) => if ws[e].u == x: Some(ws[e].v) else: None,
                until: x => x == t,
            )
            assert closer_settled([V] v => fd[v], [V] v => fc[v], t)
fn random_edges<V, E>() -> [E](u: V, v: V):
    [E] e => (u: V.nth(rand(0, V.count - 1)), v: V.nth(rand(0, V.count - 1)))
fn linked<V, E>(edges: [E](u: V, v: V), a: V, b: V, skip: ?E, gone: ?V) -> Bool:
    var d = Dsu<V>()
    for e in E:
        if e != skip and edges[e].u != gone and edges[e].v != gone: d.unite(edges[e].u, edges[e].v)
    d.same(a, b)
fn blocks_of_a_small_graph():
    domain V = 0..6
    domain E = 0..8
    for _ in 0..40:
        let edges = random_edges<V, E>()
        let (domain B, block) = blocks(Graph.undirected(edges))
        var same = Dsu<E>()
        for e in E:
            for f in E:
                let (eu, ev, fu, fv) = (edges[e].u, edges[e].v, edges[f].u, edges[f].v)
                if e == f or eu == ev or fu == fv: continue
                for x in [eu, ev]:
                    if x == fu or x == fv:
                        let a = if x == eu: ev else: eu
                        let b = if x == fu: fv else: fu
                        if linked(edges, a, b, None, Some(x)): same.unite(e, f)
        for e in E:
            for f in E: assert (block[e] == block[f]) == same.same(e, f)
        assert B.count == same.components
vertexless_closure_graphs_with_type_args()
blocks_of_a_small_graph()
fixed_cost_paths_match_dijkstra_on_the_same_moves()
until_stops_at_the_target_with_its_exact_distance()
print("YES")
