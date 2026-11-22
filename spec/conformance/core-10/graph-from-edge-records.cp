// expect: ok
// rule: 10.2.5
let n: 1..=100 = read()
let m: 0..=100 = read()
domain V = 1..=n
domain E = 0..m
let edges = read<[E](u: V, v: V)>()
let g = Graph.undirected(edges)
print(g.deg(V.first!))
