// expect: ok, output "-1 2"
use std.poly.{Poly}
let p = -Poly<Int>([1, -2])
print(p.at(0), p.at(1))
