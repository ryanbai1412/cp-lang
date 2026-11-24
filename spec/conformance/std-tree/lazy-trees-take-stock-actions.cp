// expect: ok, output "4 13 2\n6 4\n15 2\n9"
// rule: std.tree.LazyLinkCut
// LazyLinkCut and LazyEulerTourTree take a stock action such as `add`, resolved against `over:`
// (default `+`), as SegTree does, as well as a hand-written Action.
use std.tree.{LazyLinkCut, LazyEulerTourTree}

domain V = 0..4

var lc = LazyLinkCut<V, Int, Int>.new(add, Some([V] _ => 1))
lc.link(0, 1)
lc.link(1, 2)
lc.link(2, 3)
lc.apply_path(0, 2, 3)
print(lc.fold_path(1, 1), lc.fold_path(0, 3), lc.get(3) + 1)
var mx = LazyLinkCut<V, Int, Int>.new(add, Some([V] v => v.ord), over: max)
mx.link(0, 1)
mx.link(1, 2)
mx.apply_path(0, 1, 5)
print(mx.fold_path(0, 2), mx.get(2) + 2)
var et = LazyEulerTourTree<V, Int, Int>.new(add, Some([V] _ => 1))
et.link(0, 1)
et.link(1, 2)
et.apply_tree(0, 3)
print(et.fold_tree(2) + 3, et.get(3) + 1)
var hand = LazyLinkCut<V, Int, Int>.new(Action<Int, Int>(apply: (d, x, n) => x + d * n))
hand.link(0, 1)
hand.link(1, 2)
hand.apply_path(0, 2, 3)
print(hand.fold_path(0, 2))
