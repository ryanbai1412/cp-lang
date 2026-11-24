// expect: ok, output "0 0 0 35"
// With a replacing action (`assign`), an empty range folds to the identity at every end of the
// tree, as the plain fold does, instead of walking below the leaves.
use std.segtree.{SegTree}
use std.algebra.{assign}
domain V = 1..=10
var a = [V] v => v.ord
var sa = SegTree(+, a, act: assign)
sa.apply(V.nth(2)..=V.nth(5), Some(7))
print(sa.fold(..V.nth(0)), sa.fold(V.nth(3)..V.nth(3)), sa.fold(V.nth(9)..V.nth(9)), sa.fold(V.nth(1)..=V.nth(6)))
