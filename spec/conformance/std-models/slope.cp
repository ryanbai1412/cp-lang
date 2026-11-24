// expect: ok, output "ok"
// input: "1 300\n"
// model: Concave
// model: Convex
use std.rand.{Rng}
use std.slope.{Concave, Convex}

// A function on `lo..lo + v.len` as its plain values; `g` mirrors `f` negated.
fn argbest(v: List<Int>, lo: Int) -> Range<Int>:
    let m = v.max()!
    var a = 0
    while v[a] != m: a += 1
    var b = a
    while b < v.len and v[b] == m: b += 1
    lo + a..lo + b

// A small concave `h` with its values, and the convex `-h` built by the mirrored calls.
fn draw_h(mut rng: Rng) -> (Concave, Convex, Int, List<Int>):
    let hlo = rng.int(-2, 2)
    let w = rng.int(0, 2)
    var h = Concave.flat(hlo..=hlo + w)
    var hc = Convex.flat(hlo..=hlo + w)
    var hv = List.filled(w + 1, 0)
    let a = rng.int(hlo - 1, hlo + w + 1)
    if rng.chance(0.5):
        h.add_abs(a)
        hc.add_abs(a)
        for i in 0..hv.len: hv[i] -= abs(hlo + i - a)
    let k = rng.int(-3, 3)
    h.add_linear(k)
    hc.add_linear(-k)
    for i in 0..hv.len: hv[i] += k * (hlo + i)
    (h, hc, hlo, hv)

let seed = read<Int>()
let n = read<Int>()
var rng = Rng(seed)
var lo = rng.int(-5, 5)
let w0 = rng.int(0, 6)
var f = if w0 == 0: Concave.point(lo) else: Concave.flat(lo..=lo + w0)
var g = if w0 == 0: Convex.point(lo) else: Convex.flat(lo..lo + w0 + 1)
f.reserve(64)
var v = List.filled(w0 + 1, 0)
var bad = ""
for k in 0..n:
    if bad != "": break
    let op = rng.int(0, 11)
    let a = rng.int(lo - 2, lo + v.len + 1)
    if op == 0:
        f.add_left(a)
        g.add_left(a)
        for i in 0..v.len: v[i] += min(0, lo + i - a)
    elif op == 1:
        f.add_right(a)
        g.add_right(a)
        for i in 0..v.len: v[i] += min(0, a - lo - i)
    elif op == 2:
        f.add_abs(a)
        g.add_abs(a)
        for i in 0..v.len: v[i] -= abs(lo + i - a)
    elif op == 3:
        let c = rng.int(-9, 9)
        f.add_const(c)
        g.add_const(-c)
        for i in 0..v.len: v[i] += c
    elif op == 4:
        let s = rng.int(-3, 3)
        f.add_linear(s)
        g.add_linear(-s)
        for i in 0..v.len: v[i] += s * (lo + i)
    elif op == 5:
        let p = rng.int(0, 2)
        let b = rng.int(-4, 4)
        let c = rng.int(-4, 4)
        f.add_quadratic(x => -p * x * x + b * x + c)
        g.add_quadratic(x => p * x * x - b * x - c)
        for i in 0..v.len: v[i] += -p * (lo + i) * (lo + i) + b * (lo + i) + c
    elif op == 6:
        f.prefix_max()
        g.prefix_min()
        for i in 1..v.len: v[i] = max(v[i], v[i - 1])
    elif op == 7:
        f.suffix_max()
        g.suffix_min()
        for i in (0..v.len - 1).rev: v[i] = max(v[i], v[i + 1])
    elif op == 8:
        let d = rng.int(-3, 3)
        f.shift(d)
        g.shift(d)
        lo += d
    elif op == 9 and v.len < 24:
        let x = rng.int(-2, 1)
        let y = x + rng.int(0, 2)
        f.slide_max(x, y)
        g.slide_min(x, y)
        var nv = List.filled(v.len + y - x, 0)
        for c in 0..nv.len:
            nv[c] = (0..v.len).filter(j => c - (y - x) <= j and j <= c).map(j => v[j]).max()!
        v = nv
        lo += x
    elif op == 10 and v.len < 24:
        let (h, hc, hlo, hv) = draw_h(mut rng)
        f.max_convolve(h)
        g.min_convolve(hc)
        var nv = List.filled(v.len + hv.len - 1, -1000000000)
        for i in 0..v.len:
            for j in 0..hv.len: nv[i + j] = max(nv[i + j], v[i] + hv[j])
        v = nv
        lo += hlo
    if f.lo != lo or f.hi != lo + v.len - 1 or g.lo != lo or g.hi != f.hi:
        bad = "op {k}: domain {f.lo}..={f.hi} want {lo}..={lo + v.len - 1}"
    for i in 0..v.len:
        if bad == "" and f.at(lo + i) != v[i]: bad = "op {k}: concave at({lo + i}) = {f.at(lo + i)} want {v[i]}"
        if bad == "" and g.at(lo + i) != -v[i]: bad = "op {k}: convex at({lo + i}) = {g.at(lo + i)} want {-v[i]}"
    if bad == "" and f.max() != v.max()!: bad = "op {k}: max {f.max()} want {v.max()!}"
    elif bad == "" and g.min() != -v.max()!: bad = "op {k}: min {g.min()} want {-v.max()!}"
    elif bad == "" and f.argmax() != argbest(v, lo): bad = "op {k}: argmax {f.argmax()} want {argbest(v, lo)}"
    elif bad == "" and g.argmin() != argbest(v, lo): bad = "op {k}: argmin {g.argmin()} want {argbest(v, lo)}"
print(if bad == "": "ok" else: bad)
