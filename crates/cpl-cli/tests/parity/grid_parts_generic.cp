// A row or a column of a grid is a 1-D array, and goes where a `[D]T` is asked for.
fn total<D, T: Num>(xs: [D]T) -> T:
    var s = T.zero
    for i in D: s += xs[i]
    s

fn last_index<D, T>(xs: [D]T) -> ?D:
    var at: ?D = none
    for i in D: at = some(i)
    at

fn dot<D>(a: [D]Int, b: [D]Int) -> Int:
    var s = 0
    for i in D: s += a[i] * b[i]
    s

var g: [0..2, 0..3]Int = 0
for i in 0..2:
    for j in 0..3: g[i, j] = i * 3 + j
print(total(g[1, ..]), total(g[.., 2]), total(g[0, 1..3]), total(g[0..1, 1]))
print(g[1, ..].argmax, g[.., 0].argmin, last_index(g[1, ..]), last_index(g[.., 1]))
print(scan(g[.., 1], (a, b) => a + b), dot(g[0, ..], g[1, ..]))
var one: [1..=2, 1..=3]Int = 7
one[2, 3] = 9
print(total(one[2, ..]), max(one[1, ..]), sum(one[.., 2]))
g[0, ..] += g[1, ..]
print(g[0, ..], dot(g[.., 0], g[.., 1]))
print(dot(g[0, ..], g[.., 0]))
