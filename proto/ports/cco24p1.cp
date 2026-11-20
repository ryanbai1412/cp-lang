// CCO 2024 Treasure Hunt: starting at v, walk to any u and collect a[u] minus
// the road tolls paid. Best value at v = max over u of a[u] - dist(u, v):
// one Dijkstra from every vertex at once, u starting at "distance" -a[u].
let n: 1..=1e6 = read()
let m: 0..=1e6 = read()
domain V = 1..=n
let a: [V]1..=1e9 = read()
let roads: [m](u: V, v: V, toll: 1..=1e9) = read()

var adj: [V]List<(V, Int)> = []
for (u, v, toll) in roads:
    adj[u].push((v, toll))
    adj[v].push((u, toll))

var cost: [V]Int = [V] v => -a[v]
var heap = MinHeap<(Int, V)>()
for v in V: heap.push((cost[v], v))
while let (c, u) = heap.pop():
    if c > cost[u]: continue
    for (v, toll) in adj[u]:
        if c + toll < cost[v]:
            cost[v] = c + toll
            heap.push((cost[v], v))
for v in V: print(-cost[v])
