// expect: ok, output "YES NO NO 1"
use std.dsu.{PersistentDsu}
domain V = 0..4
var d = PersistentDsu<V>()
let v1 = d.unite(d.base, V(0), V(1))
let v2 = d.unite(d.base, V(2), V(3))
print(yes(d.same(v1, V(0), V(1))), yes(d.same(v2, V(0), V(1))), yes(d.same(v1, V(2), V(3))), d.size(d.base, V(0)))
