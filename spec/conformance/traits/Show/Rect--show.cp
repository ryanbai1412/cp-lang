// expect: ok, output "1 2 4 5"
// engines: next
domain R = 0..2
domain C = 0..3
let g: [R, C]Int = [R, C] (r, c) => Int(r) * 3 + Int(c)
let q = g[R(0)..=R(1), C(1)..=C(2)]
print(q)
