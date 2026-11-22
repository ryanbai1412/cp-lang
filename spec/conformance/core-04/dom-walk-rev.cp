// expect: ok, output "123321"
// rule: 04.2.1 04.2.15
domain V = 1..=3
var s = 0
for v in V: s = s * 10 + Int(v)
for v in V.rev: s = s * 10 + Int(v)
print(s)
