// expect: ok, output "0"
// rule: 05.7.6
domain V = 1..=4
var a: [V]Int = 0
let w = a[V(2)..=V(3)]
print(w.sum())
a[V(2)] = 5
