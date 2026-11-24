// CSES 1133 Tree Distances II: for every vertex, the sum of distances to all others.
let n: 1..=2e5 = read()
domain V = 1..=n
let edges: [0..n - 1](u: V, v: V) = read()
let t = Rooted<V>(Graph.undirected(edges), V.first)
type Part = (cnt: Int, sum: Int)
let add = Monoid<Part>(id: (0, 0), op: (a, b) => (a.cnt + b.cnt, a.sum + b.sum))
let all = t.reroot(add, (x, c, p) => (cnt: x.cnt, sum: x.sum + x.cnt), (u, v) => (cnt: u.cnt + 1, sum: u.sum))
print([all[v].sum for v in V])
