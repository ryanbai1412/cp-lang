// expect: ok, output "YES"
// The graph.cp @test bodies run as a program: `cpl test` on std files does not yet see built-in `Next` impls.
use std.graph.{cut_vertices}

fn random_edges<V, E>() -> [E](u: V, v: V):
    [E] e => (u: V.nth(rand(0, V.count - 1)), v: V.nth(rand(0, V.count - 1)))

fn linked<V, E>(edges: [E](u: V, v: V), a: V, b: V, skip: ?E, gone: ?V) -> Bool:
    var d = Dsu<V>()
    for e in E:
        if e != skip and edges[e].u != gone and edges[e].v != gone: d.unite(edges[e].u, edges[e].v)
    d.same(a, b)

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

fn has_walk<V>(edges: List<(V, V)>, directed: Bool) -> Bool:
    if edges.is_empty: return true
    for (u, v) in edges:
        for s in (if directed: [u] else: [u, v]):
            var used = List.filled(edges.len, false)
            if extend(edges, directed, mut used, s, 0): return true
    false

fn follows<V>(edges: List<(V, V)>, walk: List<Int>, directed: Bool, start: V) -> Bool:
    var at = start
    for i in walk:
        let (u, v) = edges[i]
        if u == at: at = v
        elif not directed and v == at: at = u
        else: return false
    true

fn components_of_an_undirected_graph():
    domain V = 0..8
    for _ in 0..30:
        var adj: [V]List<V> = [V] _ => []
        var d = Dsu<V>()
        for _ in 0..rand(0, 8):
            let (u, v) = (V.nth(rand(0, 7)), V.nth(rand(0, 7)))
            adj[u].push(v)
            adj[v].push(u)
            d.unite(u, v)
        let (domain C, comp) = components(adj)
        assert C.count == d.components
        for a in V:
            for b in V: assert (comp[a] == comp[b]) == d.same(a, b)

fn dijkstra_matches_brute_relaxation():
    domain V = 0..6
    domain E = 0..10
    for _ in 0..30:
        let es = [E] _ => (u: V.nth(rand(0, 5)), v: V.nth(rand(0, 5)), w: rand(0, 9))
        let g = Graph.directed(es)
        for s in V:
            var d: [V]?Int = None
            d[s] = Some(0)
            for _ in V:
                for e in E:
                    let (u, v, w) = (es[e].u, es[e].v, es[e].w)
                    if let du = d[u]:
                        if d[v] == None or du + w < d[v]!: d[v] = Some(du + w)
            let p = dijkstra(g, from: [(s, 0)])
            for t in V: assert p[t] == d[t]

fn paths_witness_their_distances():
    domain V = 0..7
    domain E = 0..12
    for _ in 0..30:
        let es = [E] _ => (u: V.nth(rand(0, 6)), v: V.nth(rand(0, 6)), w: rand(0, 9))
        var light: [V, V]?Int = None
        for e in E:
            let (u, v, w) = (es[e].u, es[e].v, es[e].w)
            if light[u, v] == None or w < light[u, v]!: light[u, v] = Some(w)
        let g = Graph.directed(es)
        let near = dijkstra(g, from: [(V(0), 0)])
        let hops = bfs(g, from: [V(0)])
        for t in V:
            assert (near.path(t) == None) == (near[t] == None)
            assert (hops.path(t) == None) == (hops[t] == None)
            if let path = near.path(t):
                assert path.first! == V(0) and path.last! == t
                assert sum(light[path[i], path[i + 1]]! for i in 0..path.len - 1) == near[t]!
            if let path = hops.path(t):
                assert path.first! == V(0) and path.last! == t and path.len - 1 == hops[t]!
                for i in 0..path.len - 1: assert light[path[i], path[i + 1]] != None

