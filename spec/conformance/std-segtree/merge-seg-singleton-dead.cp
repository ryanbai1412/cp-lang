// expect: ok, output "YES"
use std.segtree.{MergeSegs}
domain K = 0..10
var p = MergeSegs<K>()
var m = p.singleton(K(3))
print(yes(p.first_alive(mut m, x => x.idx > 5) == None))
