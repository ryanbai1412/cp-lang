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

@!multitest
let n: 3..=500_000 = read()
domain V = 1..=n
let (k, start): (1..=n, V) = read()
let edges: [0..n - 1](u: V, v: V) = read()
let g = Graph.undirected(edges)
let t = Rooted(g, root: start)

var dist: [V]Int = k + 2
for u in t.at.rev:
    if g[u].len == 1:
        dist[u] = 0
        continue
    var d = k + 2
    for c in t.children(u):
        let nd = dist[c] + 1
        if d + nd <= k + 1:
            d = 0
            break
        d min= nd
    dist[u] = d

print(yes(dist[start] == 0))
