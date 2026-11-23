// CF 1843D Apple Tree
// in: t; n, n-1 edges u v, q, q queries x y | out: per query leaves under x times leaves under y
@!multitest
let n: 2..=2e5 = read()
domain V = 1..=n
let t = Rooted(Graph.undirected(read<[n - 1](u: V, v: V)>()), root: V.first!)

var leaves: [V]Int = 0
for v in t.order.rev:
    if t.size(v) == 1: leaves[v] = 1
    if let p = t.parent(v): leaves[p] += leaves[v]

for _ in 0..read<1..=2e5>():
    let (x, y) = read<(V, V)>()
    print(leaves[x] * leaves[y])
