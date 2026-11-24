// expect: ok, output "4 9 2"
use std.persistent.{PersistentSegTree}
var t = PersistentSegTree(+, a: [4, 0, 2])
let v = t.set(t.base, 1, 9)
print(v.within(t))
