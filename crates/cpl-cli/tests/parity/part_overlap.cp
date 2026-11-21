var b = [1, 2, 3, 4, 5]
b[1..5] min= b[0..4]
print(b)
var c = [5, 4, 3, 2, 1]
c[0..4] max= c[1..5]
print(c)
var g: [0..2, 0..4]Int = [0..2, 0..4] (r, k) => 10 * r + k
g[0, 1..4] min= g[0, 0..3]
print(g)
var w = [1, 1, 2, 3, 5, 8]
w[1..=5] -= w[0..5]
print(w)
w[0..3] += w[3..6]
w[..] ^= w[..]
print(w)
var d = [1, 2, 3, 4, 5]
d[1..5] = d[0..4]
print(d)
g[1, ..] += g[0, ..]
g[.., 0] *= g[.., 3]
print(g)
