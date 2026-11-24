// Source: NOI 2018 Return (归程), https://dmoj.ca/problem/noi18p1
// Problem: a connected undirected graph, n <= 2e5, m <= 4e5, edges with length l <= 1e4 and
//   altitude a <= 1e9. Each of Q <= 4e5 days gives a start v and water line p (decoded with the
//   last answer when K = 1: forced online): drive free over edges with a > p, then walk to node
//   1; print the least walked length. T <= 3 tests.
// Solution: Dijkstra from node 1 gives dist. A Kruskal reconstruction tree merges components by
//   altitude, highest first; each new node keeps the edge's altitude and its subtree's minimum
//   dist. The car reaches exactly the leaves under v's highest ancestor with altitude > p, found
//   by binary lifting; print that node's minimum dist. O((n + m) log n + Q log n).
// original: corpus/submissions/6512702.cpp

let t_cases: Int = read()
for _ in 0..t_cases:
    let (n, m): (Int, Int) = read()
    domain V = 1..=n
    domain E = 0..m
    let edges: [E](u: V, v: V, len: Int, alt: Int) = read()
    let (q, k_flag, max_s): (Int, Int, Int) = read()
    domain Q = 0..q
    let raw_qs: [Q](v0: Int, p0: Int) = read()

    let g = Graph<V, E>.undirected([E] e => (u: edges[e].u, v: edges[e].v, w: edges[e].len))
    let dist = dijkstra(g, from: [(V.first!, 0)])

    let max_nodes = if 2 * n - 1 > 1: 2 * n - 1 else: 1
    domain KNode = 0..max_nodes
    let inf = 1000000000000000000
    var alt: [KNode]Int = [KNode] _ => inf
    var best_d: [KNode]Int = [KNode] _ => inf
    for v in V:
        best_d[KNode(v.ord)] = dist[v] ?? inf

    var dsu = Dsu<V>.new()
    var comp_root: [V]KNode = [V] v => KNode(v.ord)
    var parent: [KNode]?KNode = [KNode] _ => None
    var next_id = n

    let order = sorted([(-edges[e].alt, e.ord) for e in E])
    for pair in order:
        let e = E(pair.1)
        let ru = dsu.find(edges[e].u)
        let rv = dsu.find(edges[e].v)
        if ru == rv: continue
        let w = KNode(next_id)
        next_id += 1
        let cu = comp_root[ru]
        let cv = comp_root[rv]
        alt[w] = edges[e].alt
        best_d[w] = min(best_d[cu], best_d[cv])
        parent[cu] = w
        parent[cv] = w
        dsu.unite(ru, rv)
        comp_root[dsu.find(ru)] = w

    let log_n = 19
    var up: List<List<Int>> = []
    var row0: List<Int> = []
    for u in KNode:
        if let p = parent[u]: row0.push(p.ord)
        else: row0.push(-1)
    up.push(row0)
    for b in 1..log_n:
        var row: List<Int> = []
        for u in 0..max_nodes:
            let mid = up[b - 1][u]
            if mid == -1: row.push(-1)
            else: row.push(up[b - 1][mid])
        up.push(row)

    var last_ans = 0
    for qi in Q:
        let v_num = (raw_qs[qi].v0 + k_flag * last_ans - 1) % n + 1
        let p = (raw_qs[qi].p0 + k_flag * last_ans) % (max_s + 1)
        var cur = v_num - 1
        var b = log_n - 1
        while b >= 0:
            let anc = up[b][cur]
            if anc != -1 and alt[KNode(anc)] > p:
                cur = anc
            b -= 1
        last_ans = best_d[KNode(cur)]
        print(last_ans)
