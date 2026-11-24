// Tuned: port.cp walking the tree breadth-first by hand instead of a Rooted tree or bfs's
// Paths with their dist and back links, as orig.cpp walks adj directly.
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
// The order is a flat list and par an array; the edges go straight into the graph, and dist
// is narrow, so no edge list, bfs dist or back links stay alive.

@!multitest
let n: 3..=500_000 = read()
domain V = 1..=n
let (k, start): (1..=n, V) = read()
let g = Graph.undirected(read<[0..n - 1](u: V, v: V)>())
var par: [V]V = start
var order: [0..n]V = start  // breadth-first: each vertex after its parent
var seen = 1
for h in 0..n:
    let u = order[h]
    for c in g[u] if c != par[u]:
        par[c] = u
        order[seen] = c
        seen += 1

var dist: [V]0..=500_002 = k + 2
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
