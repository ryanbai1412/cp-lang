// expect: ok, output "11\n12345"
// rule: 04.2.9
domain R = 0..2
domain C = 0..3
domain P = (R, C)
let (r, c) = P.nth(4)
print(Int(r) * 10 + Int(c))
var s = 0
for (i, j) in P: s = s * 10 + Int(i) * 3 + Int(j)
print(s)
