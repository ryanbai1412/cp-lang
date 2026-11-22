// expect: E0601 at 7
// rule: 05.8.4
domain V = 1..=4
var a: [V]Int = 0
let i = V.first!
let j = V.last!
let l = V.first!
let r = V.last!
swap(mut a[i], mut a[j])                // error: a[i] and a[j] may be the same element
if i != j: swap(mut a[i], mut a[j])     // fine
