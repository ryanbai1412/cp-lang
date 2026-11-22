let (n, m) = read<(1..=1e5, 0..=2e5)>()
domain V = 1..=n
domain E = 0..m
let edges = read<[E](u: V, v: V, w: 1..=1e9)>()
let g = Graph.directed(edges)
var dist: [V]?Int = None
dist[V(1)] = 0
var pq = MinHeap<(Int, V)>()
pq.push((0, V(1)))
while let (du, u) = pq.pop():
    if du != dist[u]!:
        continue
    for a in g.out(u):
        let nd = du + a.w
        if dist[a.to].is_none_or(d => nd < d):
            dist[a.to] = nd
            pq.push((nd, a.to))
print([V] v => dist[v]!)
