// expect: ok
// rule: 05.9.1
domain V = 1..=4
var a: [V]Int = 0
let i = V.first!
let j = V.last!
let l = V.first!
let r = V.last!
var a2 = a.clone()
for x in a.filter(x => x > 0): a2[l] += x
