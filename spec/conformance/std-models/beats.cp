// expect: ok, output "ok"
// input: "1 300\n"
// model: BeatsSegTree
// A BeatsSegTree against a plain array: seeded random chmin, chmax, add, set, assign, chdiv and
// chmod over random ranges, with sum, min_val, max_val, get and the all_ getters checked after
// each op. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.beats.{BeatsSegTree}
let seed0 = read<Int>()
let n = read<Int>()
domain B = 0..17
var rng = Rng(seed0)
fn fdiv(a: Int, d: Int) -> Int = if a >= 0: a / d else: -((-a + d - 1) / d)
var a: [B]Int = [B] _ => rng.int(-30, 30)
var t = if rng.int(0, 1) == 0: BeatsSegTree(a) else: BeatsSegTree<B>(fill: 0)
if t.all_sum == 0 and t.get(B(0)) == 0 and t.all_max == Some(0):
    for i in B: a[i] = 0
var bad = ""
for k in 0..n:
    let op = rng.int(0, 6)
    let lo = rng.int(0, 17)
    let hi = rng.int(lo, 17)
    let r = B.cuts.nth(lo)..B.cuts.nth(hi)
    let x = rng.int(-40, 40)
    let i = B(rng.int(0, 16))
    if op == 0:
        t.chmin(r, x)
        for j in lo..hi: a[B(j)] min= x
    elif op == 1:
        t.chmax(r, x)
        for j in lo..hi: a[B(j)] max= x
    elif op == 2:
        t.add(r, x)
        for j in lo..hi: a[B(j)] += x
    elif op == 3:
        if rng.int(0, 1) == 0:
            t.set(i, x)
            a[i] = x
        else:
            t.assign(r, x)
            for j in lo..hi: a[B(j)] = x
    elif op == 4:
        let d = rng.int(1, 5)
        t.chdiv(r, d)
        for j in lo..hi: a[B(j)] = fdiv(a[B(j)], d)
    elif op == 5:
        let d = rng.int(1, 12)
        t.chmod(r, d)
        for j in lo..hi: a[B(j)] = a[B(j)] - d * fdiv(a[B(j)], d)
    let qlo = rng.int(0, 17)
    let qhi = rng.int(qlo, 17)
    let q = B.cuts.nth(qlo)..B.cuts.nth(qhi)
    let vs = [a[B(j)] for j in qlo..qhi]
    let all = [a[j] for j in B]
    if t.sum(q) != sum(vs): bad = "op {k}: sum {qlo}..{qhi}"
    elif t.min_val(q) != vs.min(): bad = "op {k}: min_val {qlo}..{qhi}"
    elif t.max_val(q) != vs.max(): bad = "op {k}: max_val {qlo}..{qhi}"
    elif t.get(i) != a[i]: bad = "op {k}: get"
    elif t.all_sum != sum(all) or t.all_min != all.min() or t.all_max != all.max(): bad = "op {k}: all"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
