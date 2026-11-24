// expect: ok, output "version @5 version @8"
use std.persistent.{PersistentSegTree}
var t = PersistentSegTree(+, a: [1, 2, 3])
let v = t.set(t.base, 1, 9)
print(t.base, v)
