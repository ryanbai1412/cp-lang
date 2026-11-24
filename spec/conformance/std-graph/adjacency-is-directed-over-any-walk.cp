// expect: ok, output "2 1\n\n0\n2 1\n0"
// rule: 04.2.1
// `adjacency` is a std fn: each arc lands only in its `u`'s list, so a one-way arc's reverse is
// absent, and arcs may carry extra fields and come through a filter view.
use std.graph.{adjacency}
domain V = 0..3
let arcs = [(u: V(0), v: V(2), w: 4), (u: V(0), v: V(1), w: 6), (u: V(2), v: V(0), w: 1)]
let adj = adjacency(arcs)
print(adj[V(0)])
print(adj[V(1)])
print(adj[V(2)])
let even = adjacency(arcs.filter(a => a.w % 2 == 0))
print(even[V(0)])
print(even[V(2)].len)
