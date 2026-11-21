var a: [0..6]Int = [5, 1, 4, 2, 6, 3]
a[1..4] += 10
print(a)
a[..] max= 4
print(a)
a[2..=3] -= 1
a[..2] *= 2
a[4..] min= 5
print(a)
var xs = [1, 2, 3, 4]
xs[1..3] += 5
xs[..] min= 7
print(xs)
var g: [0..3, 0..4]Int = 9
let area = 5
g[1, 1..=2] min= area
g[.., 0] += 1
g[2, ..] max= 12
g[0..2, 3] -= 4
print(g)
var f: [0..3]F64 = 1.5
f[1..] *= 2.0
print(f)
struct Cur:
    lo: [0..4]Int
var cur = Cur([4, 3, 2, 1])
cur.lo[..] += 1
cur.lo[1..3] max= 4
print(cur.lo)
domain V = 1..=5
var d: [V]Int = [V] v => v.label
d[V(2)..=V(4)] *= 3
d[..] min= 10
print(d)
print(a[..] max= 0, a[..] min= 0)
print(a)
