// expect: ok, output "1"
// rule: std.segtree.SegTree
use std.lazy.{Flip, flip_action}
domain I = 0..4
let act = flip_action<Int>(+)
var st = SegTree<I, Flip<Int>>(act.over, [I] i => if i.ord % 2 == 0: Flip(1, 0) else: Flip(0, 1), act: act)
st.apply(I(0)..I(3), true)
print(st.all.now)
