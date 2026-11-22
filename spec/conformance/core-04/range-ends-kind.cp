// expect: ok, output "9\n3"
// rule: 04.4.1 04.4.2 04.8.5
domain V = 1..=5
let a = [V] v => Int(v)
var s = 0
for v in V(2)..=V(4): s += a[v]
print(s)
let r = V.cuts(1)..=V.cuts(3)
print(r.len)
