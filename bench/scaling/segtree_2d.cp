use std.segtree.{SegTree2D}
let n: Int = read()
domain V = 0..n
let reps = 1 << 17
var s = 0

let r2 = isqrt(n)
domain R = 0..r2
domain C = 0..r2
for _ in 0..100:
    s += SegTree2D<R, C, Int>(+, fill: 1).all // timed: segtree.SegTree2D.new [let: R = \sqrt n; C = \sqrt n]
var g = SegTree2D<R, C, Int>(+, fill: 1)
let inc: Int -> Int = x => x + 1 // within: segtree.SegTree2D.update
for i in 0..reps:
    let (x, y) = (i * 7919 % r2, i * 13 % r2)
    g.set(R(x), C(y), i % 5) // timed: segtree.SegTree2D.set [let: R = \sqrt n; C = \sqrt n]
    g.update(R(y), C(x), inc) // timed: segtree.SegTree2D.update [let: R = \sqrt n; C = \sqrt n]
    s += g.get(R(x), C(x)) // timed: segtree.SegTree2D.get [let: R = \sqrt n; C = \sqrt n]
    s += g.fold(R(x / 2)..R(x / 2 + r2 / 2), C(y / 2)..C(y / 2 + r2 / 2)) // timed: segtree.SegTree2D.fold [let: R = \sqrt n; C = \sqrt n]
    s += g.all // timed: segtree.SegTree2D.all [let: R = \sqrt n; C = \sqrt n]
print(s % 1000000007)
