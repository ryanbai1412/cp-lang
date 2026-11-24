// Source: CCC 2000 S5 (Sheep and Coyotes), https://dmoj.ca/problem/ccc00s5
// limits: 1000 ms, 256 MB
// Problem: up to 1000 sheep in a 1000 x 1000 field, coordinates with two decimals. A coyote
//   enters anywhere on the south side (y = 0, 0 <= x <= 1000) and eats the nearest sheep (any of
//   tied ones). Print "The sheep at (x, y) might be eaten." for each sheep that can be eaten.
// Solution: p beats q at entry x iff 2x(q.x - p.x) <= |q|^2 - |p|^2, one half-line per rival, so
//   p's entry points are an interval; a rival with the same x and a lower y wins everywhere.
//   Coordinates are scaled to integers and interval ends kept as exact fractions. O(n^2).
// original: corpus/submissions/4584542.cpp
const SCALE = 1000

struct Frac(num: Int, den: Int)  // den > 0
fn le(a: Frac, b: Frac) -> Bool = a.num * b.den <= b.num * a.den
fn frac(num: Int, den: Int) -> Frac = if den < 0: Frac(-num, -den) else: Frac(num, den)

// 12340 -> "12.34": the scaled coordinate back in the two decimals the judge wants.
fn shown(v: Int) -> Str:
    let cents = v % SCALE / 10
    "{v / SCALE}.{cents / 10}{cents % 10}"

let n: Int = read()
var sheep: List<(Int, Int)> = []
for _ in 0..n:
    let x: F64 = read()
    let y: F64 = read()
    sheep.push((round(x * SCALE), round(y * SCALE)))

for (px, py) in sheep:
    var (lo, hi) = (Frac(0, 1), Frac(1000 * SCALE, 1))
    var hidden = false  // a rival straight below p wins everywhere
    for (qx, qy) in sheep:
        if qx == px:
            if qy < py: hidden = true
            continue
        let cut = frac(qx * qx + qy * qy - px * px - py * py, 2 * (qx - px))
        if qx > px:
            if le(cut, hi): hi = cut
        elif le(lo, cut):
            lo = cut
    if not hidden and le(lo, hi):
        print("The sheep at ({shown(px)}, {shown(py)}) might be eaten.")
