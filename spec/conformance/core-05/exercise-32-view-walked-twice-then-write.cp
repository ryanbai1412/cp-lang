// expect: ok
// rule: 05.7.6
domain V = 1..=4
var a: [V]Int = 0
let i = V.first!
let j = V.last!
let l = V.first!
let r = V.last!
let evens = a.filter(x => x % 2 == 0)
let total = evens.sum()
let best = evens.argmax()
a[l] = 0
