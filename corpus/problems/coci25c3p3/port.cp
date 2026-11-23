// Source: COCI 2025/2026 Contest 3 P3 (Domjenak), https://dmoj.ca/problem/coci25c3p3
// Problem: 2N people (N <= 5e5), M <= 1e6 friendships, bipartite, with a unique perfect
//   matching (the pairs who came together). Told a story, a person passes it to their pair if
//   the pair hasn't heard it, else to one friend who hasn't. Print the longest chain of
//   tellers and one such sequence: an alternating path that starts with a pair edge.
// Solution: with a unique perfect matching some vertex has degree 1; two-colour the graph,
//   then peel degree-1 vertices on side 0: the lone unmatched neighbour is its pair, and
//   removing the pair's other edges creates new degree-1 vertices. The peel order is a DAG
//   of non-pair edges, so dp[v] (the longest chain of pairs ending at v) relaxes along it,
//   with a parent pointer to rebuild the path. O(N + M).
// original: corpus/submissions/7730378.cpp
use scout.connectivity.{two_color}

let (n, m): (Int, Int) = read()
domain V = 1..=2 * n
domain E = 0..m

let edges: [E](u: V, v: V) = read()
let g = Graph<V, E>.undirected(edges)
let side = two_color::<V>(u => g.out(u))!

var deg: [V]Int = [V] u => g.deg(u)
var mate: [V]?V = None
var dp: [V]Int = 0
var nxt: [V]?V = None

var q = [u for u in V if not side[u] and deg[u] == 1]
var head = 0
var best_len = -1
var start: ?V = None

while head < q.len:
    let u = q[head]
    head += 1
    for a in g.out(u):
        if mate[a.to] == None:
            mate[a.to] = u
            mate[u] = a.to
    let p = mate[u]!
    for a in g.out(p):
        let v = a.to
        if mate[v] == None:
            if dp[v] < dp[u] + 1:
                dp[v] = dp[u] + 1
                nxt[v] = u
            deg[v] -= 1
            if deg[v] == 1: q.push(v)
    if dp[u] > best_len:
        best_len = dp[u]
        start = u

print(2 * (best_len + 1))
var line = ""
var cur = start
while let u = cur:
    if line.len > 0: line += " "
    line += "{mate[u]!.label} {u.label}"
    cur = nxt[u]
print(line)
