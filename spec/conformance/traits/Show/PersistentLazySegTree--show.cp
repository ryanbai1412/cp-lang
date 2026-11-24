// expect: ok, output "3 1 2"
use std.persistent.{PersistentLazySegTree}
var t = PersistentLazySegTree(Action<Int, Int>(apply: (d, x, span) => x + d * span), a: [3, 1, 2])
print(t)
