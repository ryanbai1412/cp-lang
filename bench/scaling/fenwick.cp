// fixed: C
use std.fenwick.{Fenwick, Fenwick2D, DualFenwick, RangeFenwick, XorFenwick, SparseFenwick2D}
let n: Int = read()
domain V = 0..n
domain C = 0..16
let reps = 1 << 17
var s = 0
let a: [V]Int = [V] v => Int(v) % 5 + 1

for _ in 0..400:
    let f = Fenwick<V, Int>(a) // timed: fenwick.Fenwick.new
    s += f.all
var f = Fenwick<V, Int>(a)
let tot = f.all
for i in 0..reps:
    f.add(V(i * 7919 % n), 1) // timed: fenwick.Fenwick.add
    s += f.all // timed: fenwick.Fenwick.all
    s += f.prefix(V.cuts.nth(i * 13 % n)) // timed: fenwick.Fenwick.prefix
    s += f.suffix(V.cuts.nth(i * 17 % n)) // timed: fenwick.Fenwick.suffix
    s += f.fold(V(i % (n / 2))..V(i % (n / 2) + n / 2)) // timed: fenwick.Fenwick.fold
    s += Int(f.first_ge(i % tot) ?? V.cuts.nth(0)) // timed: fenwick.Fenwick.first_ge
    s += Int(f.first_gt(i % tot) ?? V.cuts.nth(0)) // timed: fenwick.Fenwick.first_gt
    s += Int(f.kth(i % tot + 1) ?? V(0)) // timed: fenwick.Fenwick.kth
    s += f.get(V(i * 31 % n)) // timed: fenwick.Fenwick.get
    f.set(V(i * 37 % n), i % 7 + 1) // timed: fenwick.Fenwick.set
for _ in 0..1600:
    f.clear() // timed: fenwick.Fenwick.clear

for _ in 0..100:
    let g = Fenwick2D<V, C, Int>() // timed: fenwick.Fenwick2D.new
    s += g.get(V(0), C(0))
var g = Fenwick2D<V, C, Int>()
for i in 0..reps:
    g.add(V(i * 7919 % n), C(i % 16), 1) // timed: fenwick.Fenwick2D.add
    s += g.fold(V(i % (n / 2))..V(i % (n / 2) + n / 2), C(i % 8)..C(i % 8 + 8)) // timed: fenwick.Fenwick2D.fold
    s += g.prefix(V.cuts.nth(i * 13 % n), C.cuts.nth(i % 16)) // timed: fenwick.Fenwick2D.prefix
    s += g.get(V(i * 31 % n), C(i % 16)) // timed: fenwick.Fenwick2D.get
for _ in 0..100:
    g.clear() // timed: fenwick.Fenwick2D.clear

for _ in 0..400:
    let d = DualFenwick<V, Int>(a) // timed: fenwick.DualFenwick.new
    s += d.get(V(0))
var d = DualFenwick<V, Int>(a)
for i in 0..reps:
    d.add(V(i % (n / 2))..V(i % (n / 2) + n / 2), 1) // timed: fenwick.DualFenwick.add
    s += d.get(V(i * 31 % n)) // timed: fenwick.DualFenwick.get
for _ in 0..1600:
    d.clear() // timed: fenwick.DualFenwick.clear

for _ in 0..400:
    let r = RangeFenwick<V>(a) // timed: fenwick.RangeFenwick.new
    s += r.fold(V(0)..V(1))
var r = RangeFenwick<V>(a)
for i in 0..reps:
    r.add(V(i % (n / 2))..V(i % (n / 2) + n / 2), 1) // timed: fenwick.RangeFenwick.add
    s += r.prefix(V.cuts.nth(i * 13 % n)) // timed: fenwick.RangeFenwick.prefix
    s += r.fold(V(i % (n / 3))..V(i % (n / 3) + n / 3)) // timed: fenwick.RangeFenwick.fold
for _ in 0..1600:
    r.clear() // timed: fenwick.RangeFenwick.clear

for _ in 0..400:
    let x = XorFenwick<V, Int>(a) // timed: fenwick.XorFenwick.new
    s += x.prefix(V.cuts.nth(1))
var x = XorFenwick<V, Int>(a)
for i in 0..reps:
    x.add(V(i * 7919 % n), i) // timed: fenwick.XorFenwick.add
    s += x.prefix(V.cuts.nth(i * 13 % n)) // timed: fenwick.XorFenwick.prefix
    s += x.xor(V(i % (n / 2))..V(i % (n / 2) + n / 2)) // timed: fenwick.XorFenwick.xor

for _ in 0..reps:
    var e = SparseFenwick2D<Int>() // timed: fenwick.SparseFenwick2D.new
    e.add_point(0, 0)
for _ in 0..4:
    var e = SparseFenwick2D<Int>()
    for i in 0..n:
        e.add_point(i * 7919 % n, i * 31 % n) // timed: fenwick.SparseFenwick2D.add_point
for _ in 0..12:
    var e = SparseFenwick2D<Int>()
    for i in 0..n: e.add_point(i * 7919 % n, i * 31 % n)
    e.build() // timed: fenwick.SparseFenwick2D.build
var e = SparseFenwick2D<Int>()
for i in 0..n: e.add_point(i * 7919 % n, i * 31 % n)
e.build()
for i in 0..reps:
    let j = i % n
    e.add(j * 7919 % n, j * 31 % n, 1) // timed: fenwick.SparseFenwick2D.add
    s += e.fold(i % (n / 2)..i % (n / 2) + n / 2, i * 13 % (n / 2)..i * 13 % (n / 2) + n / 2) // timed: fenwick.SparseFenwick2D.fold
print(s)
