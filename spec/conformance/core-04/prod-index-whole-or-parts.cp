// expect: ok, output "7"
// rule: 04.2.11
domain R = 0..2
domain C = 0..3
domain P = (R, C)
let g: [R, C]Int = [P] p => 1
var s = 0
for p in P: s += g[p]
s += g[R(1), C(2)]
print(s)
