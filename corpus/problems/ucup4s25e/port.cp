// UCup 4 Stage 25 (Zagreb) E: for each query v, maximize sum(a[i..j]) + (j - i) * v
// over all non-empty subarrays a[i..j] (0 <= i < j <= n).
//
// Represent each prefix as Pt(i, sum(a[..i])). For a split at md:
// - negating left prefixes in reverse order `(lo..md).rev` gives
//   `-pref[i] = Pt(-i, -sum(a[..i]))` with strictly increasing x = -i;
// - right prefixes `pref[j]` (`md..hi`) have strictly increasing x = j;
// - every Minkowski sum point `(-pref[i]) + pref[j]` is `Pt(j - i, sum(a[i..j]))`,
//   so `minkowski_upper` of the two upper hulls yields all cross-boundary hull
//   maxima in O(hi - lo) time.
// Taking the upper hull of the per-length maxima and maximizing `dot(p, Pt(v, 1))`
// answers each query in O(log n).
use std.geometry.{hull_extreme, minkowski_upper, upper_hull}

@!multitest
let (n, q): (1..=500_000, 1..=500_000) = read()
let a: [0..n]Int = read()

var pref: [0..=n]Pt = Pt(0, 0)
for i in 0..n: pref[i + 1] = Pt(i + 1, pref[i].y + a[i])

fn solve_dc(lo: Int, hi: Int) -> List<?Int>:
    let m = hi - lo - 1
    if m <= 0: return []
    let md = (lo + hi) / 2
    var res: List<?Int> = List.filled(m, none)
    for (i, v) in solve_dc(lo, md).items:
        if let y = v: res[i] max= y
    for (i, v) in solve_dc(md, hi).items:
        if let y = v: res[i] max= y
    let los = upper_hull([-pref[i] for i in (lo..md).rev])
    let his = upper_hull([pref[j] for j in md..hi])
    for p in minkowski_upper(los, his): res[p.x - 1] max= p.y
    res

let best = solve_dc(0, n + 1)
// `upper_hull` returns left-to-right (CW); `hull_extreme` requires CCW order.
var hull = upper_hull([Pt(i + 1, y) for (i, v) in best.items if let y = v])
hull.reverse()

fn answer(v: Int) -> Int:
    let dir = Pt(v, 1)
    hull[hull_extreme(hull, dir)].dot(dir)

let qs: [0..q]Int = read()
print([answer(v) for v in qs])
