// expect: ok, output "1 2 1"
use std.poly.{Poly}
let p = Poly<Int>([1, 1]) * Poly<Int>([1, 1])
print(p.at(0), p.at(1), p.at(2))
