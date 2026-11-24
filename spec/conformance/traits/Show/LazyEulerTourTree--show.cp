// expect: ok, output "0 4 4"
use std.tree.{LazyEulerTourTree}
use std.algebra.{add}
domain V = 0..3
var et = LazyEulerTourTree<V, Int, Int>(add)
et.link(1, 2)
et.apply_tree(2, 4)
print(et)
