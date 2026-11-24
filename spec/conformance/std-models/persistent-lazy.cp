// expect: ok, output "ok"
// input: "1 300\n"
// model: PersistentLazySegTree
// A PersistentLazySegTree with range-add tags over + against one plain list per version: seeded
// random apply and set on a random old version, with get, fold and all read from a random
// version after each op. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.persistent.{PersistentLazySegTree, Version}
let seed0 = read<Int>()
let n = read<Int>()
domain L = 0..13
var rng = Rng(seed0)
let act = Action<Int, Int>(apply: (d, x, span) => x + d * span)
let fill = rng.int(0, 1) == 0
var t = if fill: PersistentLazySegTree<L, Int, Int>(act, fill: 3) else: PersistentLazySegTree(act, a: [L] i => i.ord)
var vs: List<Version<PersistentLazySegTree<L, Int, Int>>> = [t.base]
var bs: List<List<Int>> = [[if fill: 3 else: j for j in 0..13]]
var bad = ""
for k in 0..n:
    let v = rng.int(0, vs.len - 1)
    let lo = rng.int(0, 13)
    let hi = rng.int(lo, 13)
    let x = rng.int(-9, 9)
    let i = rng.int(0, 12)
    var b = bs[v].clone()
    if rng.int(0, 2) > 0:
        vs.push(t.apply(vs[v], L.cuts.nth(lo)..L.cuts.nth(hi), x))
        for j in lo..hi: b[j] += x
    else:
        vs.push(t.set(vs[v], L(i), x))
        b[i] = x
    bs.push(b)
    let q = rng.int(0, vs.len - 1)
    let a = bs[q]
    let ql = rng.int(0, 13)
    let qh = rng.int(ql, 13)
    if t.get(vs[q], L(i)) != a[i]: bad = "op {k}: get v{q} {i}"
    elif t.fold(vs[q], L.cuts.nth(ql)..L.cuts.nth(qh)) != sum(a[ql..qh]): bad = "op {k}: fold v{q} {ql}..{qh}"
    elif t.all(vs[q]) != sum(a): bad = "op {k}: all v{q}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
