// expect: ok, output "ok"
// input: "1 300\n"
// model: PersistentSegTree
// A PersistentSegTree over + against one plain list per version: seeded random set and update
// on a random old version, with get, fold, all, right_while and left_while read from a random
// version; and a chain of count versions checked with diff_kth. Reads `seed n`.
use std.rand.{Rng}
use std.persistent.{PersistentSegTree, Version}
let seed0 = read<Int>()
let n = read<Int>()
domain P = 0..14
var rng = Rng(seed0)
var t = PersistentSegTree(+, a: [P] _ => rng.int(0, 9))
t.reserve(64)
var vs: List<Version<PersistentSegTree<P, Int>>> = [t.base]
var bs: List<List<Int>> = [[t.get(t.base, i) for i in P]]
var c = PersistentSegTree<P, Int>(+, fill: 0)
var cs: List<Version<PersistentSegTree<P, Int>>> = [c.base]
var cb: List<List<Int>> = [[0 for _ in 0..14]]
var bad = ""
for k in 0..n:
    let op = rng.int(0, 3)
    let v = rng.int(0, vs.len - 1)
    let i = rng.int(0, 13)
    let x = rng.int(0, 9)
    var b = bs[v].clone()
    if op == 0:
        vs.push(t.set(vs[v], P(i), x))
        b[i] = x
        bs.push(b)
    elif op == 1:
        vs.push(t.update(vs[v], P(i), y => y * 2 + x))
        b[i] = b[i] * 2 + x
        bs.push(b)
    else:
        let last = cs.len - 1
        cs.push(c.update(cs[last], P(i), y => y + x % 3))
        var d = cb[last].clone()
        d[i] += x % 3
        cb.push(d)
    let q = rng.int(0, vs.len - 1)
    let a = bs[q]
    let lo = rng.int(0, 14)
    let hi = rng.int(lo, 14)
    let lim = rng.int(0, 40)
    var e = lo
    var acc = 0
    while e < 14 and acc + a[e] <= lim:
        acc += a[e]
        e += 1
    var f = hi
    acc = 0
    while f > 0 and acc + a[f - 1] <= lim:
        acc += a[f - 1]
        f -= 1
    let (cl, ch) = (rng.int(0, cs.len - 1), rng.int(0, cs.len - 1))
    let (l2, h2) = (min(cl, ch), max(cl, ch))
    var items: List<Int> = []
    for j in 0..14:
        for _ in 0..cb[h2][j] - cb[l2][j]: items.push(j)
    let kk = rng.int(0, items.len)
    let want_k = if kk < items.len: Some(items[kk]) else: None
    if t.get(vs[q], P(i)) != a[i]: bad = "op {k}: get v{q}"
    elif t.fold(vs[q], P.cuts.nth(lo)..P.cuts.nth(hi)) != sum(a[lo..hi]): bad = "op {k}: fold v{q} {lo}..{hi}"
    elif t.all(vs[q]) != sum(a): bad = "op {k}: all v{q}"
    elif t.right_while(vs[q], P.cuts.nth(lo), s => s <= lim).ord != e: bad = "op {k}: right_while v{q} {lo}"
    elif t.left_while(vs[q], P.cuts.nth(hi), s => s <= lim).ord != f: bad = "op {k}: left_while v{q} {hi}"
    elif c.diff_kth(cs[h2], cs[l2], kk).map(p => p.ord) != want_k: bad = "op {k}: diff_kth {h2} {l2} {kk}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
