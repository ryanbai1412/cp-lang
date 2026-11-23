// expect: ok, output "0\n7"
use std.segtree.{SegTree}
domain V = 0..5
let t = SegTree<V, Int>.of(max, v => 0)
print(t.all)
var u: SegTree<V, Int, Int> = t
u.set(V(2), 7)
print(u.all)
