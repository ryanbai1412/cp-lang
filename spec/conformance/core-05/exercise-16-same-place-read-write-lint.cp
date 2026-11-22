// expect: L0601 at 10
// rule: 05.8.4
domain V = 1..=4
var a: [V]Int = 0
let i = V.first!
let j = V.last!
let l = V.first!
let r = V.last!
fn inc(mut x: Int) -> Int:
    x += 1
    x
a[l] += inc(mut a[l])
