use std.prefix.{Prefix, Diff, SweepAdd, ArithDiff, FillOrder, prefix_fold}
let n: Int = read()
domain V = 0..n
let reps = 1 << 17
var s = 0
let a: [V]Int = [V] v => Int(v) * 7 % 11

for _ in 0..500:
    s += a.scan(+).all // timed: prefix.scan
for _ in 0..500:
    s += prefix_fold(+, a)[V.cuts.nth(n)] // timed: prefix.prefix_fold
for _ in 0..500:
    let p = Prefix<V, Int>(a) // timed: prefix.Prefix.new
    s += p.all
let p = Prefix<V, Int>(a)
for i in 0..reps:
    s += p.all // timed: prefix.Prefix.all
    s += p.at(V.cuts.nth(i % n)) // timed: prefix.Prefix.at
    s += p.prefix(V.cuts.nth(i * 7 % n)) // timed: prefix.Prefix.prefix
    s += p.suffix(V.cuts.nth(i * 13 % n)) // timed: prefix.Prefix.suffix
    s += p.cut(i * 17 % n) // timed: prefix.Prefix.cut
    s += p.fold(V(i % (n / 2))..V(i % (n / 2) + n / 2)) // timed: prefix.Prefix.fold
    s += p.cyclic_sum(a.ring.span(V(i % n), n / 2)) // timed: prefix.Prefix.cyclic_sum
    s += Int(p.first_ge(i % p.all) ?? V.cuts.nth(0)) // timed: prefix.Prefix.first_ge
    s += Int(p.first_gt(i % p.all) ?? V.cuts.nth(0)) // timed: prefix.Prefix.first_gt
for _ in 0..1200:
    var f = Prefix<V, Int>.filling() // timed: prefix.Prefix.filling
    f.set(V(0), 1)
for _ in 0..8:
    var g = Prefix<V, Int>.filling()
    for i in 0..n:
        g.set(V(i), i % 5) // timed: prefix.FillPrefix.set
var f = Prefix<V, Int>.filling()
for i in 0..n: f.set(V(i), i % 5)
for i in 0..reps:
    s += f.prefix(V.cuts.nth(i * 7 % n)) // timed: prefix.FillPrefix.prefix
    s += f.at(V.cuts.nth(i % n)) // timed: prefix.FillPrefix.at
    s += f.cut(i * 17 % n) // timed: prefix.FillPrefix.cut
    s += f.fold(V(i % (n / 2))..V(i % (n / 2) + n / 2)) // timed: prefix.FillPrefix.fold

for _ in 0..1200:
    var d = Diff<V, Int>() // timed: prefix.Diff.new
    d.add(V(0)..V(1), 1)
var d = Diff<V, Int>()
for i in 0..reps:
    d.add(V(i % (n / 2))..V(i % (n / 2) + n / 2), i) // timed: prefix.Diff.add
for _ in 0..500:
    s += d.finish()[V(n / 2)] // timed: prefix.Diff.finish

for _ in 0..1500:
    var w = SweepAdd<V, Int>() // timed: prefix.SweepAdd.new
    w.add(V(0)..V(1), 1)
for _ in 0..8:
    var w = SweepAdd<V, Int>()
    for i in 0..n:
        w.add(V(i)..V((i + n / 3) % n), 1) // timed: prefix.SweepAdd.add
        s += w.advance(V(i)) // timed: prefix.SweepAdd.advance
        s += w.current // timed: prefix.SweepAdd.current

for _ in 0..800:
    var e = ArithDiff<V, Int>() // timed: prefix.ArithDiff.new
    e.add(V(0)..V(1), 1)
var e = ArithDiff<V, Int>()
for i in 0..reps:
    e.add_arith(V(i % (n / 2))..V(i % (n / 2) + n / 2), i, 3) // timed: prefix.ArithDiff.add_arith
    e.add(V(i % (n / 3))..V(i % (n / 3) + n / 3), i) // timed: prefix.ArithDiff.add
for _ in 0..200:
    s += e.finish()[V(n / 2)] // timed: prefix.ArithDiff.finish
print(s)
