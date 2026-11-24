// Source: CCC 2023 S4 (Minimum Cost Roads), https://dmoj.ca/problem/ccc23s4
// Problem: N, M <= 2000; road i joins u_i, v_i with length 0 <= l_i <= 1e9 and cost c_i <= 1e9.
//   Keep a subset of roads of least total cost such that every pair of intersections keeps its
//   shortest distance. Print that cost.
// Solution: take roads by (length, cost); keep a road iff the kept roads do not already join its
//   ends within its length (a Dijkstra from one end over the kept graph, stopped once the other end settles). Shorter roads never
//   need longer ones, so this is exact. O(M (N + M) log N).
// original: corpus/submissions/5350808.cpp
let n: 1..=2000 = read()
let m: 1..=2000 = read()
domain V = 1..=n
domain E = 0..m
let roads: [E](u: V, v: V, len: 0..=1e9, cost: 1..=1e9) = read()

var kept: [V]List<(to: V, w: Int)> = []
var total = 0
for e in sorted(E, by: e => (roads[e].len, roads[e].cost)):
    let (u, v, len, cost) = roads[e]
    let joined = if let d = dijkstra(kept, from: [(u, 0)], until: v)[v]: d <= len else: false
    if not joined:
        total += cost
        kept[u].push((to: v, w: len))
        kept[v].push((to: u, w: len))
print(total)
