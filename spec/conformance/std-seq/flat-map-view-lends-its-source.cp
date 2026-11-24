// expect: E0602
// rule: 05.5.3
// `flat_map(xs, x => [x])` keeps `xs`, so `xs` can't change while `f` lives.
use std.seq.{flat_map}
var xs = [1, 2, 3]
let ys = [4, 5, 6]
let f = flat_map(xs, x => [x])
xs.push(9) // <- here
var n = 0
for _ in f:
    n += 1
print(n)
