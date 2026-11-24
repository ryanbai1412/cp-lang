// Source: CCO 2024 P1 (Treasure Hunt), https://dmoj.ca/problem/cco24p1
// Problem: an undirected graph of N <= 1e6 islands, island i holding v_i coins, and M <= 1e6
//   routes of cost c_i (0 <= v_i, c_i <= 1e9). For every start x print the best net profit,
//   the max over y of v_y - dist(x, y) (y = x allowed).
// Solution: the answer at x is -min over y of (-v_y + dist(y, x)), a shortest path from a
//   virtual source joined to each y by weight -v_y. So one Dijkstra seeds every vertex at
//   -v_y and relaxes as usual (route costs are nonnegative). O((N + M) log N).
let n: 1..=1e6 = read()
let m: 0..=1e6 = read()
domain V = 1..=n
let a: [V]0..=1e9 = read()
let roads: [m](u: V, v: V, toll: 0..=1e9) = read()

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
