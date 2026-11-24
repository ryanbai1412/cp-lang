// expect: ok, output "ok"
// input: "1 300\n"
// model: RangeKth
// A RangeKth over seeded random values against sorted slices: n random ranges with kth,
// count_le, count_lt and count_in checked. Reads `seed n`; prints the first query that differs.
use std.rand.{Rng}
use std.persistent.{RangeKth}
let seed0 = read<Int>()
let n = read<Int>()
domain K = 0..25
var rng = Rng(seed0)
let a: [K]Int = [K] _ => rng.int(-8, 8)
let t = RangeKth(a)
var bad = ""
for k in 0..n:
    let lo = rng.int(0, 25)
    let hi = rng.int(lo, 25)
    let r = K.cuts.nth(lo)..K.cuts.nth(hi)
    var ys = [a[K(j)] for j in lo..hi]
    ys.sort()
    let x = rng.int(-10, 10)
    let y = rng.int(-10, 10)
    if hi > lo:
        let j = rng.int(0, hi - lo - 1)
        if t.kth(r, j) != ys[j]: bad = "op {k}: kth {lo}..{hi} {j}"
    if t.count_le(r, x) != ys.count(v => v <= x): bad = "op {k}: count_le {x}"
    elif t.count_lt(r, x) != ys.count(v => v < x): bad = "op {k}: count_lt {x}"
    elif t.count_in(r, x, y) != ys.count(v => x <= v and v <= y): bad = "op {k}: count_in {x} {y}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
