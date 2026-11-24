// expect: ok, output "2 1 0\n1\n0 2 1 1\n0 1\n4 1"
// rule: std.graph undirected_of_pairs
// `Graph.undirected` and `Graph.directed` on a plain list of pairs give a `Graph` in compressed
// sparse rows (one arcs list, no list per vertex), arcs in pair order, a loop's two arcs adjacent.
domain V = 0..3
let g = Graph.undirected([(V(0), V(1)), (V(1), V(2)), (V(1), V(1))])
let d = Graph.directed([(V(2), V(0)), (V(0), V(2)), (V(2), V(1))])
print(d.deg(V(2)), d.deg(V(0)), d.deg(V(1)))
print(g.next(V(0)).collect())
print(g.next(V(1)).collect())
print(d.next(V(2)).collect())
print(g.deg(V(1)), g[V(2)].len)
