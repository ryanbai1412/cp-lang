// expect: E0602
// rule: 05.7.7
domain V = 1..=4
var a: [V]Int = 0
let i = V.first!
let j = V.last!
let l = V.first!
let r = V.last!
let p = mut a[i]
p += 1
a[j] = 0 // <- here
p += 1
