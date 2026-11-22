// expect: E0302 at 4
// rule: 04.4.5
domain V = 1..=4
var a: [V]Int = 0
let s = V.cuts(1)..V.cuts(3)
print(a[s].sum())
