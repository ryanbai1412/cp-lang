// APIO 2023 "Cyberland": reach H from 0; some cities reset the elapsed time to
// 0, some halve it (at most k uses, and more than ~70 never helps). Arriving at
// H ends the trip. Layered Dijkstra: each layer is one more halving.

@grader
fn solve(n: Int, m: Int, k: Int, h: Int, x: List<Int>, y: List<Int>, c: List<Int>, arr: List<Int>) -> F64:
    domain V = [n]
    let goal = V(h)
    let roads = Graph<V>.undirected([x.keys] e => (u: V(x[e]), v: V(y[e]), w: F64(c[e])))
    let trip = roads.only(from: u => u != goal)
    let seen = reachable(trip, from: [V(0)])
    if not seen[goal]:
        return -1.0
    let starts = chain([V(0)], V.filter(v => seen[v] and arr[Int(v)] == 0))
    var dist = dijkstra(trip, from: starts.map(v => (v, 0.0))).dist
    for _ in 0..min(k, 70):
        let halved = V.filter(v => arr[Int(v)] == 2).flat_map(v =>
            dist[v].map(d => trip[v].map(a => (a.to, d / 2 + a.w))) ?? [])
        dist = dijkstra(trip, from: chain(dist.items.filter_map((v, d) => d.map(d => (v, d))), halved)).dist
    dist[goal]!
