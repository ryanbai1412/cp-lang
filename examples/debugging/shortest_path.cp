// The shortest distance from vertex 1 to vertex n over undirected weighted
// roads, or -1 when n can't be reached. A bug is planted in the relaxation.
use std.heap.{MinHeap}

let (n, m): (2..=6, 1..=8) = read()
domain V = 1..=n
domain E = 0..m
let roads: [E](u: V, v: V, w: 1..=9) = read()

var adj: [V]List<(to: V, w: Int)> = []
for e in E:
    adj[roads[e].u].push((to: roads[e].v, w: roads[e].w))
    adj[roads[e].v].push((to: roads[e].u, w: roads[e].w))

var dist: [V]?Int = none
var heap = MinHeap<(Int, V)>()
dist[V.first] = 0
heap.push((0, V.first))
while let (d, u) = heap.pop():
    for road in adj[u]:
        if dist[road.to] == none:
            dist[road.to] = d + road.w
            heap.push((d + road.w, road.to))
print(dist[V.last] ?? -1)

@alt(program)
fn bellman_ford():
    var far: [V]?Int = none
    far[V.first] = 0
    for _ in V:
        for e in E:
            let r = roads[e]
            if let du = far[r.u]: far[r.v] min= du + r.w
            if let dv = far[r.v]: far[r.u] min= dv + r.w
    print(far[V.last] ?? -1)
