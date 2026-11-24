// Source: COCI '21 Contest 3 #2, Cijanobakterije, https://dmoj.ca/problem/coci21c3p2
// limits: 1000 ms, 512 MB
// Problem: a forest on n <= 1e5 vertices with m < n edges. Joining trees by new edges, print
//   the most vertices a single path (chain) can have.
// Solution: link the trees' diameters end to end, so the answer is the sum of every tree's
//   diameter counted in vertices. Root the forest; children before parents, keep the deepest
//   branch below each vertex and the longest path topped there, then take each tree's best. O(n).
// original: corpus/submissions/4196157.cpp
// Bounds: n: 1..=1e5; m: 0..n; edges.u, edges.v: 1..=n

let n: 1..=1e5 = read()
let m: 0..n = read()
domain V = 1..=n
domain E = 0..m
let edges: [E](u: V, v: V) = read()

let forest = Rooted.forest(Graph.undirected(edges))
var deepest: [V]Int = 1  // vertices on the longest branch down from v
var topped: [V]Int = 1  // vertices on the longest path whose highest vertex is v
for v in forest.order.rev:
    if let p = forest.parent(v):
        topped[p] max= deepest[p] + deepest[v]
        deepest[p] max= deepest[v] + 1

var diameter: [V]Int = 0
for v in V: diameter[forest.root_of(v)] max= topped[v]
print(diameter.sum())
