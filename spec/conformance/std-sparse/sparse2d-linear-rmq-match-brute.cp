// expect: ok, output "3 2 none 1 2\n1 4 none 1\nYES"
use std.sparse.{SparseTable2D, LinearRmq}
use std.rand.{Rng}
domain R = 0..2
domain C = 0..2
let g: [R, C]Int = [R, C] (r, c) => Int(r) * 2 + Int(c)
print(SparseTable2D(max, g).fold(R(0)..=R(1), C(0)..=C(1)), SparseTable2D(min, g).fold(R(1)..=R(1), C(0)..=C(1)), SparseTable2D(min, g).try_fold(R(0)..=R(1), C(1)..C(1)), SparseTable2D(min, g).try_fold(R(0)..=R(1), C(1)..=C(1)), SparseTable2D(min, g).get(R(1), C(0)))
domain P = 0..4
let xs: [P]Int = [3, 1, 1, 4]
print(LinearRmq(xs).argbest(P(0)..=P(3)), LinearRmq(xs, before: (x, y) => x > y).best(P(0)..=P(3)), LinearRmq(xs).try_best(P(1)..P(1)), LinearRmq(xs).try_best(P(0)..P(2)))
var rng = Rng(5)
domain Q = 0..300
let a: [Q]Int = [Q] _ => rng.int(0, 50)
let lm = LinearRmq(a)
var ok = true
for _ in 0..400:
    let lo = rng.int(0, 299)
    let hi = rng.int(lo, 299)
    var bi = lo
    for i in lo..=hi:
        if a[Q(i)] < a[Q(bi)]: bi = i
    if lm.argbest(Q(lo)..=Q(hi)) != Q(bi): ok = false
domain X = 0..7
domain Y = 0..9
let b: [X, Y]Int = [X, Y] (_, _) => rng.int(0, 99)
let st = SparseTable2D(min, b)
for _ in 0..300:
    let r0 = rng.int(0, 6)
    let r1 = rng.int(r0, 6)
    let c0 = rng.int(0, 8)
    let c1 = rng.int(c0, 8)
    var best = 1000
    for i in r0..=r1:
        for j in c0..=c1: best = min(best, b[X(i), Y(j)])
    if st.fold(X(r0)..=X(r1), Y(c0)..=Y(c1)) != best: ok = false
print(ok)
