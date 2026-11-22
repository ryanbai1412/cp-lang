// expect: ok, output "YES\n5 6"
use std.persistent.{PersistentSegTree}
domain P = 0..3
let a: [P]Int = [1, 2, 3]
var t = PersistentSegTree(+, a: a)
t.reserve(100)
print(t.all(t.base) == 6)
let v = t.set(t.base, P(0), 0)
print(t.all(v), t.all(t.base))
