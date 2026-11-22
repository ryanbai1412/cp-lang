// expect: E0601 at 5
// rule: 05.8.2
domain V = 1..=4
var a: [V]Int = 0
let l: V = read()
let r: V = read()
swap(mut a[l], mut a[r])
