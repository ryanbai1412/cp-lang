// expect: ok, output "version at node 2"
use std.dsu.{DsuNode, PersistentDsu}
use std.persistent.{PArena}
domain V = 0..2
var ar = PArena<DsuNode<V>>()
print(PersistentDsu<V>.new(mut ar))
