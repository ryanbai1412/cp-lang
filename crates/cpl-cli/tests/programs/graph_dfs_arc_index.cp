// The depth-first searches resume at an arc by its index, and give the same
// answer whether `next` is a stored `Graph`, a table of lists, or a closure.
domain V = 0..7
use std.graph.{articulation_points, biconnected_components, bridges, euler_walk, find_cycle, two_edge_components}
domain E = 0..8
let edges: [E](u: V, v: V) = [E] e => [
    (u: V(0), v: V(1)), (u: V(1), v: V(2)), (u: V(2), v: V(0)), (u: V(2), v: V(3)),
    (u: V(3), v: V(4)), (u: V(4), v: V(5)), (u: V(5), v: V(3)), (u: V(5), v: V(6)),
][e.to_idx]
let g = Graph.undirected(edges)
let d = Graph.directed(edges)
let lists: [V]List<V> = [V] v => [a.to for a in g.out(v)]
let arcs: [V]List<(to: V, e: E)> = [V] v => [a for a in g.out(v)]
print(scc(d).comp, scc::<V>(v => [a.to for a in d.out(v)]).comp)
print(articulation_points(g), articulation_points(lists), articulation_points::<V>(v => lists[v]))
print(biconnected_components(g) == biconnected_components::<V>(v => lists[v]))
print(two_edge_components(g).comp, two_edge_components::<V>(v => arcs[v]).comp)
print(bridges::<V, E>(g), bridges::<V, E>(v => arcs[v]))
print(find_cycle(d), find_cycle::<V>(v => [a.to for a in d.out(v)]))
print(euler_walk(edges, directed: false) == none, euler_walk(edges, directed: true) == none)
