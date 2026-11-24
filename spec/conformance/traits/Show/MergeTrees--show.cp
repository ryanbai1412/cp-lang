// expect: ok, output "1 nodes 3 nodes"
// rule: std.segtree MergeTrees prints its node count, past the shared empty node
use std.segtree.{MergeTree, MergeTrees}
domain D = 0..8
var p = MergeTrees<D, Int>()
var t = MergeTree<D>()
p.set(mut t, D(3), 5)
let one = "{p}"
p.set(mut t, D(4), 1)
print(one, p)
