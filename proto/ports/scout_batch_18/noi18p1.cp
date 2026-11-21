// NOI '18 P1 - Return (DMOJ: noi18p1, 17pt, TL 4.0s, ML 511MB)
// Statement-first CP Lang solution (written before reading corpus C++).
//
// Multitest (T <= 3). Undirected connected graph (n <= 2e5, m <= 4e5) where
// each edge has length l and altitude a. Q <= 4e5 online queries (v0, p0)
// decoded with lastans: starting at v, drive free on edges with altitude > p,
// then walk to vertex 1 paying shortest edge-length distance. Minimize walking cost.
//
// 1) Dijkstra from vertex 1 on edge lengths -> dist[v].
// 2) Kruskal reconstruction tree (edges sorted by descending altitude):
//    nodes 1..=n are original vertices, nodes n+1..=2n-1 are internal edge nodes
//    with alt[w] = edge.a and min_dist[w] = min(min_dist[left], min_dist[right]).
// 3) Binary lifting `up[k][w]` over the Kruskal tree answers each online query
//    in O(log n) by climbing while alt[ancestor] > p.
use std.graph.{Graph, dijkstra}
use std.dsu.{Dsu}

@multitest
let (n, m): (1..=2e5, 0..=4e5) = read()
domain V = 1..=n
domain E = 0..m
// WISH W21: `Graph(connected: true)` typed input constraint + edge record fields
let edges: [E](u: V, v: V, len: 1..=1e4, alt: 1..=1e9) = read()
let (q, k_flag, max_s): (1..=4e5, 0..=1, 0..=1e9) = read()
domain Q = 0..q
// WISH W22: online encrypted queries depend on `lastans`, so `cpl stress` cannot
// validate decoded `(v, p)` constraints purely from the raw `(v0, p0)` read types.
let raw_qs: [Q](v0: 1..=n, p0: 0..=max_s) = read()

fn solve() -> [Q]Int:
    let g = Graph<V, E>.undirected([E] e => (u: edges[e].u, v: edges[e].v))
    let dist = dijkstra::<V>(u => g.out(u), [(V.first, 0)], a => edges[a.e].len)

    // WISH W23: Multi-feature collision — Kruskal reconstruction tree extends domain V
    // with up to n - 1 dynamically created virtual internal nodes. There is no
    // `ExtendedDomain` or `pool` + binary-lifting support, so we drop down to
    // raw `0..(2 * n - 1)` indices and lose V-vs-KNode type safety.
    let max_nodes = max(1, 2 * n - 1)
    domain KNode = 0..max_nodes
    var alt: [KNode]Int = Int.MAX
    var best_d: [KNode]Int = Int.MAX
    for v in V:
        best_d[KNode(v.to_idx)] = dist[v] ?? Int.MAX

    // Bare-core DSU + tree root mapping (stdlib Dsu has no payload per component,
    // so we maintain `comp_root: [V]KNode` beside `Dsu<V>`).
    var dsu = Dsu<V>()
    var comp_root = [V] v => KNode(v.to_idx)
    var parent: [KNode]?KNode = none
    var next_id = n

    for e in sorted(E, by: e => -edges[e].alt):
        let (ru, rv) = (dsu.find(edges[e].u), dsu.find(edges[e].v))
        if ru == rv: continue
        let w = KNode(next_id)
        next_id += 1
        let (cu, cv) = (comp_root[ru], comp_root[rv])
        alt[w] = edges[e].alt
        best_d[w] = min(best_d[cu], best_d[cv])
        parent[cu] = w
        parent[cv] = w
        dsu.unite(ru, rv)
        comp_root[dsu.find(ru)] = w

    // Binary lifting table: up[b][node] = 2^b-th ancestor
    // WISH W24: `stdlib/tree.cp` (`Rooted`) does not expose a generic binary-lifting
    // / jump-while-predicate helper (`lift_while(v, anc => alt[anc] > p)`), and
    // Kruskal trees may even be forests if input isn't connected.
    let log_n = 19
    var up = [0..log_n, KNode] (_, _) => (none: ?KNode)
    for u in KNode:
        up[0, u] = parent[u]
    for b in 1..log_n:
        for u in KNode:
            if let mid = up[b - 1, u]:
                up[b, u] = up[b - 1, mid]

    var last_ans = 0
    var out: [Q]Int = 0
    for qi in Q:
        let v_num = (raw_qs[qi].v0 + k_flag * last_ans - 1) % n + 1
        let p = (raw_qs[qi].p0 + k_flag * last_ans) % (max_s + 1)
        var cur = KNode(v_num - 1)
        for b in (0..log_n).rev:
            if let anc = up[b, cur]:
                if alt[anc] > p:
                    cur = anc
        last_ans = best_d[cur]
        out[qi] = last_ans
    out

@brute(solve)
fn solve_brute() -> [Q]Int
    where n <= 80 and q <= 80:
    let g = Graph<V, E>.undirected([E] e => (u: edges[e].u, v: edges[e].v))
    let dist = dijkstra::<V>(u => g.out(u), [(V.first, 0)], a => edges[a.e].len)
    var last_ans = 0
    var out: [Q]Int = 0
    for qi in Q:
        let v = V((raw_qs[qi].v0 + k_flag * last_ans - 1) % n + 1)
        let p = (raw_qs[qi].p0 + k_flag * last_ans) % (max_s + 1)
        var seen: [V]Bool = false
        seen[v] = true
        var queue = [v]
        var head = 0
        var ans = dist[v] ?? Int.MAX
        while head < queue.len:
            let u = queue[head]
            head += 1
            ans min= (dist[u] ?? Int.MAX)
            for a in g.out(u):
                if edges[a.e].alt > p and not seen[a.to]:
                    seen[a.to] = true
                    queue.push(a.to)
        last_ans = ans
        out[qi] = ans
    out

for ans in solve(): print(ans)
