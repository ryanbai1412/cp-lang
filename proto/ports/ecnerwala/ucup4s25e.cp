// UCup 4 Stage 25 (Zagreb) E: divide-and-conquer Minkowski sum over prefix-sum
// upper convex hulls to find maximum subarray sum per length, then binary
// search the resulting upper hull for each linear query.
use std.geometry.{dot, hull_extreme, minkowski_upper, upper_hull}

@!multitest
let (n, q): (Int, Int) = read()
let a: [0..n]Int = read()

var pref: [0..=n]Pt = Pt(0, 0)
for i in 0..n: pref[i + 1] = Pt(i + 1, pref[i].y + a[i])
// Best subarray sum of each length (1..=n), seeded with prefixes starting at 0.
var best: [0..n]Pt = [0..n] i => pref[i + 1]

fn solve_dc(lo: Int, hi: Int):
    if hi - lo <= 1: return
    let md = (lo + hi) / 2
    solve_dc(lo, md)
    solve_dc(md, hi)
    let los = upper_hull([-pref[i] for i in (lo..md).rev])
    let his = upper_hull([pref[i] for i in md..hi])
    for p in minkowski_upper(los, his): best[p.x - 1].y max= p.y

solve_dc(0, n + 1)
var hull = upper_hull([best[i] for i in 0..n])
hull.reverse()

let qs: [0..q]Int = read()
print([dot(hull[hull_extreme(hull, Pt(v, 1))], Pt(v, 1)) for v in qs])
