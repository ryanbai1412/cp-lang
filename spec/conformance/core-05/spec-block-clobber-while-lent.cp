// expect: E0602 at 7
// rule: 05.8.4
domain V = 1..=4
let l = V.first!
let r = V.last!
var a: [V]Int = 0
fn clobber(): a[V.first!] = 0
let part = a[l..r]
clobber()                   // error: clobber writes a, and a is lent to part until line 5
print(part.sum())
