let (n, m): (Int, Int) = read()
domain V = 1..=n
domain E = 0..m
let edges: [E](u: V, v: V, w: Int) = read()
let g = Graph.undirected(edges)
for u in V:
    for a in g[u]: print(u, a.to, edges[a.e].w)
let d = Graph.directed(edges)
print([d[u].len for u in V])
var tri: List<(V, V, Int)> = []
for e in E: tri.push((edges[e].u, edges[e].v, edges[e].w))
let h = Graph.undirected(tri)
for u in V:
    for (v, w) in h[u]: print(u, v, w)
let k = Graph.directed(tri)
print([k[u].len for u in V])
let pos: [E](V, V, Int) = [E] e => (edges[e].u, edges[e].v, edges[e].w * 2)
let gp = Graph.undirected(pos)
for u in V:
    for (v, w) in gp[u]: print(u, v, w)
