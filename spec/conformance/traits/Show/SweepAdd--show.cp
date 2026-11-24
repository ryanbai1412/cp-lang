// expect: ok, output "5 0 5"
use std.prefix.{SweepAdd}
domain D = 0..4
var s = SweepAdd<D, Int>()
s.add(1..=2, 5)
let a = s.advance(0)
let b = s.advance(1)
print("{s}", a, b)
