// expect: ok, output "0\n7"
use std.segtree.{SegTree}
domain V = 0..5
var t = SegTree<V, Int>.of(max, v => 0)
let u: SegTree<V, Int, Int> = t
print(u.all)
t.set(V(2), 7)
print(t.all)
