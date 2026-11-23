// Source: SAC '22 Code Challenge 3 P4 (Finding Friends), https://dmoj.ca/problem/sac22cc3p4
// Problem: a tree on N <= 1e5 vertices and Q <= 2e5 queries: "1 u v" prints the distance from
//   u to v in edges, or -1 if they are no longer connected; "2 u v" deletes the edge u-v.
//   Every edge is deleted exactly once.
// Solution: answer the queries backwards, where deletions become unions in a DSU that tells
//   whether u and v are still connected. If they are, their path is the original tree path:
//   depth(u) + depth(v) - 2 depth(lca(u, v)) on the tree rooted at 1. O((N + Q) log N).
let n: 2..=2e5 = read()
let q: 1..=2e5 = read()
domain V = 1..=n
domain E = 1..n

fn random_tree() -> [E](u: V, v: V):
    [E] e => (V(rand(1, e.label)), V(e.label + 1))

let edges: [E](u: V, v: V) = read(gen: random_tree)
let queries: [q](kind: 1..=2, u: V, v: V) = read()
let tree = Rooted(Graph.undirected(edges), V.first!)

// Backwards in time a cut is a join, so one Dsu answers "still connected?".
var joined = Dsu<V>()
var answers: List<Int> = []
for (kind, u, v) in queries.rev:
    if kind == 2: joined.unite(u, v)
    elif joined.same(u, v): answers.push(
        tree.depth(u) + tree.depth(v) - 2 * tree.depth(tree.lca(u, v))
    )
    else: answers.push(-1)
for a in answers.rev: print(a)
