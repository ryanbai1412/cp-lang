// expect: ok, output "0 0 1 1\n0 0 0 1 0 1 1 1\n0 1 2\n2 1\n0 2\n2 1\nYES\n0 1 2\n3 1"
use std.graph.{de_bruijn, directed_of_pairs, undirected_of_pairs, adjacency, prim, max_clique, chromatic_number}
use std.bitset.{Bitset}
print(de_bruijn(2, 2))
print(de_bruijn(2, 3))
print(de_bruijn(3, 1))
domain V = 0..3
print(directed_of_pairs([(V(0), V(2)), (V(0), V(1))])[V(0)])
print(undirected_of_pairs([(V(0), V(1)), (V(1), V(2))])[V(1)])
print(adjacency([(u: V(0), v: V(2)), (u: V(0), v: V(1))])[V(0)])
var w: [V, V]?Int = None
w[V(0), V(1)] = Some(1)
w[V(1), V(0)] = Some(1)
w[V(1), V(2)] = Some(2)
w[V(2), V(1)] = Some(2)
w[V(0), V(2)] = Some(5)
w[V(2), V(0)] = Some(5)
let pr = prim(w)!
print(yes(pr.total == 3 and pr.parent[V(0)] == None and pr.parent[V(1)] == Some(V(0)) and pr.parent[V(2)] == Some(V(1))))
domain U = 0..4
let adj: [U]Bitset<U> = [Bitset.of([U(1), U(2)]), Bitset.of([U(0), U(2)]), Bitset.of([U(0), U(1), U(3)]), Bitset.of([U(2)])]
print(max_clique(adj))
let tri: [V]Bitset<V> = [Bitset.of([V(1), V(2)]), Bitset.of([V(0), V(2)]), Bitset.of([V(0), V(1)])]
let none3: [V]Bitset<V> = [Bitset.empty(), Bitset.empty(), Bitset.empty()]
print(chromatic_number(tri), chromatic_number(none3))
