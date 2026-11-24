// expect: ok, output "#0 #4"
// rule: std.segtree MergeSeg prints its root node in the pool, #0 for the empty set
use std.segtree.{MergeSeg, MergeSegs}
domain D = 0..8
var q = MergeSegs<D>()
let s = q.singleton(D(2))
print(MergeSeg<D>(), s)
