// expect: ok, output "-2 2 -5"
use std.poly.{Poly}
let p = Poly<Int>([1, 2]) - Poly<Int>([3, 0, 5])
print(p.at(0), p.at(1), p.at(2))
