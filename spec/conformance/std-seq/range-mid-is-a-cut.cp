// expect: ok, output "3 2\n1 1"
use std.range.{mid}
domain W = 0..6
let r = W(1)..W(5)
print(Int(mid(r)), Int(mid(W(2)..W(2))))
print(Int(mid(W(1)..W(2))), Int(mid(W(0)..W(3))))
