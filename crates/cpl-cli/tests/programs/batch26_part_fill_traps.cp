let n: Int = read()
var a: [0..n]Int = 0
var on: [0..n]Bool = false
var xs = [1, 2, 3, 4, 5, 6]
a[1..4] = 7
print(a)
a[..] = 2
a[2..=3] = 9
a[..2] = 5
a[4..] = 1
print(a)
on[1 - 1..3] = true
print([yes(x) for x in on])
xs[1..3] = 0
xs[4..=5] = 8
print(xs)
var g: [0..3, 0..4]Int = 0
g[1, ..] = 5
g[2, 1..3] = 7
g[.., 0] = 9
g[0..2, 3] = 4
print(g)
struct S:
    var lo: [0..4]Int
var cur = S([1, 2, 3, 4])
cur.lo[..] = 6
cur.lo[1..3] = 0
print(cur.lo)
print("before")
a[3..n + 2] = 1
print("not reached")
