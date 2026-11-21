let n = 8
var sums: [0..n]Int = 0
let a = [5, 7, 11]
for i in 0..3:
    let bit = 1 << i
    sums[bit..2 * bit] = sums[0..bit] + a[i]
print(sums)
var xs = [1, 2, 3, 4, 5, 6]
xs[1..6] = xs[0..5] + 10
print(xs)
xs[0..5] = 100 - xs[1..6]
print(xs)
xs[2..4] = xs[0..2] * 3
xs[..] = xs[..] ^ 1
print(xs)
var g: [0..3, 0..4]Int = [0..3, 0..4] (r, c) => r * 4 + c
g[2, ..] = g[0, ..] + 100
g[.., 3] = 2 * g[.., 0]
g[1, 1..3] = sums[5..7] - 1
print(g)
var u: [0..4]U64 = [1, 2, 3, 4]
u[0..2] = u[2..4] | 8
print(u)
var f = [1.5, 2.5, 3.5]
f[0..2] = f[1..3] * 2.0
print(f)
domain V = 0..5
var w: [V]Int = [V] v => v.ord
w[V(1)..V(3)] = w[V(2)..V(4)] + 7
print(w)
print("short")
xs[0..3] = xs[0..2] + 1
