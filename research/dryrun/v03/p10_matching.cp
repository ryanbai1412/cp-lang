// CSES 1130 Tree Matching: the maximum number of disjoint edges in a tree.
let n: 1..=2e5 = read()
domain V = 1..=n
let edges = read<[n - 1](u: V, v: V)>()
let t = Rooted(Graph.undirected(edges), root: V.first!)
// per vertex: (free, used) = best in its subtree with it unmatched / matched to a child;
// over the children: the sum of their bests and the best gain of matching one of them
let kids = Monoid<(Int, Int)>(id: (0, -inf), op: (a, b) => (a.0 + b.0, max(a.1, b.1)))
let dp = t.up(
    kids,
    lift: (x, c, p) => (max(x.0, x.1), x.0 + 1 - max(x.0, x.1)),
    finish: (s, v) => (s.0, s.0 + s.1),
)
let root = dp[V.first!]
print(max(root.0, root.1))
