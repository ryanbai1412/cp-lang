// expect: ok, output "12 0 10"
domain R = 0..2
domain C = 0..3
let g: [R, C]Int = [R, C] (r, c) => 10 * Int(r) + Int(c)
let t = g.transposed()
print(t[C(2), R(1)], t[C(0), R(0)], t[C(0), R(1)])
