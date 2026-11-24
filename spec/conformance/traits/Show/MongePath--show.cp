// expect: ok, output "0 1 5"
use std.dp.{MongePath}
var p = MongePath([0, 1, 3], (a, b) => (b - a) * (b - a))
p.next()
p.next()
print(p)
