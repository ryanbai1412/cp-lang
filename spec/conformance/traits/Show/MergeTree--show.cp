// expect: ok, output "#0\n#1"
// rule: std.segtree MergeTree prints its root node in the pool, #0 for the all-zero tree
use std.segtree.{MergeTree, MergeTrees}
domain D = 0..8
var p = MergeTrees<D, Int>()
var t = MergeTree<D>()
print(t)
p.set(mut t, D(3), 5)
print(t)
