// Batch #13: `r.find(..)` on a range, a range with no end, and a graph from an array over `0..m`.
let (n, m): (Int, Int) = read()
domain V = 1..=n
let edges: [0..m](u: V, v: V) = read()
let g = Graph.directed(edges)
for u in V: print(u, g[u].len, g[u])
let both = Graph.undirected(edges)
print(sum(both[u].len for u in V))
print((0..).find(i => i * i >= n * 10), (3..).find(i => i % 7 == 0)!)
print((0..n).find(i => i * i >= 100), (0..n).find(i => i > 0))
print(V.all.find(v => g[v].len == 1), V.all.find(v => g[v].len > 5))
let r = 2..n
print(r.find(i => i % 2 == 1))
