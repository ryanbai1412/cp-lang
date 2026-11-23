// expect: ok, output "5 5 YES"
// rule: 04.2.12
// Two indices into a `[Pair<V>]T` name one unordered pair, in either order (decisions 164, 409).
domain V = 0..4
var d = [Pair<V>] p => 0
d[V(1), V(3)] = 5
print(d[V(1), V(3)], d[V(3), V(1)], yes(d[V(3), V(1)] == d[Pair<V>(V(1), V(3))]))
