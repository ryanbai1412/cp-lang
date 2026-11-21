// Kruskal: edges by weight, joined with a Dsu; prints the forest's weight
// and its number of edges.
let (n, m): (1..=1e6, 0..=2e6) = read()
domain V = 1..=n
domain E = 0..m
let edges: [E](u: V, v: V, w: 1..=1e9) = read()
var dsu = Dsu<V>.new()
var (weight, used) = (0, 0)
for e in [e for e in E].sorted(by: e => edges[e].w):
    if dsu.unite(edges[e].u, edges[e].v):
        weight += edges[e].w
        used += 1
print(weight, used)
