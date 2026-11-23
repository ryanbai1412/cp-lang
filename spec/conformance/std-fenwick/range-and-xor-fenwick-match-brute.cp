// expect: ok, output "YES"
use std.fenwick.{RangeFenwick, XorFenwick}
use std.rand.{Rng}
domain D = 0..7
var r = Rng(7)
var a: [D]Int = [D] v => r.int(0, 19)
var rf = RangeFenwick<D>(a: Some(a))
var xa: [D]Int = [D] v => r.int(0, 63)
var xf = XorFenwick<D, Int>(a: Some(xa))
var ok = true
for step in 0..200:
    let l = r.int(0, 6)
    let h = l + r.int(0, 6 - l)
    let x = r.int(-5, 5)
    if step % 2 == 0:
        rf.add(D(l)..=D(h), x)
        for i in l..=h: a[D(i)] = a[D(i)] + x
        let j = r.int(0, 6)
        xf.flip(D(j), x + 5)
        xa[D(j)] = xa[D(j)] ^ (x + 5)
    var s = 0
    var t = 0
    for i in l..=h:
        s += a[D(i)]
        t = t ^ xa[D(i)]
    if rf.fold(D(l)..=D(h)) != s or xf.xor(D(l)..=D(h)) != t: ok = false
if rf.fold(D(2)..D(2)) != 0 or xf.xor(D(2)..D(2)) != 0: ok = false
rf.clear()
if rf.fold(D(0)..=D(6)) != 0: ok = false
print(yes(ok))
