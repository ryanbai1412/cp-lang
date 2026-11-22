// CSES 1133 Tree Distances II: for every vertex, the sum of distances to all others.
let n: 1..=2e5 = read()
domain V = 1..=n
let edges = read<[n - 1](u: V, v: V)>()
let t = Rooted(Graph.undirected(edges), root: V.first!)
type Part = (cnt: Int, sum: Int)
let both = Monoid<Part>(id: (cnt: 0, sum: 0), op: (a, b) => (cnt: a.cnt + b.cnt, sum: a.sum + b.sum))
let all = t.reroot(
    both,
    lift: (x, c, p) => (cnt: x.cnt, sum: x.sum + x.cnt),
    finish: (s, v) => (cnt: s.cnt + 1, sum: s.sum),
)
print([all[v].sum for v in V])
