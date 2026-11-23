// expect: ok, output "0 1 2 3 4\n5\n0 3 5\n3\n0\n1"
use std.graph.{max_clique, chromatic_number}
domain V = 0..5
var k5: [V]Bitset<V> = [Bitset.empty() for _ in V]
for a in V:
    for b in V:
        if a != b: k5[a].add(b)
print(max_clique(k5))
print(chromatic_number(k5))
domain U = 0..6
var g: [U]Bitset<U> = [Bitset.empty() for _ in U]
for (a, b) in [(1, 2), (1, 4), (2, 4), (3, 5), (5, 0), (0, 3), (0, 1)]:
    g[U(a)].add(U(b))
    g[U(b)].add(U(a))
print(max_clique(g))
print(chromatic_number(g))
domain Z = 0..1
let one: [Z]Bitset<Z> = [Bitset.empty()]
print(max_clique(one))
print(chromatic_number(one))
