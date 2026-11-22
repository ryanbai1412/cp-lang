// expect: E0602 at 3
// rule: 05.7.6 05.8.4 07.5.9
var xs = [3, 1, 2]
let w = xs[0..2]
xs.push(4)
print(w.sum())
