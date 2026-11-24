// expect: E0608
// rule: 05.7.6
var xs = List(1, 2)
let ys = xs
print(xs.len) // <- here
