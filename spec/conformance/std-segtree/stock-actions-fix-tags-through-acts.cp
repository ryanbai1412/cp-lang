// expect: ok, output "12\n7\n6\n11\n0\n9"
// rule: std.segtree.SegTree.new
// A stock action's tag type comes from its std `Acts<T, F>` impl, solved at the ctor's
// `A: Acts<T, F>` bound: `add` tags with `T`, `assign` with `?T`, `affine` with `(T, T)`, and an
// `Action<T, F>` passes through as itself.
use std.algebra.{Action}
use std.tree.{LazyLinkCut}
var s = SegTree(+, [1, 2, 3], act: add)
s.apply(0..2, 3)
print(s.all)
var m = SegTree(max, [1, 5, 3], act: add)
m.apply(1..3, 2)
print(m.all)
var a = SegTree(+, [1, 2, 3], act: assign)
a.apply(0..3, Some(2))
print(a.all)
var f = SegTree(+, [1, 2, 3], act: affine)
f.apply(0..2, (2, 1))
print(f.all)
let neg = Action<Int, Bool>(id: false, compose: (x, y) => x != y, apply: (t, v, n) => if t: -v else: v)
var g = SegTree(+, [1, 2, 3], act: neg)
g.apply(0..2, true)
print(g.all)
domain V = 0..3
var lc = LazyLinkCut<V, Int, Int>(add)
lc.link(0, 1)
lc.link(1, 2)
lc.apply_path(0, 2, 3)
print(lc.fold_path(0, 2))
