// expect: ok, output "0"
// rule: 05.7.6
domain V = 1..=4
var a: [V]Int = 0
let w = a[2..=3]
print(w.sum())
a[2] = 5
