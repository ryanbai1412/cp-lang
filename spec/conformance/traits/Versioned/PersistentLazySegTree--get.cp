// expect: ok, output "3 9 2"
use std.persistent.{PersistentLazySegTree}
var t = PersistentLazySegTree(Action<Int, Int>(apply: (d, x, span) => x + d * span), a: [3, 1, 2])
let v = t.set(t.base, 1, 9)
print(v.within(t))
