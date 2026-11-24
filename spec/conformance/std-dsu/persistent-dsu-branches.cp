// expect: ok, output "YES NO NO 1"
use std.dsu.{DsuNode, PersistentDsu}
use std.persistent.{PArena}
domain V = 0..4
var ar = PArena<DsuNode<V>>()
let d = PersistentDsu<V>.new(mut ar)
let v1 = d.unite(mut ar, V(0), V(1))
let v2 = d.unite(mut ar, V(2), V(3))
print(yes(v1.same(ar, V(0), V(1))), yes(v2.same(ar, V(0), V(1))), yes(v1.same(ar, V(2), V(3))), d.size(ar, V(0)))
