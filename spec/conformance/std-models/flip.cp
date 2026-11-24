// expect: ok, output "ok"
// input: "1 300\n"
// model: Flip
// model: flip_action
// SegTree<P, Flip<Int>, Bool> over flip_action(+) and Flip.op(max), with leaves from Flip.of,
// against plain arrays of values and on/off bits: seeded random range toggles and point sets,
// with the on-sum, off-sum and on-max of random ranges checked after each op. Reads `seed n`.
use std.rand.{Rng}
use std.lazy.{Flip, flip_action}
let seed0 = read<Int>()
let n = read<Int>()
domain P = 0..15
var rng = Rng(seed0)
var on: [P]Bool = [P] _ => rng.int(0, 1) == 1
var x: [P]Int = [P] _ => rng.int(0, 20)
let act = flip_action<Int>(+)
var s = SegTree<P, Flip<Int>, Bool>(act.over, [P] i => Flip.of(on[i], x[i], 0), act: act)
let mx = flip_action<Int>(max)
var t = SegTree<P, Flip<Int>, Bool>(Flip<Int>.op(max), [P] i => Flip.of(on[i], x[i], Int.MIN), act: mx)
var bad = ""
for k in 0..n:
    let op = rng.int(0, 2)
    let lo = rng.int(0, 15)
    let hi = rng.int(lo, 15)
    let r = P.cuts.nth(lo)..P.cuts.nth(hi)
    if op <= 1:
        let f = rng.int(0, 3) > 0
        s.apply(r, f)
        t.apply(r, f)
        if f:
            for j in lo..hi: on[P(j)] = not on[P(j)]
    else:
        let i = P(rng.int(0, 14))
        on[i] = rng.int(0, 1) == 1
        x[i] = rng.int(0, 20)
        s.set(i, Flip.of(on[i], x[i], 0))
        t.set(i, Flip.of(on[i], x[i], Int.MIN))
    let qlo = rng.int(0, 15)
    let qhi = rng.int(qlo, 15)
    let q = P.cuts.nth(qlo)..P.cuts.nth(qhi)
    let js = [P(j) for j in qlo..qhi]
    let got = s.fold(q)
    let m = t.fold(q)
    if got.now != sum(x[j] for j in js if on[j]): bad = "op {k}: on sum {qlo}..{qhi}"
    elif got.flipped != sum(x[j] for j in js if not on[j]): bad = "op {k}: off sum {qlo}..{qhi}"
    elif m.now != ([x[j] for j in js if on[j]].max() ?? Int.MIN): bad = "op {k}: on max {qlo}..{qhi}"
    elif m.flipped != ([x[j] for j in js if not on[j]].max() ?? Int.MIN): bad = "op {k}: off max {qlo}..{qhi}"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
