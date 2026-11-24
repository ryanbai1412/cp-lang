// expect: E0606
// rule: 08.2.5
// A struct type argument that fails its parameter's bound is reported at the type the user wrote, not inside a std body: `E`, an edge domain, is not an arc type of `V`.
use std.graph.{Graph}
domain V = 1..=3
domain E = 1..=2
let edges: [E](u: V, v: V) = [(u: 1, v: 2), (u: 2, v: 3)]
let g = Graph<V, E>.undirected(edges)  // <- here
print(g.deg(1))
