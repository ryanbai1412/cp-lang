// expect: ok, output "no lines\n2 1"
use std.dp.{LiChao, Line}
var t = LiChao(max, over: 0..=100)
print(t)
t.add(Line(2, 1))
print(t)
