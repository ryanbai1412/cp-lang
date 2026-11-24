// expect: ok, output "YES\n1 2 3 | 1 3 4 | 9 2 3"
// model: VersionWithin
// Each tree's versions read through v.within(t) against one plain list per version: seeded
// random sets and range adds on random old versions, then len, each and items of a random one.
use std.persistent.{PersistentArray, PersistentLazySegTree, PersistentSegTree, Versioned, VersionWithin}
use std.rand.{Rng}
domain Q = 0..30
var rng = Rng(5)
let a: [Q]Int = [Q] _ => rng.int(0, 9)
var st = PersistentSegTree<Q, Int>(+, a: a)
var lz = PersistentLazySegTree<Q, Int, Int>(Action<Int, Int>(apply: (d, x, span) => x + d * span), a: a)
var pa = PersistentArray<Q, Int>(a: a)
var (sv, lv, pv) = ([st.base], [lz.base], [pa.base])
var (sb, lb, pb) = ([[a[i] for i in Q]], [[a[i] for i in Q]], [[a[i] for i in Q]])
fn agrees<S: Versioned<Q, Int>>(w: VersionWithin<S, Q, Int>, b: List<Int>) -> Bool:
    w.len == b.len and not w.is_empty and w.each().collect() == b and [(i.ord, x) for (i, x) in w.items] == [(i, b[i]) for i in 0..b.len]
var ok = true
for _ in 0..100:
    let (k, i, x) = (rng.int(0, sv.len - 1), rng.int(0, 29), rng.int(0, 9))
    sv.push(st.set(sv[k], Q(i), x))
    var s = [y for y in sb[k]]
    s[i] = x
    sb.push(s)
    let (l, r) = (rng.int(0, 29), rng.int(0, 29))
    let (lo, hi) = (min(l, r), max(l, r))
    lv.push(lz.apply(lv[k], Q(lo)..=Q(hi), x))
    var t = [y for y in lb[k]]
    for j in lo..=hi: t[j] += x
    lb.push(t)
    pv.push(pa.set(pv[k], Q(i), x))
    var p = [y for y in pb[k]]
    p[i] = x
    pb.push(p)
    let j = rng.int(0, sv.len - 1)
    if not (agrees(sv[j].within(st), sb[j]) and agrees(lv[j].within(lz), lb[j]) and agrees(pv[j].within(pa), pb[j])): ok = false
print(yes(ok))
domain P = 0..3
var t3 = PersistentLazySegTree<P, Int, Int>(Action<Int, Int>(apply: (d, x, span) => x + d * span), a: [P] i => i.ord + 1)
var a3 = PersistentArray<P, Int>(a: [P] i => i.ord + 1)
print("{t3.base.within(t3)} | {t3.apply(t3.base, P(1)..=P(2), 1).within(t3)} | {a3.set(a3.base, P(0), 9).within(a3)}")
