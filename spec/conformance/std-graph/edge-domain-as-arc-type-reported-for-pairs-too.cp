// expect: E0606
// rule: 08.2.5
// A list of plain pairs reads through `Graph.directed`'s pairs form, and the receiver's type arguments still meet `Graph`'s bounds: `E`, an edge domain, is not an arc type of `V`.
use std.graph.{Graph}
domain V = 1..=3
domain E = 0..2
let arcs: [E](V, V) = [(1, 2), (2, 3)]
let g = Graph<V, E>.directed(arcs)  // <- here
print(g.deg(1))
