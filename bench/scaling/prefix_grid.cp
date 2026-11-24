// n: 2048
// fixed: C
// A 2-D table of n rows by 64 columns: its costs read |R| * |C| with |C| held at 64.
use std.prefix.{Prefix, Diff}
let n: Int = read()
domain R = 0..n
domain C = 0..64
let reps = 1 << 17
var s = 0
let a: [R, C]Int = [R, C] (r, c) => (Int(r) * 7 + Int(c)) % 11

for _ in 0..40:
    s += Prefix<R, C, Int>(a).fold(R(0)..R(n / 2), C(0)..C(32)) // timed: prefix.Prefix<R,C,T>.new
let p = Prefix<R, C, Int>(a)
for i in 0..reps:
    s += p.prefix(R.cuts.nth(i * 7 % n), C.cuts.nth(i % 64)) // timed: prefix.Prefix<R,C,T>.prefix
    s += p.fold(R(i % (n / 2))..R(i % (n / 2) + n / 2), C(i % 32)..C(i % 32 + 32)) // timed: prefix.Prefix<R,C,T>.fold

for _ in 0..100:
    var f = Prefix<R, C, Int>.filling() // timed: prefix.Prefix<R,C,T>.filling
    f.set(R(0), C(0), 1)
for _ in 0..4:
    var g = Prefix<R, C, Int>.filling()
    for r in R:
        for c in C:
            g.set(r, c, (Int(r) + Int(c)) % 5) // timed: prefix.FillPrefix<R,C,T>.set
var g = Prefix<R, C, Int>.filling()
for r in R:
    for c in C: g.set(r, c, (Int(r) + Int(c)) % 5)
for i in 0..reps:
    s += g.prefix(R.cuts.nth(i * 7 % n), C.cuts.nth(i % 64)) // timed: prefix.FillPrefix<R,C,T>.prefix
    s += g.fold(R(i % (n / 2))..R(i % (n / 2) + n / 2), C(i % 32)..C(i % 32 + 32)) // timed: prefix.FillPrefix<R,C,T>.fold

for _ in 0..100:
    var d = Diff<R, C, Int>() // timed: prefix.Diff<R,C,T>.new
    d.add(R(0)..R(1), C(0)..C(1), 1)
var d = Diff<R, C, Int>()
for i in 0..reps:
    d.add(R(i % (n / 2))..R(i % (n / 2) + n / 2), C(i % 32)..C(i % 32 + 32), i) // timed: prefix.Diff<R,C,T>.add
for _ in 0..40:
    s += d.finish()[R(n / 2), C(5)] // timed: prefix.Diff<R,C,T>.finish
print(s)
