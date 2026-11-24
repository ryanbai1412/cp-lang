// expect: ok, output "0 2 2"
use std.dsu.{DsuNode, PersistentDsu}
use std.persistent.{PArena}
domain V = 0..3
var ar = PArena<DsuNode<V>>()
let d = PersistentDsu<V>.new(mut ar).unite(mut ar, 2, 1)
print(d.within(ar))
