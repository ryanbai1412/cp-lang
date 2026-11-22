// expect: ok, output "2"
// rule: 01.8.5
domain R = 0..2
domain C = 0..2
let g = [R, C] (r, c) => Int(r) * 2 + Int(c) + 1
print(g[R(0), C(1)])
