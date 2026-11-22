// expect: E0602 at 3
// rule: 05.7.6
var xs = List(1, 2)
let w = xs[0..1]
let ys = xs
print(w.sum(), ys.len)
