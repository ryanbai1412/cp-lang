// expect: ok, output "6"
// rule: 04.2.10
domain R = 0..3
domain C = 0..3
domain P = (R, C)
var n = 0
for (r, c) in P:
    if let q = P.at((Int(r) - 1, Int(c))): n += 1
print(n)
