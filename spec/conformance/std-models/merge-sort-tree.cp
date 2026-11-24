// expect: ok, output "ok"
// input: "1 300\n"
// model: MergeSortTree
// MergeSortTree over a seeded random array against a scan of the range: each op draws a range
// and two values and checks every count and first_ge, last_le. Reads `seed n`; prints the first
// op that differs.
use std.rand.{Rng}
use std.segtree.{MergeSortTree}
let seed0 = read<Int>()
let n = read<Int>()
domain Q = 0..30
var rng = Rng(seed0)
let a: [Q]Int = [Q] _ => rng.int(-5, 20)
let t = MergeSortTree(a)
var bad = ""
for k in 0..n:
    let lo = rng.int(0, 30)
    let hi = rng.int(lo, 30)
    let x = rng.int(-7, 22)
    let y = rng.int(-7, 22)
    var ys = [a[Q(i)] for i in lo..hi]
    ys.sort()
    let r = Q.cuts.nth(lo)..Q.cuts.nth(hi)
    let ge = [u for u in ys if u >= x]
    let le = [u for u in ys if u <= x]
    if t.count_lt(r, x) != ys.count(u => u < x): bad = "op {k}: count_lt"
    elif t.count_le(r, x) != ys.count(u => u <= x): bad = "op {k}: count_le"
    elif t.count_gt(r, x) != ys.count(u => u > x): bad = "op {k}: count_gt"
    elif t.count_ge(r, x) != ys.count(u => u >= x): bad = "op {k}: count_ge"
    elif t.count_in(r, x, y) != ys.count(u => x <= u and u <= y): bad = "op {k}: count_in"
    elif t.first_ge(r, x) != ge.first: bad = "op {k}: first_ge {x}"
    elif t.last_le(r, x) != le.last: bad = "op {k}: last_le {x}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
