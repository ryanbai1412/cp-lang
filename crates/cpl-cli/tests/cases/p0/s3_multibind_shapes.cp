// Synthesis #3 Leap 1: homogeneous multi-binding `let n, m: T = read()` and `[expr]T` input shapes.
let n, m: 1..=20 = read()
domain V = 1..=n
let xs: [n]Int = read()
let edges: [n - 1](u: V, v: V, w: Int) = read()
var lo, hi: Int = (0, m)
var s = 0
for i in 0..n: s += xs[i]
var ws = 0
for i in 0..(n - 1): ws += edges[i].w + Int(edges[i].u) + Int(edges[i].v)
lo += 1
hi += 2
print(n, m, s, ws, lo, hi)
