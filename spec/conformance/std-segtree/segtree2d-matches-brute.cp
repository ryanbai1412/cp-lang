// expect: ok, output "12\n7 18\n19 10 0\nYES"
use std.segtree.{SegTree2D}
use std.rand.{Rng}
domain R = 0..3
domain C = 0..4
var g = SegTree2D<R, C, Int>(+, fill: 1)
print(g.all)
g.set(R(1), C(2), 7)
print(g.get(R(1), C(2)), g.all)
g.update(R(2), C(2), x => x + 1)
print(g.all, g.fold(R(0)..R(2), C(1)..C(3)), g.fold(R(0)..R(2), C(1)..C(1)))
var rng = Rng(4)
var a: [R, C]Int = 0
for r in R:
    for c in C: a[r, c] = rng.int(-9, 9)
var h = SegTree2D<R, C, Int>(max, a: a)
var ok = true
for _ in 0..200:
    let r = rng.int(0, 2)
    let c = rng.int(0, 3)
    let x = rng.int(-9, 9)
    h.set(R(r), C(c), x)
    a[R(r), C(c)] = x
    let r0 = rng.int(0, 2)
    let r1 = rng.int(r0, 2)
    let c0 = rng.int(0, 3)
    let c1 = rng.int(c0, 3)
    var best = -100
    for i in r0..=r1:
        for j in c0..=c1: best = max(best, a[R(i), C(j)])
    if h.fold(R(r0)..=R(r1), C(c0)..=C(c1)) != best: ok = false
print(ok)
