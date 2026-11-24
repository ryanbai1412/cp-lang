// expect: E0602
// rule: 05.7.6
domain V = 1..=4
var a: [V]Int = 0
let i = V.first!
let j = V.last!
let l = V.first!
let r = V.last!
fn clobber(): a[l] = 0
let part = a[l..r]
clobber() // <- here
print(part.sum())
