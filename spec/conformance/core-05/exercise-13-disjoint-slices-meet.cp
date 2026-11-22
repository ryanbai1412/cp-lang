// expect: ok
// rule: 05.8.2
domain V = 1..=8
var a: [V]Int = 0
let i = V.first!
let j = V.last!
let l = V.first!
let r = V.last!
fn solve(mut lo: [V]Int, mut hi: [V]Int) = todo
let m = l.next(5)!
solve(mut a[l..m], mut a[m..r])
