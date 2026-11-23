let (n, m): (2..=500, 0..=1000) = read()
domain V = 1..=n
let es: [0..m](u: V, v: V, c: 1..=1e9) = read()
var g = FlowGraph<V>.new()
for e in es: g.add(e.u, e.v, Some(e.c))
print(g.max_flow(V(1), V(n)))
