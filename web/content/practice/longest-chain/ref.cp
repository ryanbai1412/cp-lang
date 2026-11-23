let (n, m): (1..=2e5, 0..=2e5) = read()
domain V = 1..=n
let es: [0..m](u: V, v: V) = read()
let g = Graph.directed(es)
var len: [V]Int = 0
for u in topo(g)!:
    for v in g[u]:
        len[v] max= len[u] + 1
print(len.max()!)
