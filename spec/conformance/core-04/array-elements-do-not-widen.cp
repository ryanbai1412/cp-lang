// expect: E0302 at 3
// rule: 04.4.24
// Widening is for scalars: an array of a narrow range does not convert to `[3]Int` implicitly.
let xs: [3]1..=9 = [1, 2, 3]
var d: [3]Int = xs
d[0] = 100
print(d)
