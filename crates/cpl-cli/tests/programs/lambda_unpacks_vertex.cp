// A k-parameter lambda passed as a graph over tuple vertices `(V, K)`
// unpacks each vertex into its parts (decision 159).
domain V = 0..3
domain K = 0..2
fn mv(v: V, k: K) -> yields (to: (V, K), w: Int):
    if Int(v) + 1 < 3: yield (to: (V(Int(v) + 1), k), w: 4)
    if Int(k) == 0 and Int(v) + 1 < 3: yield (to: (V(Int(v) + 1), K(1)), w: 1)
let p = dijkstra((v, k) => mv(v, k), from: [((V(0), K(0)), 0)])
print(p.dist[(V(2), K(0))] ?? -1, p.dist[(V(2), K(1))] ?? -1)
let q = dijkstra(((v, k)) => mv(v, k), from: [((V(0), K(0)), 0)])
print(q.dist[(V(1), K(1))] ?? -1)
