// expect: ok, output "0 10 7 4 2"
use std.prefix.{ArithDiff}
domain D = 0..5
var d = ArithDiff<D, Int>()
d.add_arith(D(1)..=D(3), 10, -3)
d.add(D(4)..=D(4), 2)
print(d)
