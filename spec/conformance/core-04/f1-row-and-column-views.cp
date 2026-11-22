// expect: ok, output "6\n2"
// rule: 04a.7.1
domain R = 0..2
domain C = 0..3
domain P = (R, C)
let g: [R, C]Int = [P] p => 1
var s = 0
for (c, x) in g[R(1)].items: s += Int(c) + x
print(s)
print(g[.., C(2)].sum())
