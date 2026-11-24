// expect: ok, output "ok"
// input: "1 300\n"
// model: Prefix
// model: FillPrefix
// model: scan
// model: prefix_fold
// model: Diff
// model: ArithDiff
// model: SweepAdd
// Prefix tables (built, scanned forward and back, filled either way), prefix_fold, and the
// Diff, ArithDiff and SweepAdd range-add arrays over a small domain, each against plain loops
// over a List, after a seeded random point edit or range add per op.
// Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.seq.{span}
use std.prefix.{ArithDiff, SweepAdd, prefix_fold}
domain D = 0..12
let m = 12

fn sum(xs: List<Int>, l: Int, r: Int) -> Int:
    var s = 0
    for j in l..r: s += xs[j]
    s

fn cut(j: Int) -> Cut<D> = D.cuts.nth(j)

fn rg(l: Int, h: Int) -> Range<D> = span(cut(l), cut(h))

let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
var xs = List.filled(m, 0)
var diff = Diff<D, Int>()
var dv = List.filled(m, 0)
var ad = ArithDiff<D, Int>()
var av = List.filled(m, 0)
var bad = ""
for k in 0..n:
    let neg = rng.int(0, 3) == 0
    xs[rng.int(0, m - 1)] = rng.int(if neg: -9 else: 0, 9)
    let lo = rng.int(0, m)
    let hi = rng.int(lo, m)
    let r = rg(lo, hi)
    let x = rng.int(-9, 9)
    let step = rng.int(-3, 3)
    diff.add(r, x)
    for j in lo..hi: dv[j] += x
    if rng.int(0, 1) == 0:
        ad.add_arith(r, x, step)
        for j in lo..hi: av[j] += x + step * (j - lo)
    else:
        ad.add(r, x)
        for j in lo..hi: av[j] += x
    let arr: [D]Int = [D] v => xs[Int(v)]
    let total = sum(xs, 0, m)
    let p = if x % 2 == 0: Prefix(arr) else: Prefix<D, Int>.new(arr)
    let sc = arr.scan(+)
    let sf = arr.scan(+, from: 10)
    let mx = arr.scan(max)
    let back = arr.rev.scan(+)
    let rs = Prefix<D, Int>.of_rev_scan(arr, +, None)
    let pf = prefix_fold(+, arr)
    let pm = prefix_fold(max, arr)
    var best = Int.MIN
    for j in 0..=m:
        let c = cut(j)
        let want = sum(xs, 0, j)
        if p.prefix(c) != want or p.at(c) != want or p.cut(j) != want or sc.prefix(c) != want or sc.cut(j) != want:
            bad = "op {k}: prefix at {j}"
        elif p.suffix(c) != total - want or sc.suffix(c) != total - want or back.suffix(c) != total - want:
            bad = "op {k}: suffix at {j}"
        elif sf.prefix(c) != 10 + want or pf[c] != want or back.at(c) != total - want or rs.at(c) != total - want:
            bad = "op {k}: scan from/prefix_fold/rev scan at {j}"
        elif mx.cut(j) != (if j == 0: Int.MIN else: best) or pm[c] != mx.cut(j):
            bad = "op {k}: scan(max) at {j}"
        if j < m: best = max(best, xs[j])
        if bad != "": break
    if bad == "" and (p.all != total or sc.all != total or back.all != total or rs.all != total):
        bad = "op {k}: all"
    if bad == "" and (p.fold(r) != sum(xs, lo, hi) or sc.fold(r) != sum(xs, lo, hi) or back.fold(r) != sum(xs, lo, hi)):
        bad = "op {k}: fold {lo}..{hi}"
    let s = rng.int(0, m - 1)
    let len = rng.int(0, m)
    var want_ring = 0
    for j in 0..len: want_ring += xs[(s + j) % m]
    if bad == "" and p.cyclic_sum(arr.ring.span(D(s), len)) != want_ring: bad = "op {k}: cyclic_sum {s} {len}"
    if bad == "" and not any(y < 0 for y in xs):
        let t = rng.int(-2, total + 2)
        var ge = 0
        while ge <= m and sum(xs, 0, ge) < t: ge += 1
        var gt = 0
        while gt <= m and sum(xs, 0, gt) <= t: gt += 1
        let got_ge = if let c = p.first_ge(t): Int(c) else: m + 1
        let got_gt = if let c = sc.first_gt(t): Int(c) else: m + 1
        if got_ge != ge or got_gt != gt: bad = "op {k}: first_ge/gt {t}: {got_ge} {got_gt} want {ge} {gt}"
    if bad == "":
        let t = rng.int(0, m)
        var f = Prefix<D, Int>.filling()
        for j in 0..t: f.set(D(j), xs[j])
        if f.rev or f.next != t: bad = "op {k}: forward fill state"
        for j in 0..=t:
            if bad == "" and (f.prefix(cut(j)) != sum(xs, 0, j) or f.at(cut(j)) != sum(xs, 0, j) or f.cut(j) != sum(xs, 0, j)):
                bad = "op {k}: forward fill prefix {j} of {t}"
        let fl = rng.int(0, t)
        let fh = rng.int(fl, t)
        if bad == "" and f.fold(rg(fl, fh)) != sum(xs, fl, fh): bad = "op {k}: forward fill fold"
        var g = Prefix<D, Int>.filling(order: FillOrder.Reverse)
        for j in (t..m).rev: g.set(D(j), xs[j])
        if not g.rev or g.next != t - 1: bad = "op {k}: reverse fill state"
        let gl = rng.int(t, m)
        let gh = rng.int(gl, m)
        if bad == "" and g.fold(rg(gl, gh)) != sum(xs, gl, gh): bad = "op {k}: reverse fill fold"
        if bad == "" and t == 0:
            for j in 0..=m:
                if g.prefix(cut(j)) != sum(xs, 0, j): bad = "op {k}: reverse fill prefix {j}"
    if bad == "":
        let dfin = diff.finish()
        let afin = ad.finish()
        for j in 0..m:
            if dfin[D(j)] != dv[j]: bad = "op {k}: Diff at {j}"
            if afin[D(j)] != av[j]: bad = "op {k}: ArithDiff at {j}"
    if bad == "":
        let rev = x % 2 != 0
        var sw = if rev: SweepAdd<D, Int>(order: FillOrder.Reverse) else: SweepAdd<D, Int>()
        var sv = List.filled(m, 0)
        for t in 0..m:
            let i = if rev: m - 1 - t else: t
            for _ in 0..rng.int(0, 2):
                let a = rng.int(0, m)
                let b = rng.int(a, m)
                let ok = if rev: b <= i + 1 else: a >= i
                if ok:
                    let y = rng.int(-9, 9)
                    sw.add(rg(a, b), y)
                    for j in a..b: sv[j] += y
            if sw.advance(D(i)) != sv[i] or sw.current != sv[i]: bad = "op {k}: SweepAdd at {i}"
    if bad != "": break
print(if bad == "": "ok" else: bad)
