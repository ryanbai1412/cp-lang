// expect: ok, output "ok"
// input: "1 300\n"
// model: HistoricBeatsSegTree
// A HistoricBeatsSegTree against a plain array, its running maxima and running sums: seeded
// random chmin, chmax and add (each records a version, empty ranges too), with sum, max_val,
// hist_max and hist_sum checked after each op. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.beats.{HistoricBeatsSegTree}
let seed0 = read<Int>()
let n = read<Int>()
domain H = 0..13
var rng = Rng(seed0)
let fill = rng.int(0, 1) == 0
var a: [H]Int = [H] _ => if fill: 4 else: rng.int(-20, 20)
var t = if fill: HistoricBeatsSegTree<H>(fill: 4) else: HistoricBeatsSegTree(a)
var hm = a.clone()
var hs = a.clone()
var bad = ""
for k in 0..n:
    let op = rng.int(0, 2)
    let lo = rng.int(0, 13)
    let hi = rng.int(lo, 13)
    let x = rng.int(-25, 25)
    let r = H.cuts.nth(lo)..H.cuts.nth(hi)
    if op == 0: t.chmin(r, x)
    elif op == 1: t.chmax(r, x)
    else: t.add(r, x)
    for j in lo..hi:
        if op == 0: a[H(j)] min= x
        elif op == 1: a[H(j)] max= x
        else: a[H(j)] += x
    for j in H:
        hm[j] max= a[j]
        hs[j] += a[j]
    let qlo = rng.int(0, 13)
    let qhi = rng.int(qlo, 13)
    let q = H.cuts.nth(qlo)..H.cuts.nth(qhi)
    let js = [H(j) for j in qlo..qhi]
    if t.sum(q) != sum(a[j] for j in js): bad = "op {k}: sum {qlo}..{qhi}"
    elif t.max_val(q) != [a[j] for j in js].max(): bad = "op {k}: max_val {qlo}..{qhi}"
    elif t.hist_max(q) != [hm[j] for j in js].max(): bad = "op {k}: hist_max {qlo}..{qhi}"
    elif t.hist_sum(q) != sum(hs[j] for j in js): bad = "op {k}: hist_sum {qlo}..{qhi}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
