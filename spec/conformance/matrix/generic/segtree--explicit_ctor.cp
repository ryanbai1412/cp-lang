// expect: ok, output "0"
use std.segtree.{SegTree}
domain V = 0..5
let t = SegTree<V, Int, Int>(max, [V] v => 0)
let u: SegTree<V, Int, Int> = t
print(u.all)
