// `st.at(i)` lends a leaf of a segment tree as a place; the write-back pulls its ancestors.
use std.segtree.{SegTree}
use std.lazy.{LazySegTree, DualSegTree, AddSum}
domain I = 0..5
let a: [I]Int = [1, 2, 3, 4, 5]
var seg = SegTree<I, Int>(+, a)
seg.at(I(1)) += 10
seg.at(I(3)) max= 9
seg.at(I(0)) = 7
var mx = SegTree<I, Int>(max, a)
for i in I:
    mx.at(i) *= 2
var lz = LazySegTree<I, Int>(AddSum, a)
lz.apply(I(1)..=I(3), 100)
lz.at(I(2)) -= 3
var d = DualSegTree<I, Int, Int>(0, (f, g) => f + g, (f, x) => f + x, a)
d.apply(I.all, 1)
d.at(I(4)) *= 10
print(seg.all, seg.fold(..I(2)), mx.all, mx.get(I(2)))
print(lz.all, lz.fold(I(2)..), d.get(I(4)), d.get(I(0)))
