use std.graph.{Graph}
domain V = 1..=4
domain E = 0..3
let es: [E](u: V, v: V) = [(u: 1, v: 2), (u: 2, v: 3), (u: 2, v: 4)]
let g = Graph.undirected(es)
print(g[V(2)].len, g[V(4)].len)
var total = 0
for (to: u, ..) in g[V(2)]:
    total += u.ord
print(total)
fn deg(v: V) -> Int = g[v].len
print(V.all.map(deg))
