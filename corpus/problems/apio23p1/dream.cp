// APIO 2023 "Cyberland": reach H from 0; some cities reset the elapsed time to
// 0, some halve it (at most k uses, and more than ~70 never helps). Arriving at
// H ends the trip. Layered Dijkstra: each layer is one more halving.
enum Power: Reset | Plain | Halve

@!multitest
let (n, m, k): (2..=1e5, 0..=1e5, 1..=1e6) = read()
domain V = 0..n
let h: 1..=n - 1 = read()
let goal = V(h)
let kind: [V]0..=2 = read()
let power = [V] v => Power.at(kind[v])
let roads: [m](u: V, v: V, w: 1..=1e9) = read()
let both = [a for r in roads for a in [(u: r.u, v: r.v, w: F64(r.w)), (u: r.v, v: r.u, w: F64(r.w))]]
let trip = Graph.of([(from: a.u, arc: (to: a.v, w: a.w)) for a in both if a.u != goal])
let reach = dijkstra(trip, from: [(V(0), 0.0)])
if reach.get(goal) == None:
    print(-1)
else:
    let starts = [(v, 0.0) for v in V if reach.get(v) != None and (v == V(0) or power[v] == Reset)]
    var dist = dijkstra(trip, from: starts)
    for _ in 0..min(k, 80):
        let halved = [(a.to, d / 2 + a.w) for (v, d) in dist.iter() if power[v] == Halve for a in trip.out(v)]
        dist = dijkstra(trip, from: chain(dist.iter().map(p => (p.v, p.d)), halved))
    print(dist.get(goal)!, precision: 6)
