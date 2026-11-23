// expect: ok, output "YES YES YES"
// rule: std.segtree.MergeSegs.first_alive
use std.segtree.{MergeSegs}
domain K = 0..10
var p = MergeSegs<K>()
var m = p.merge(p.merge(p.singleton(K(7)), p.singleton(K(3))), p.singleton(K(3)))
let a = p.first_alive(mut m, x => x.idx > 5)
let b = p.first_alive(mut m, x => true)
let c = p.first_alive(mut m, x => x.idx > 8)
print(yes(a == Some(K(7))), yes(b == Some(K(7))), yes(c == None))
