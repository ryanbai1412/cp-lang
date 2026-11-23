// CCC 2000 S5 Sheep and Coyotes: sheep stand at points with y > 0, and a
// coyote enters anywhere on the fence, the segment y = 0, 0 <= x <= 1000,
// and eats the nearest sheep. Print every sheep that is nearest (ties count)
// for some entry point, in input order.
//
// Sheep p beats sheep q at x when 2 x (q.x - p.x) <= |q|^2 - |p|^2, one
// linear bound per rival, so the entry points that reach p are an interval.
// Coordinates have at most two decimals; they are scaled to whole numbers
// and the interval ends stay exact fractions, as in the original.
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
