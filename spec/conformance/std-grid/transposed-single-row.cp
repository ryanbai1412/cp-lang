// expect: ok, output "3 1 12"
use std.grid
domain R = 0..1
domain C = 0..3
let g: [R, C]Int = [R, C] (r, c) => 10 + Int(c)
let t = g.transposed()
print(C.size, R.size, t[C(2), R(0)])
