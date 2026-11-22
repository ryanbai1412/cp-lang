// expect: R0401 at 4 (checked)
// rule: std.segtree.SegTree.new
use std.segtree.{SegTree}
domain D = 0..2
let a: [D]Int = [D] v => Int(v)
let s = SegTree<D, Int>(+, a, fill: 0)
print(s.all)
