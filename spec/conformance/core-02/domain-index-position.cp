// expect: ok, output "4"
// rule: 02.8.1
domain V = 1..=3
let a = [V] v => Int(v) * 2
print(a[V(2)])
