// expect: E0602 at 4
// rule: 05.7.6
domain V = 1..=4
var a: [V]Int = 0
let w = a[2..=3]
a[2] = 5
print(w.sum())
