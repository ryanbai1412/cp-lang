// Tuned: port.cp walking the tree in bfs order with parents from bfs's back links, as orig.cpp
// walks adj directly, instead of a Rooted tree, which also builds tours and depths never read.
// Source: Codeforces 2207D (Boxed Like a Fish), https://codeforces.com/contest/2207/problem/D
// limits: 2000 ms, 256 MB
// Problem: a tree of n <= 5e5 vertices; Cyndaquil starts at non-leaf v and wants a leaf. Snorlax
//   moves first and may block one edge at a time, choosing a new one only k turns after the
//   last; Cyndaquil moves one edge or stays per turn. Print YES iff Cyndaquil can surely reach a
//   leaf. Multitest, sum of n <= 5e5.
// Solution: root at v. dist[u] is the shortest escape distance below u (leaves 0, capped at
//   k + 2). If two children of u give escapes at total distance <= k + 1, Snorlax cannot block
//   both in time, so u counts as an escape itself (dist 0); else dist[u] is the closest child's
//   + 1. Bottom-up, O(n); YES iff dist[v] == 0.
// Bounds: n: 3..=500_000; k, start, edges.u, edges.v: 1..=n

@!multitest
let n: 3..=500_000 = read()
domain V = 1..=n
let (k, start): (1..=n, V) = read()
let edges: [0..n - 1](u: V, v: V) = read()
let g = Graph.undirected(edges)
let walk = bfs(g, from: [start])
let order = [s.v for s in walk.iter()]
let par: [V]V = [V] v => walk.back[v].map(b => b.from) ?? start

var dist: [V]Int = k + 2
for u in order.rev:
    if g[u].len == 1:
        dist[u] = 0
        continue
    var d = k + 2
    for c in g[u] if c != par[u]:
        let nd = dist[c] + 1
        if d + nd <= k + 1:
            d = 0
            break
        d min= nd
    dist[u] = d

print(yes(dist[start] == 0))
