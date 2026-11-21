// Strongly connected components of a directed graph; prints their number
// and the sum of v * comp(v), with comp in topological order.
let (n, m): (1..=1e6, 0..=2e6) = read()
domain V = 1..=n
domain E = 0..m
let edges: [E](u: V, v: V) = read()
let (C, comp) = scc(Graph.directed(edges))
var total = 0
for v in V: total += v.to_idx * comp[v].to_idx
print(C.K, total)
