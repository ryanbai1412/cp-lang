// expect: ok, output "5 5 0\n6 4"
// rule: std.pair.Pair
// A `[Pair<V>]T` table takes `d[u, v]` because std's `Pair.new` takes two
// positions, and `Pair<V>.count` is its `impl Index` count, as for any
// `impl Index` type.
domain V = 0..4
var d: [Pair<V>]Int = 0
d[V(0), V(2)] = 5
print(d[V(2), V(0)], d[Pair(V(0), V(2))], d[V(1), V(3)])
let t: [Pair<V>][V]Int = 4
print(Pair<V>.count, t[Pair(V(0), V(1))][V(2)])
