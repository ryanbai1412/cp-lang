// expect: ok, output "8 5"
// rule: 06.1.4
// `dijkstra((v, k) => ...)` over `(V, K)` states: a 2-parameter lambda unpacks the tuple vertex (decision 159; round 27, F5).
domain V = 0..3
domain K = 0..2
fn mv(v: V, k: K) -> yields (to: (V, K), w: Int):
    if Int(v) + 1 < 3: yield (to: (V(Int(v) + 1), k), w: 4)
    if Int(k) == 0 and Int(v) + 1 < 3: yield (to: (V(Int(v) + 1), K(1)), w: 1)
let p = dijkstra((v, k) => mv(v, k), from: [((V(0), K(0)), 0)])
print(p.dist[(V(2), K(0))] ?? -1, p.dist[(V(2), K(1))] ?? -1)
