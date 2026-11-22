// expect: ok, output "1 7\n2 8\n3 9"
// rule: 04b.1.7
domain V = 1..=3
let a = [V] v => Int(v)
let r = [7, 8, 9].rekey(V.range)
for v in V: print(a[v], r[v])
