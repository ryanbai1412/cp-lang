// expect: E0302
// rule: 04.8.2
domain V = 1..=4
let v = V.first!
let u: V = v + 1 // <- here
