// expect: E0602
// rule: 05.9.1
domain V = 1..=4
var a: [V]Int = 0
let i = V.first!
let j = V.last!
let l = V.first!
let r = V.last!
fn evens(xs: [V]Int) -> lends Filter<[V]Int, Int -> Bool> = xs.filter(x => x % 2 == 0)
for x in evens(a): a[l] += x // <- here
