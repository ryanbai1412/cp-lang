// expect: E0602
// rule: 05.7.6
domain V = 1..=4
var a: [V]Int = 0
let i = V.first!
let j = V.last!
let l = V.first!
let r = V.last!
let w = a[l..r]
a[l] = 5 // <- here
print(w.sum())
