// expect: ok, output "6 5 2 5 0\n3 YES 0\n9\n0\n2 1 2\n0\n0"
use std.fenwick.{Fenwick, Fenwick2D, DualFenwick}
domain D = 0..3
let a: [D]Int = [D] v => 1 + Int(v)
var f = Fenwick<D, Int>(a: Some(a))
print(f.all, f.fold(D(1)..), f.get(D(1)), f.suffix(D.cuts.nth(1)), f.suffix(D.cuts.nth(3)))
print(Int(f.first_gt(3)!), yes(f.first_gt(6) == None), Int(f.first_gt(-1)!))
f.set(D(1), 5)
print(f.all)
f.clear()
print(f.all)
domain R = 0..2
var g = Fenwick2D<R, R, Int>()
g.add(R(0), R(0), 1)
g.add(R(1), R(1), 2)
print(g.fold(R(0).., R(1)..), g.prefix(R.cuts.nth(1), R.cuts.nth(2)), g.get(R(1), R(1)))
g.clear()
print(g.get(R(1), R(1)))
var d = DualFenwick<D, Int>(a: Some(a))
d.clear()
print(d.get(D(0)))
