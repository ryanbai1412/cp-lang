// expect: ok, output "0 5 6\n5 0 7\n6 7 0\n-1"
// rule: std.pair.symmetric
// `symmetric` mirrors each pair's value to both `m[u, v]` and `m[v, u]` and puts `diag` on the diagonal.
use std.pair.{symmetric}
domain V = 0..3
var d: [Pair<V>]Int = 0
d[Pair<V>(V(0), V(1))] = 5
d[Pair<V>(V(0), V(2))] = 6
d[Pair<V>(V(1), V(2))] = 7
let m = symmetric<V, Int>(d, diag: 0)
for u in V: print([m[u, v] for v in V])
domain W = 0..1
let e: [Pair<W>]Int = 0
print(symmetric<W, Int>(e, diag: -1)[W(0), W(0)])
