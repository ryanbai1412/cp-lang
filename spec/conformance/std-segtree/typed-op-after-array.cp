// expect: E0302 at 4
use std.segtree.{SegTree}
domain D = 0..3
let a: [D]Int = [1, 2, 3]
let s = SegTree<D, Int>(a, +)
