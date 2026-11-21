// Push-relabel on the Dinic network: 5000 vertices, 200000 arcs, capacities up to 10^6.
use std.flow.{FlowGraph}

domain V = 0..5000
var seed = 1
fn next(n: Int) -> Int:
    seed = seed * 48271 % 2147483647
    seed % n

var g = FlowGraph<V>()
for _ in 0..200000:
    let u = next(V.K)
    let v = next(V.K)
    g.add(V(u), V(v), next(1000000) + 1)
print(g.push_relabel(V.first, V.last))
