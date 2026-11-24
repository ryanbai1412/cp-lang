// expect: E0608
// rule: 05.2.2
var xs = List(1, 2)
let ys = xs
print(xs.len + ys.len) // <- here
