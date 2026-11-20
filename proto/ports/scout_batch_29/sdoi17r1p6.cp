// SDOI 2017 Round 1 P6, Correlation analysis: arrays x and y of length n
// and three kinds of operation on positions l..=r:
//   1 l r      print the least-squares slope of y on x,
//              sum (x - mx)(y - my) / sum (x - mx)^2 with mx, my the means
//   2 l r S T  add S to every x and T to every y
//   3 l r S T  set x[i] = S + i and y[i] = T + i
// The slope is printed to 5 decimals (the original first cuts it to 6).
//
// Times the length m, the slope is (m sum xy - sum x sum y) /
// (m sum x^2 - (sum x)^2), so a node keeps the four sums; it also keeps
// where it starts, because "set" writes a different value at each i.
// original: corpus/submissions/5461671.cpp
use std.lazy.{LazySegTree}
use std.algebra.{Monoid, Action}

type Sums = (xy: Int, x: Int, y: Int, xx: Int, lo: Int, len: Int)
type Edit = (set: ?(x: Int, y: Int), dx: Int, dy: Int)       // set first, then add

fn join(a: Sums, b: Sums) -> Sums:
    if a.len == 0: return b
    if b.len == 0: return a
    (xy: a.xy + b.xy, x: a.x + b.x, y: a.y + b.y, xx: a.xx + b.xx, lo: a.lo, len: a.len + b.len)

/// 1 + ... + k and 1^2 + ... + k^2.
fn tri(k: Int) -> Int = k * (k + 1) / 2
fn pyr(k: Int) -> Int = k * (k + 1) * (2 * k + 1) / 6

fn edit(f: Edit, t: Sums, len: Int) -> Sums:
    if t.len == 0: return t
    var s = t
    if let v = f.set:
        // x[i] = v.x + i and y[i] = v.y + i for i in lo..lo + len
        let hi = t.lo + t.len - 1
        let i1 = tri(hi) - tri(t.lo - 1)
        let i2 = pyr(hi) - pyr(t.lo - 1)
        s = (xy: t.len * v.x * v.y + (v.x + v.y) * i1 + i2,
             x: t.len * v.x + i1,
             y: t.len * v.y + i1,
             xx: t.len * v.x * v.x + 2 * v.x * i1 + i2,
             lo: t.lo, len: t.len)
    (xy: s.xy + f.dx * s.y + f.dy * s.x + s.len * f.dx * f.dy,
     x: s.x + s.len * f.dx,
     y: s.y + s.len * f.dy,
     xx: s.xx + 2 * f.dx * s.x + s.len * f.dx * f.dx,
     lo: s.lo, len: s.len)

/// `f` after `g`.
fn then(f: Edit, g: Edit) -> Edit = if f.set != none: f else: (set: g.set, dx: g.dx + f.dx, dy: g.dy + f.dy)

/// `num / den` rounded to 5 decimals, after cutting it to 6 towards zero.
fn slope(num: Int, den: Int) -> Str:
    let cut = abs(num) * 1_000_000 / den
    let units = (cut + 5) / 10
    var digits = "{units % 100_000}"
    while digits.len < 5: digits = "0" + digits
    (if num < 0 and units > 0: "-" else: "") + "{units / 100_000}." + digits

let (n, q): (Int, Int) = read()
domain Pos = 1..=n
let x: [Pos]Int = read()
let y: [Pos]Int = read()
let empty: Sums = (xy: 0, x: 0, y: 0, xx: 0, lo: 0, len: 0)
let keep: Edit = (set: none, dx: 0, dy: 0)
var tree = LazySegTree<Pos, Sums, Edit>(Action<Sums, Edit>(keep, then, edit, over: Monoid<Sums>(empty, join)),
    [Pos] i => (xy: x[i] * y[i], x: x[i], y: y[i], xx: x[i] * x[i], lo: i.label, len: 1))

for _ in 0..q:
    let (kind, l, r): (Int, Pos, Pos) = read()
    if kind == 1:
        let s = tree.fold(l..=r)
        print(slope(s.xy * s.len - s.x * s.y, s.len * s.xx - s.x * s.x))
    else:
        let (a, b): (Int, Int) = read()
        tree.apply(l..=r, if kind == 2: (set: none, dx: a, dy: b) else: (set: (x: a, y: b), dx: 0, dy: 0))
