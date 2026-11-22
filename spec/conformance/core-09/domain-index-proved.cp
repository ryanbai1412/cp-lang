// expect: ok, output "12"
// rule: 09.3.1 09.5.1
domain V = 1..=3
let a = [V] v => Int(v) * 2
var s = 0
for v in V: s += a[v]
print(s)
