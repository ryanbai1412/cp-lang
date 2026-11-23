// Source: Codeforces 2245E (Tom and Jerry), https://codeforces.com/contest/2245/problem/E
// Problem: a tree on n <= 2e5 vertices. Players alternately pick a simple path that shares no
//   edge with earlier paths and has an end on the previous path; who cannot move loses. Count
//   the first paths with which the first player wins. Multitest, sum of n <= 2e5.
// Solution: the winning first paths are those whose two ends have odd degree and whose inner
//   vertices all have even degree. Glue each edge to its even-degree ends in a DsuOf over
//   vertices and edges, weighting edges +1 and vertices -1: a piece is a subtree with even
//   inner vertices and e - v + 1 = diff + 1 odd ends, so it adds choose2(diff + 1). O(n a(n)).

@!multitest
let n: 2..=200_000 = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()

var odd: [V]Bool = false
for (u, v) in edges:
    odd[u] = not odd[u]
    odd[v] = not odd[v]

// Packs `V` (0..n) and `E` (n..2*n-1) until sum domains `V | E` exist (dream:23).
domain X = 0..2 * n - 1
var pieces = DsuOf<X, Int>(x => if x.ord >= n: 1 else: -1, +)
for (e, (u, v)) in edges.items:
    if not odd[u]: pieces.unite(X(n + e), X(u.ord))
    if not odd[v]: pieces.unite(X(n + e), X(v.ord))

print(sum(choose2(diff + 1) for diff in pieces.values()))
