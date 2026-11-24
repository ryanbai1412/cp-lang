// Source: Codeforces 2135 C (By the Assignment), https://codeforces.com/contest/2135/problem/C
// Problem: a connected graph, n <= 2e5 vertices, m <= 4e5 edges; some vertices have weights,
//   the rest (-1) get one in 0..V-1 (V <= 1e9). Count the assignments where every two vertices
//   have the same XOR of weights along all simple paths between them, modulo 998244353.
//   Multitest, sums of n and m bounded.
// Solution: two paths differ by a cycle, so every 2-edge-connected component needs one common
//   value, and 0 if it has an odd cycle (an inner edge joins equal BFS depth parities). Fixed
//   weights force their component's value; a conflict gives 0, and each unforced component
//   multiplies by V. Bridge decomposition and BFS, O(n + m).

use std.graph.{two_edge_components}

type Z = Mod<998_244_353>

@!multitest
let (n, m, lim): (2..=200_000, 1..=400_000, 1..=10 ** 9) = read()
domain V = 1..=n
domain E = 0..m
let a: [V]?Int = read(none: -1)
let edges: [E](u: V, v: V) = read()
let g = Graph.undirected(edges)

let (domain C, comp) = two_edge_components(g)
let depth = bfs(g, [V(1)])

var want: [C]?Int = None
var ans: Z = 1
fn force(c: C, x: Int):
    if (want[c] ?? x) != x: ans = 0
    want[c] = x
for (u, v) in edges:
    if comp[u] == comp[v] and depth[u]! % 2 == depth[v]! % 2: force(comp[u], 0)
for u in V:
    if let x = a[u]: force(comp[u], x)
for c in C:
    if want[c] == None: ans *= Z(lim)
print(ans)
