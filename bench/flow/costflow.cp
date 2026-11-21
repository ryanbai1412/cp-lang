// Successive shortest paths: 1000 + 1000 vertex transport network, 60000 arcs with
// capacities up to 5 and costs up to 10^4, every left vertex supplying 5.
use std.flow.{CostFlow}

domain V = 0..2002
var seed = 1
fn next(n: Int) -> Int:
    seed = seed * 48271 % 2147483647
    seed % n

let (s, t) = (V(2000), V(2001))
var g = CostFlow<V>()
for i in 0..1000:
    g.add(s, V(i), 5, cost: 0)
    g.add(V(1000 + i), t, 5, cost: 0)
for _ in 0..60000:
    let u = next(1000)
    let v = next(1000)
    g.add(V(u), V(1000 + v), next(5) + 1, cost: next(10000))
let r = g.min_cost_flow(s, t)
print(r.flow, r.cost)
