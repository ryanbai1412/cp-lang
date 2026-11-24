// Source: SAC '22 Code Challenge 3 P4 (Finding Friends), https://dmoj.ca/problem/sac22cc3p4
// limits: 500 ms, 256 MB
// Problem: a tree on N <= 1e5 vertices and Q <= 2e5 queries: "1 u v" prints the distance from
//   u to v in edges, or -1 if they are no longer connected; "2 u v" deletes the edge u-v.
//   Every edge is deleted exactly once.
// Solution: answer the queries backwards, where deletions become unions in a Dsu that tells
//   whether u and v are still connected. If they are, their path is the original tree path,
//   so the answer is its length in the tree rooted at 1. O((N + Q) log N).
// original: corpus/submissions/4331646.cpp
let (n, q) = read<(2..=1e5, 1..=2e5)>()
domain V = 1..=n
let tree = Rooted(Graph.undirected(read<Tree<V>>()), root: V.first!)
let queries = read<[q](kind: 1..=2, u: V, v: V)>()

// Backwards in time a cut is a join, so one Dsu answers "still connected?".
var joined = Dsu<V>()
var answers: List<?Int> = []
for (kind, u, v) in queries.rev:
    if kind == 2: joined.unite(u, v)
    else: answers.push(if joined.same(u, v): tree.dist(u, v) else: None)
for a in answers.rev: print(a ?? -1)
