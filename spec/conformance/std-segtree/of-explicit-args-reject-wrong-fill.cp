// expect: E0302 at 3
use std.segtree.{SegTree}
domain V = 0..5
let t = SegTree<V, Int>.of(max, v => "x")
