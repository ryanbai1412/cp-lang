// expect: ok, output "5 2 6 5\n2 YES 3\n10 4\n12\n5\n1 11 12 14 0\n4 5 5\n0 5\n8 0\n16\n11"
use std.prefix.{SweepAdd, ArithDiff}
domain D = 0..3
let p = Prefix<D, Int>([D] v => 1 + Int(v))
print(p.fold(D(1)..=D(2)), p.fold(D(1)..=D(1)), p.all, p.suffix(D.cuts.nth(1)))
print(Int(p.first_ge(3)!), yes(p.first_ge(7) == None), Int(p.first_gt(3)!))
var f = Prefix<D, Int>.filling(order: FillOrder.Reverse)
f.set(D(2), 4)
f.set(D(1), 6)
print(f.fold(D(1)..), f.fold(D(2)..=D(2)))
var g = Prefix<D, Int>.filling()
g.set(D(0), 5)
g.set(D(1), 7)
print(g.fold(D(0)..D(2)))
var l = Prefix<Int>.filling(3, order: FillOrder.Reverse)
l.set(2, 4)
l.set(1, 1)
print(l.fold(1..3))
domain E = 0..5
var ad = ArithDiff<E, Int>()
ad.add_arith(E(1)..=E(3), 10, 2)
ad.add(E(0)..E(2), 1)
print([ad.finish()[v] for v in E])
var s = SweepAdd<D, Int>(order: FillOrder.Reverse)
s.add(D(1)..=D(2), 4)
let a = s.advance(D(2))
s.add(D(0)..=D(1), 1)
let b = s.advance(D(1))
print(a, b, s.current)
var t = SweepAdd<E, Int>()
t.add(E(1)..=E(2), 5)
print(t.advance(E(0)), t.advance(E(1)))
domain X = 0..3
let xs: [X]Int = [2, 3, 4]
let (xpre, xsuf) = (xs.scan(*), xs.rev.scan(*))
domain Y = 0..1
let ys: [Y]Int = [5]
let (ypre, ysuf) = (ys.scan(+), ys.rev.scan(+))
let (xout, yout) = ([X] i => xpre[i.l] * xsuf[i.r], [Y] i => ypre[i.l] + ysuf[i.r])
print(xout[X(1)], yout[Y(0)])
domain R = 0..2
domain C = 0..3
let q = Prefix<R, C, Int>([R, C] (r, c) => 1 + 3 * Int(r) + Int(c))
print(q.fold(R(0)..=R(1), C(1)..=C(2)))
var h = Prefix<R, R, Int>.filling(order: FillOrder.Reverse)
h.set(R(1), R(1), 5)
h.set(R(1), R(0), 6)
print(h.fold(R(1)..=R(1), R(0)..=R(1)))
