// expect: ok, output "0 nodes\n4 nodes"
// rule: std.segtree MergeSegs prints its node count, past the shared empty node
use std.segtree.{MergeSegs}
domain D = 0..8
var q = MergeSegs<D>()
print(q)
let s = q.singleton(D(2))
print(q)
