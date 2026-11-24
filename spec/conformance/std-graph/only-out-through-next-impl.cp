// expect: ok, output "2\n0->2 2->3"
// `Graph`'s `out` is its `Next` impl, so `Only<V, Graph>.out` finds it and its `G.Arc` through that impl, and `Only`'s `Show` walks the kept arcs.
use std.graph.{Graph}
domain V = 0..4
domain E = 0..4
let es: [E](u: V, v: V) = [(u: 0, v: 1), (u: 0, v: 2), (u: 1, v: 3), (u: 2, v: 3)]
let g = Graph.directed(es)
for a in g.only(keep: v => v != 1).out(0): print(a.to.ord)
print(g.only(keep: v => v != 1))