fn bellman_ford_matches_relaxation_and_finds_negative_cycles():
    domain V = 0..5
    domain E = 0..8
    var (negative, fine) = (0, 0)
    for _ in 0..60:
        let es = [E] _ => (u: V.nth(rand(0, 4)), v: V.nth(rand(0, 4)), w: rand(-3, 9))
        let g = Graph.directed(es)
        for s in V:
            var d: [V]?Int = None
            d[s] = Some(0)
            var moved = false
            for _ in 0..V.count + 1:
                moved = false
                for e in E:
                    let (u, v, w) = (es[e].u, es[e].v, es[e].w)
                    if let du = d[u]:
                        if d[v] == None or du + w < d[v]!:
                            d[v] = Some(du + w)
                            moved = true
            let got = bellman_ford(g, from: [(s, 0)])
            assert (got == None) == moved
            if let p = got:
                fine += 1
                for t in V: assert p[t] == d[t]
            else: negative += 1
    assert negative > 0 and fine > 0

fn two_color_finds_a_colouring_or_an_odd_cycle():
    domain V = 0..7
    for _ in 0..40:
        var adj: [V]List<V> = [V] _ => []
        var es: List<(V, V)> = []
        for _ in 0..rand(0, 8):
            let (u, v) = (V.nth(rand(0, 6)), V.nth(rand(0, 6)))
            adj[u].push(v)
            adj[v].push(u)
            es.push((u, v))
        var any = false
        for m in 0..128:
            if es.all(e => ((m >> e.0.ord) & 1) != ((m >> e.1.ord) & 1)): any = true
        let got = two_color(adj)
        assert (got != None) == any
        if let side = got:
            for (u, v) in es: assert side[u] != side[v]

fn bridges_and_cut_vertices_match_removing_them():
    domain V = 0..6
    domain E = 0..7
    for _ in 0..40:
        let edges = random_edges<V, E>()
        let g = Graph.undirected(edges)
        let bridge = bridges(g)
        let cut = cut_vertices(g)
        for e in E:
            assert bridge[e] == (not linked(edges, edges[e].u, edges[e].v, Some(e), None))
        for v in V:
            var splits = false
            for a in V:
                for b in V:
                    if a != v and b != v and linked(edges, a, b, None, None) and not linked(edges, a, b, None, Some(v)):
                        splits = true
            assert cut[v] == splits

fn at_most_one_forbids_every_pair_and_allows_each_single():
    domain X = 0..5
    let lits = [Lit(X(i), i % 2 == 0) for i in 0..5]
    for i in 0..5:
        for j in 0..5:
            var s = TwoSat<X>()
            s.at_most_one(lits)
            s.must(lits[i])
            s.must(lits[j])
            let got = s.solve()
            assert (got == None) == (i != j)
            if let a = got:
                for k in 0..5: assert (a[X(k)] == lits[k].value) == (k == i)


fn bfs_over_a_stored_graph_and_over_a_lambda():
    domain V = 0..10
    let next = [V] v => [V.nth(x) for x in [v.ord + 1, v.ord * 2, v.ord - 3] if 0 <= x < 10]
    let es = [(v, w) for v in V for w in next[v]]
    let stored = bfs(Graph.directed(es), from: [V(1)])
    let lambda = bfs<V>(v => next[v], from: [V(1)])
    var dist: [V]?Int = None
    dist[V(1)] = Some(0)
    for _ in V:
        for v in V:
            if let d = dist[v]:
                for w in next[v]:
                    if dist[w] == None or d + 1 < dist[w]!: dist[w] = Some(d + 1)
    for v in V: assert stored[v] == dist[v] and lambda[v] == dist[v]

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
            assert w.vertices.collect().len == w.arcs.len + 1

components_of_an_undirected_graph()
dijkstra_matches_brute_relaxation()
paths_witness_their_distances()
bellman_ford_matches_relaxation_and_finds_negative_cycles()
two_color_finds_a_colouring_or_an_odd_cycle()
bridges_and_cut_vertices_match_removing_them()
at_most_one_forbids_every_pair_and_allows_each_single()
bfs_over_a_stored_graph_and_over_a_lambda()
euler_walks_use_every_edge_once_and_exist_when_brute_force_finds_one()
print("YES")
