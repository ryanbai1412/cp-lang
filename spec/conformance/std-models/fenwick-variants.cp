// expect: ok, output "ok"
// input: "1 300\n"
// model: DualFenwick
// model: RangeFenwick
// model: XorFenwick
// model: Fenwick2D
// model: SparseFenwick2D
// The other Fenwick trees against plain arrays: range add with point get (DualFenwick), range add
// with range sum (RangeFenwick), point xor with range xor (XorFenwick), each built from a random
// array; a 2-D grid (Fenwick2D) and an offline one over registered points (SparseFenwick2D).
// Every op is followed by a random read of each. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.fenwick.{DualFenwick, RangeFenwick, XorFenwick, Fenwick2D, SparseFenwick2D}
domain V = 0..23
domain R = 0..5
domain C = 0..7
let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
let a0 = [rng.int(-9, 9) for _ in 0..23]
let x0 = [rng.int(0, 63) for _ in 0..23]
var d = DualFenwick<V, Int>([V] i => a0[i.ord])
var rf = RangeFenwick<V>([V] i => a0[i.ord])
var xf = XorFenwick<V, Int>([V] i => x0[i.ord])
var g = Fenwick2D<R, C, Int>()
var pts: List<(Int, Int)> = [(rng.int(-20, 20), rng.int(-20, 20)) for _ in 0..12]
var sf = SparseFenwick2D<Int>()
for (x, y) in pts: sf.add_point(x, y)
sf.add_point(pts[0].0, pts[0].1)
sf.build()
var a = a0.clone()
var xa = x0.clone()
var ga = List.filled(5 * 7, 0)
var sv = List.filled(pts.len, 0)
var bad = ""
for k in 0..n:
    let op = rng.int(0, 4)
    let lo = rng.int(0, 23)
    let hi = rng.int(lo, 23)
    let x = rng.int(-9, 9)
    if op == 0:
        d.add(V.cuts.nth(lo)..V.cuts.nth(hi), x)
        rf.add(V.cuts.nth(lo)..V.cuts.nth(hi), x)
        for j in lo..hi: a[j] += x
    elif op == 1:
        let i = rng.int(0, 22)
        let v = rng.int(0, 63)
        xf.add(V(i), v)
        xa[i] ^= v
    elif op == 2:
        let (r, c) = (rng.int(0, 4), rng.int(0, 6))
        g.add(R(r), C(c), x)
        ga[r * 7 + c] += x
    elif op == 3:
        let p = rng.int(0, pts.len - 1)
        sf.add(pts[p].0, pts[p].1, x)
        sv[p] += x
    elif op == 4 and rng.int(0, 9) == 0:
        d.clear()
        rf.clear()
        g.clear()
        for j in 0..23: a[j] = 0
        for j in 0..ga.len: ga[j] = 0
    let i = rng.int(0, 22)
    if d.get(V(i)) != a[i]:
        bad = "op {k}: DualFenwick.get {i}"
    elif rf.fold(V.cuts.nth(lo)..V.cuts.nth(hi)) != sum(a[j] for j in lo..hi):
        bad = "op {k}: RangeFenwick.fold {lo}..{hi}"
    elif rf.prefix(V.cuts.nth(hi)) != sum(a[j] for j in 0..hi):
        bad = "op {k}: RangeFenwick.prefix {hi}"
    if bad == "":
        var want = 0
        for j in lo..hi: want ^= xa[j]
        var pre = 0
        for j in 0..hi: pre ^= xa[j]
        if xf.xor(V.cuts.nth(lo)..V.cuts.nth(hi)) != want: bad = "op {k}: XorFenwick.xor {lo}..{hi}"
        elif xf.prefix(V.cuts.nth(hi)) != pre: bad = "op {k}: XorFenwick.prefix {hi}"
    if bad == "":
        let (r0, c0) = (rng.int(0, 5), rng.int(0, 7))
        let (r1, c1) = (rng.int(r0, 5), rng.int(c0, 7))
        var want = 0
        for r in r0..r1:
            for c in c0..c1: want += ga[r * 7 + c]
        var pre = 0
        for r in 0..r1:
            for c in 0..c1: pre += ga[r * 7 + c]
        let (r, c) = (rng.int(0, 4), rng.int(0, 6))
        if g.fold(R.cuts.nth(r0)..R.cuts.nth(r1), C.cuts.nth(c0)..C.cuts.nth(c1)) != want:
            bad = "op {k}: Fenwick2D.fold {r0}..{r1} x {c0}..{c1}"
        elif g.prefix(R.cuts.nth(r1), C.cuts.nth(c1)) != pre:
            bad = "op {k}: Fenwick2D.prefix {r1} {c1}"
        elif g.get(R(r), C(c)) != ga[r * 7 + c]:
            bad = "op {k}: Fenwick2D.get {r} {c}"
    if bad == "":
        let (x0, y0) = (rng.int(-22, 22), rng.int(-22, 22))
        let (x1, y1) = (rng.int(-22, 22), rng.int(-22, 22))
        var want = 0
        for p in 0..pts.len:
            let (px, py) = pts[p]
            if x0 <= px and px < x1 and y0 <= py and py < y1: want += sv[p]
        if sf.fold(x0..x1, y0..y1) != want: bad = "op {k}: SparseFenwick2D.fold {x0}..{x1} x {y0}..{y1}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
