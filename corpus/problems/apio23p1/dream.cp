// apio23p1 Cyberland: the least time from vertex 0 to `h`. Ability 0 resets the time to 0;
// ability 2 halves it, at most k times, and past about 70 halvings nothing changes in F64.
// Layered Dijkstra: each round starts from every vertex's best time so far, plus every halving.
@grader
fn solve(n: Int, m: Int, k: Int, h: V, x: [E]V, y: [E]V, c: [E]1..=1_000_000_000, arr: [V]0..=2) -> F64:
    let roads = Graph.undirected(E.map(e => (u: x[e], v: y[e], w: F64(c[e]))).collect())
    let start = V.first!
    let g = (v: V) => roads[v].filter(_ => v != h)
    let reach = dijkstra(g, from: [(start, 0.0)]).dist
    if reach[h] == none:
        return -1.0
    var dist = dijkstra(g, from: [(v, 0.0) for v in V if v == start or (arr[v] == 0 and reach[v] != none)]).dist
    for _ in 0..min(k, 70):
        let keep = V.filter_map(v => dist[v].map(d => (v, d)))
        let halve = V.filter(v => arr[v] == 2).flat_map(v => g(v).filter_map(a => dist[v].map(d => (a.to, d / 2 + a.w))))
        dist = dijkstra(g, from: keep.chain(halve)).dist
    dist[h]!
