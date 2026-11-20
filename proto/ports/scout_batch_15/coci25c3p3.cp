// COCI '25 C3 P3: a bipartite graph on 2n vertices with a unique perfect
// matching (peelable from degree-1 vertices on side 0). Find the longest
// alternating path and print its length and vertices (1-based).
//
// Two-color the graph with `scout.connectivity.two_color`, then peel degree-1
// vertices on side 0 to reveal each forced matching edge `(u, partner)` while
// computing longest-path DP over the resulting DAG of unmatched edges.
// original: corpus/submissions/7730378.cpp
use std.graph.{Graph}
use scout.connectivity.{two_color}

let (n, m): (Int, Int) = read()
domain V = 1..=2 * n
domain E = 0..m

let edges: [E](u: V, v: V) = read()
let g = Graph<V, E>.undirected(edges)
let side = two_color::<V>(u => g.adj(u))!

var deg: [V]Int = [V] u => g.deg(u)
var mate: [V]?V = none
var dp: [V]Int = 0
var nxt: [V]?V = none

var q = [u for u in V if not side[u] and deg[u] == 1]
var head = 0
var best_len = -1
var start: ?V = none

while head < q.len:
    let u = q[head]
    head += 1
    for a in g.adj(u):
        if mate[a.to] == none:
            mate[a.to] = u
            mate[u] = a.to
    let p = mate[u]!
    for a in g.adj(p):
        let v = a.to
        if mate[v] == none:
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
    if line.len > 0: line = line + " "
    line = line + "{mate[u]!.label} {u.label}"
    cur = nxt[u]
print(line)
