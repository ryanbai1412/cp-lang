// expect: ok, output "1 2 0 1 0"
// rule: std.graph.Graph.only

use std.graph.{Graph}

domain V = 0..4
domain E = 0..4
let es: [E](u: V, v: V) = [(u: V(0), v: V(1)), (u: V(0), v: V(2)), (u: V(1), v: V(3)), (u: V(2), v: V(3))]
let g = Graph.directed(es)
let d1 = g.only(keep: v => v != V(1)).deg(V(0))
let h = g.only(from: v => v != V(1))
let d2 = h.deg(V(0))
let d3 = h.deg(V(1))

domain V2 = 0..3
let adj: [V2]List<V2> = [[V2(1), V2(2)], [V2(2)], []]
let tr = adj.only(keep: v => v != V2(1)).transpose().next(V2(2)).collect()
print(d1, d2, d3, tr.len, tr[0])
