// APIO 2023 "Cyberland": reach H from 0; some cities reset the elapsed time to
// 0, some halve it (at most k uses, and more than ~70 never helps). Arriving at
// H ends the trip. Layered Dijkstra: each layer is one more halving.
//
// The task is a grader function; this reads the sample grader's input (T, then
// per case `N M K`, `H`, the N abilities, M roads `x y c`) and prints each
// answer, -1 when H is unreachable.
@!multitest
let (n, m, k) = read<(2..=1e5, 0..=1e5, 1..=1e6)>()
domain V = [n]
let goal = read<V>()
let power = read<[V]0..=2>()
let roads = read<[m](u: V, v: V, w: 1..=1e9)>()
var adj: [V]List<(to: V, w: F64)> = []
for r in roads:
    adj[r.u].push((to: r.v, w: F64(r.w)))
    adj[r.v].push((to: r.u, w: F64(r.w)))
let trip = adj.only(from: v => v != goal)
let first = dijkstra(trip, from: [(V(0), 0.0)])
if first[goal] == None:
    print(-1)
    return
var dist = dijkstra(
    trip,
    from: chain([(V(0), 0.0)], V.filter(v => first[v] != None and power[v] == 0).map(v => (v, 0.0))),
).dist
for _ in 0..min(k, 70):
    var starts: List<(V, F64)> = []
    for v in V:
        let d = dist[v] else: continue
        starts.push((v, d))
        if power[v] == 2:
            for a in trip.out(v): starts.push((a.to, d / 2 + a.w))
    dist = dijkstra(trip, from: starts).dist
print(dist[goal]!, precision: 6)
