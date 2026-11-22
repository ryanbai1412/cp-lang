// expect: E0602 at 4
var xs = [3, 1, 2]
let w = xs[0..2]
xs.push(4)
print(w.sum())
