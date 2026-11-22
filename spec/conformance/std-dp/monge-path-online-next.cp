// expect: ok, output "1\n1 0 1 YES 3"
use std.dp.{MongePath}
var p = MongePath([0, 1, 2], (a, b) => (b - a) * (b - a))
print(p.len)
let a = p.next()
print(a.val, a.prev, p.val(1), yes(p.next() == (val: 2, prev: 1)), p.len)
