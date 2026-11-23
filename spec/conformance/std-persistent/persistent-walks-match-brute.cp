// expect: ok, output "2 1\nYES"
use std.persistent.{PersistentSegTree}
use std.rand.{Rng}
domain P = 0..3
let xs: [P]Int = [1, 2, 3]
var t = PersistentSegTree<P, Int>(Monoid<Int>(0, (a, b) => a + b), a: xs)
print(t.right_while(t.base, P.cuts.nth(0), s => s <= 3).ord, t.left_while(t.base, P.cuts.nth(3), s => s <= 5).ord)
domain Q = 0..40
var rng = Rng(8)
var a: [Q]Int = [Q] _ => rng.int(0, 9)
var u = PersistentSegTree<Q, Int>(Monoid<Int>(0, (x, y) => x + y), a: a)
var vs = [u.base]
var arrs = [[a[Q(i)] for i in 0..40]]
var ok = true
for _ in 0..200:
    let k = rng.int(0, vs.len - 1)
    let i = rng.int(0, 39)
    let x = rng.int(0, 9)
    vs.push(u.set(vs[k], Q(i), x))
    var b = [y for y in arrs[k]]
    b[i] = x
    arrs.push(b)
    let j = rng.int(0, vs.len - 1)
    let from = rng.int(0, 40)
    let lim = rng.int(0, 60)
    var (e, s) = (from, 0)
    while e < 40 and s + arrs[j][e] <= lim:
        s += arrs[j][e]
        e += 1
    if u.right_while(vs[j], Q.cuts.nth(from), z => z <= lim).ord != e: ok = false
    var (g, s2) = (from, 0)
    while g > 0 and s2 + arrs[j][g - 1] <= lim:
        s2 += arrs[j][g - 1]
        g -= 1
    if u.left_while(vs[j], Q.cuts.nth(from), z => z <= lim).ord != g: ok = false
print(yes(ok))
