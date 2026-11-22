// expect: ok, output "2"
// rule: 04b.1.11
domain V = 1..=5
let a = [V] v => Int(v)
print(a[V(2)..=V(4)][V(2)])
