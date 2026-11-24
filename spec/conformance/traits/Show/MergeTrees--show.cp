// expect: ok, output "4 nodes"
// rule: std.segtree MergeTrees prints its node count, past the shared empty node
use std.segtree.{MergeTree, MergeTrees}
domain D = 0..8
var p = MergeTrees<D, Int>()
var t = MergeTree<D>()
p.set(mut t, D(3), 5)
print(p)
