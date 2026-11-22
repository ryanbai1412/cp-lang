// expect: ok, output "70\n4"
// rule: 04a.7.2
domain R = 0..4
domain C = 0..4
let g: [R, C]Int = [R, C] (r, c) => Int(r) * 10 + Int(c)
print(g[1..3, 2..4].sum())
print(g[1..3, 2..4].len)
