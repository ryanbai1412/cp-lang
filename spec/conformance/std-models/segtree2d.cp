// expect: ok, output "ok"
// input: "1 300\n"
// model: SegTree2D
// SegTree2D<R, C, Int> under + against a plain grid: seeded random set and update, each followed
// by a random rectangle fold, a point get and the total. Reads `seed n`; prints the first op
// that differs.
use std.rand.{Rng}
use std.segtree.{SegTree2D}
let seed0 = read<Int>()
let n = read<Int>()
domain R = 0..5
domain C = 0..7
var rng = Rng(seed0)
var a: [R, C]Int = 0
for r in R:
    for c in C: a[r, c] = rng.int(-9, 9)
var g = SegTree2D<R, C, Int>(+, a: a)
var bad = ""
for k in 0..n:
    let (r, c, x) = (R(rng.int(0, 4)), C(rng.int(0, 6)), rng.int(-9, 9))
    if rng.int(0, 1) == 0:
        g.set(r, c, x)
        a[r, c] = x
    else:
        g.update(r, c, v => v - x)
        a[r, c] -= x
    let r0 = rng.int(0, 5)
    let r1 = rng.int(r0, 5)
    let c0 = rng.int(0, 7)
    let c1 = rng.int(c0, 7)
    var want = 0
    for i in r0..r1:
        for j in c0..c1: want += a[R(i), C(j)]
    var total = 0
    for i in R:
        for j in C: total += a[i, j]
    if g.fold(R.cuts.nth(r0)..R.cuts.nth(r1), C.cuts.nth(c0)..C.cuts.nth(c1)) != want:
        bad = "op {k}: fold {r0}..{r1} x {c0}..{c1}"
    elif g.get(r, c) != a[r, c]:
        bad = "op {k}: get"
    elif g.all != total:
        bad = "op {k}: all"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
