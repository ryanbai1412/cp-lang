// expect: ok, output "ok"
// input: "1 300\n"
// model: SegTree
// SegTree<P, Int> over + with add tags, and a point max tree built with `of`, against a plain
// array: seeded random set, update, at, apply, apply_all, refresh and clear, with fold, get,
// try_fold, walk, right_while and left_while checked after each op. Reads `seed n`.
use std.rand.{Rng}
let seed0 = read<Int>()
let n = read<Int>()
domain P = 0..21
var rng = Rng(seed0)
var a: [P]Int = [P] _ => rng.int(0, 9)
var s = SegTree<P, Int>(+, a: a, act: add)
var w: [P]Int = [P] _ => rng.int(-50, 50)
var m = SegTree<P, Int>.of(max, i => w[i])
var bad = ""
for k in 0..n:
    let op = rng.int(0, 6)
    let i = P(rng.int(0, 20))
    let x = rng.int(0, 9)
    let lo = rng.int(0, 21)
    let hi = rng.int(lo, 21)
    let r = P.cuts.nth(lo)..P.cuts.nth(hi)
    if op == 0:
        s.set(i, x)
        a[i] = x
    elif op == 1:
        s.update(i, v => v * 2 + x)
        a[i] = a[i] * 2 + x
    elif op == 2:
        s.at(i) += x
        a[i] += x
    elif op == 3:
        s.apply(r, x)
        for j in lo..hi: a[P(j)] += x
    elif op == 4:
        s.apply_all(x)
        for j in P: a[j] += x
    elif op == 5:
        w[i] = x * 10 - 45
        m.refresh(i)
    elif rng.int(0, 9) == 0:
        s.clear(fill: Some(x))
        for j in P: a[j] = x
    let want = sum(a[P(j)] for j in lo..hi)
    let lim = rng.int(0, 60)
    let from = rng.int(0, 21)
    var d = from
    var acc = 0
    while d < 21 and acc + a[P(d)] <= lim:
        acc += a[P(d)]
        d += 1
    var e = from
    acc = 0
    while e > 0 and acc + a[P(e - 1)] <= lim:
        acc += a[P(e - 1)]
        e -= 1
    let want_max = if lo == hi: None else: max(w[P(j)] for j in lo..hi)
    if s.fold(r) != want: bad = "op {k}: fold {lo}..{hi}"
    elif s.get(i) != a[i]: bad = "op {k}: get"
    elif s.all != sum(a[j] for j in P): bad = "op {k}: all"
    elif s.try_fold(r) != (if lo == hi: None else: Some(want)): bad = "op {k}: try_fold"
    elif sum(s.walk(r)) != want: bad = "op {k}: walk {lo}..{hi}"
    elif s.right_while(P.cuts.nth(from), t => t <= lim).ord != d: bad = "op {k}: right_while {from}"
    elif s.left_while(P.cuts.nth(from), t => t <= lim).ord != e: bad = "op {k}: left_while {from}"
    elif m.try_fold(r) != want_max: bad = "op {k}: max fold {lo}..{hi}"
    elif m.get(i) != w[i]: bad = "op {k}: max get"
    if bad != "":
        break
print(if bad == "": "ok" else: bad)
