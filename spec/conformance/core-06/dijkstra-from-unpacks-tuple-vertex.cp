// expect: ok, output "5"
// rule: 01.8.2
// With `from:` given, a `(v, k) =>` lambda over `(V, K)` states still unpacks into a `V` and a `K`, not two `Int`s (A6).
domain V = 0..4
domain K = 0..2
let w: [V]Int = [3, 1, 4, 1]
fn step(v: V, k: K) -> List<((V, K), Int)> = [((v.next ?? v, k), w[v]), ((v, k.next ?? k), 0)]
let d = dijkstra(from: [(V(0), K(0))], (v, k) => step(v, k))
print(d[V(3), K(1)])
