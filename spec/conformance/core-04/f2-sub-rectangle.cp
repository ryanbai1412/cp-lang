// expect: ok, output "4"
// rule: 04a.7.2
domain R = 0..3
domain C = 0..3
domain P = (R, C)
let g: [R, C]Int = [P] p => 1
print(g[..2, ..2].sum())
