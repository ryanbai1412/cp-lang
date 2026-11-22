// expect: ok, output "10\n10"
// rule: 04.2.4
domain V = 1..=4
let a = [V] v => Int(v)
print(a[V.range].sum())
print(a[a.keys].sum())
