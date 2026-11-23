// cf1473e Minimum Path, revised: the graph may be disconnected; print -1 for unreachable vertices.
let n: 2..=2e5 = read()
let m: 1..=2e5 = read()
domain V = 1..=n
let g = Graph.undirected(read<[m](u: V, v: V, w: 1..=1e9)>())
domain K = 0..4                         // bit 1: an edge dropped; bit 2: an edge doubled

fn moves(v: V, k: K) -> yields (to: (V, K), w: Int):
    for a in g.out(v):
        yield (to: (a.to, k), w: a.w)
        if Int(k) & 1 == 0: yield (to: (a.to, K(Int(k) | 1)), w: 0)
        if Int(k) & 2 == 0: yield (to: (a.to, K(Int(k) | 2)), w: 2 * a.w)
        if Int(k) == 0: yield (to: (a.to, K(3)), w: a.w)

let p = dijkstra((v, k) => moves(v, k), from: [((V.first!, K(0)), 0)])
print([p[(v, K(3))] ?? -1 for v in V if v != V.first!])
