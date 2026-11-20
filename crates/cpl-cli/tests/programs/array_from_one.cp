// An array indexed from a first index other than 0.
var b: [1..=9]Int = 0
b[3] = 2
b[9] += 7
b[1] = 1
print(b[3], b[9], b)
var c: [-2..3]Int = 5
c[-2] = 1
c[2] = 9
print(c, sum(c[i] for i in -2..3))
fn total(xs: [1..=9]Int) -> Int = sum(xs[i] for i in 1..=9)
print(total(b))
var g: [1..=2, 1..=3]Int = 0
g[2, 3] = 6
g[1, 1] = 1
print(g[2, 3] + g[1, 1])
let i: Int = read()
print(b[i])
