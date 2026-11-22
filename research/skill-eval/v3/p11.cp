let n = read<1..=2e5>()
domain V = 1..=n
let t = Rooted(Graph.undirected(read<[n - 1](u: V, v: V)>()), root: V(1))
var dist = [V] v => 0
dist[V(1)] = V.all.sum(of: v => t.depth(v))
for p in t.Pre:
    let v = t.at[p]
    if let Some(u) = t.parent(v):
        dist[v] = dist[u] + n - 2 * t.size(v)
print(dist)
