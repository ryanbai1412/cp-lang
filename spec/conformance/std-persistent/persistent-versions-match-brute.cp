// expect: ok, output "YES"
use std.persistent.{PersistentLazySegTree, PersistentArray, Version}
use std.algebra.{Action, Monoid}
use std.rand.{Rng}
domain N = 0..9
let assign_sum = Action<Int, ?Int>(None, (f, g) => f ?? g, (f, x, span) => if let v = f: v * span else: x, Monoid<Int>(0, (a, b) => a + b))
let add_sum = Action<Int, Int>(0, (f, g) => f + g, (d, x, span) => x + d * span, Monoid<Int>(0, (a, b) => a + b))
var rng = Rng(11)
var a0: [N]Int = 0
for i in N: a0[i] = rng.int(-9, 9)
var t = PersistentLazySegTree<N, Int, ?Int>(assign_sum, a: a0)
var u = PersistentLazySegTree<N, Int, Int>(add_sum, a: a0)
var p = PersistentArray<N, Int>(a: a0)
var vs = [t.base]
var us = [u.base]
var ps = [p.base]
var brute = [[a0[i] for i in N]]
var ok = true
for _ in 0..300:
    let k = rng.int(0, vs.len - 1)
    let l = rng.int(0, 8)
    let r = rng.int(l, 8)
    let x = rng.int(-9, 9)
    let op = rng.int(0, 2)
    var b = [y for y in brute[k]]
    var nv = vs[k]
    var nu = us[k]
    var np = ps[k]
    if op == 0:
        nv = t.apply(vs[k], N(l)..=N(r), Some(x))
        for i in l..=r: b[i] = x
        np = ps[k]
        for i in l..=r: np = p.set(np, N(i), x)
        nu = us[k]
        for i in l..=r: nu = u.apply(nu, N(i)..=N(i), x - brute[k][i])
    elif op == 1:
        nv = t.set(vs[k], N(l), x)
        nu = u.set(us[k], N(l), x)
        np = p.set(ps[k], N(l), x)
        b[l] = x
    else:
        nu = u.apply(us[k], N(l)..=N(r), x)
        for i in l..=r: b[i] += x
        nv = vs[k]
        for i in l..=r: nv = t.set(nv, N(i), b[i])
        np = ps[k]
        for i in l..=r: np = p.set(np, N(i), b[i])
    vs.push(nv)
    us.push(nu)
    ps.push(np)
    brute.push(b)
    let j = rng.int(0, vs.len - 1)
    let ql = rng.int(0, 8)
    let qr = rng.int(ql, 8)
    var s = 0
    for i in ql..=qr: s += brute[j][i]
    if t.fold(vs[j], N(ql)..=N(qr)) != s or u.fold(us[j], N(ql)..=N(qr)) != s: ok = false
    if t.get(vs[j], N(ql)) != brute[j][ql] or p.get(ps[j], N(qr)) != brute[j][qr]: ok = false
    if t.all(vs[j]) != brute[j].sum() or u.all(us[j]) != brute[j].sum(): ok = false
print(ok)
