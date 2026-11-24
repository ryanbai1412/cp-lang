// expect: ok, output "2\n1\n1 2\n2 2"
// rule: 04.2.1
// `Bitset<D>.undirected`, `full` and `complement` are std fns: edges may carry extra fields
// and come through a filter view, and each edge lands in both ends' sets.
use std.bitset.{Bitset}
domain V = 0..4
let edges = [(u: V(0), v: V(1), w: 3), (u: V(0), v: V(2), w: 5), (u: V(1), v: V(3), w: 2)]
let odd = Bitset<V>.undirected(edges.filter(e => e.w % 2 == 1))
print(odd[V(0)].len)
print(odd[V(1)].len)
let side = Bitset<V>.full().without(V(0)).without(V(3))
print(side)
print(side.complement().len, odd[V(0)].complement().len)
