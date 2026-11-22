// expect: R0401 at 4 (checked)
// rule: std.segtree.SegTree.right_while
use std.segtree.{SegTree}
domain D = 0..2
let s = SegTree<D, Int>(+, fill: 1)
print(yes(s.right_while(D.cuts.first!, x => x < 0) == D.cuts.first!))
