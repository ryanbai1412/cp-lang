// expect: ok, output "ok"
// input: "1 300\n"
// model: WaveletMatrix
// WaveletMatrix over a seeded random array against a sorted slice: each op draws a range, a
// value and a rank and checks kth, rank, count_lt, count_in, prev_val, next_val and
// quantile_sum. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.segtree.{WaveletMatrix}
let seed0 = read<Int>()
let n = read<Int>()
domain Q = 0..30
var rng = Rng(seed0)
let a: [Q]Int = [Q] _ => rng.int(0, 20)
let v = WaveletMatrix(a)
var bad = ""
for k in 0..n:
    let lo = rng.int(0, 30)
    let hi = rng.int(lo, 30)
    let x = rng.int(-2, 22)
    let y = rng.int(-2, 22)
    let j = rng.int(0, hi - lo)
    var ys = [a[Q(i)] for i in lo..hi]
    ys.sort()
    let r = Q.cuts.nth(lo)..Q.cuts.nth(hi)
    let lt = ys.count(t => t < x)
    let below = [t for t in ys if t < x]
    let above = [t for t in ys if t > x]
    if v.kth(r, j) != (if j < ys.len: Some(ys[j]) else: None): bad = "op {k}: kth {lo}..{hi} {j}"
    elif v.rank(r, x) != ys.count(t => t == x): bad = "op {k}: rank"
    elif v.count_lt(r, x) != lt: bad = "op {k}: count_lt"
    elif v.count_in(r, x, y) != ys.count(t => x <= t and t <= y): bad = "op {k}: count_in {x} {y}"
    elif v.prev_val(r, x) != below.last: bad = "op {k}: prev_val {x}"
    elif v.next_val(r, x) != above.first: bad = "op {k}: next_val {x}"
    elif v.quantile_sum(r, j) != sum(ys[i] for i in 0..j): bad = "op {k}: quantile_sum {j}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
