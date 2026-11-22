// expect: ok, output "3\n2"
// rule: 04.4.11 04.8.6
domain R = 0..2
domain C = 0..3
domain P = (R, C)
let g: [R, C]Int = [P] p => 1
print(g[1, ..].sum())
print(g[.., 2].sum())
