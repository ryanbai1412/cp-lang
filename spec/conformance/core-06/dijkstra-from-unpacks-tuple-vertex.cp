// expect: ok, output "8 4"
// rule: 01.8.2
// With `from:` given as `((V, K), W)` start pairs, a `(v, k) =>` lambda over `(V, K)` states unpacks into a `V` and a `K`, not two `Int`s (A6).
domain V = 0..4
domain K = 0..2
let w: [V]Int = [3, 1, 4, 1]
fn step(v: V, k: K) -> List<(to: (V, K), w: Int)> = [(to: (v.next ?? v, k), w: w[v]), (to: (v, k.next ?? k), w: 0)]
let d = dijkstra(from: [((V(0), K(0)), 0)], (v, k) => step(v, k))
print(d[(V(3), K(1))], d[(V(2), K(0))])
