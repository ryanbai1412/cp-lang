// expect: ok, output "6 1"
// rule: 04.2.9
// Shortest paths over `(V, K)` states, one free edge allowed; interpreter and C++ agree.
domain V = 0..4
domain K = 0..2
let es = [(V(0), V(1), 5), (V(1), V(3), 1), (V(0), V(2), 1), (V(2), V(3), 9)]
fn next(s: (V, K)) -> yields (to: (V, K), w: Int):
    let (v, k) = s
    for (a, b, w) in es:
        if a == v:
            yield (to: (b, k), w: w)
            if Int(k) == 0: yield (to: (b, K(1)), w: 0)
let p = dijkstra(next, from: [((V(0), K(0)), 0)])
print(p.dist[(V(3), K(0))] ?? -1, p.dist[(V(3), K(1))] ?? -1)
