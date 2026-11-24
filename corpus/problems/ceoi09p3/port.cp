// Source: CEOI 2009 P3 (Photo), https://dmoj.ca/problem/ceoi09p3
// limits: 1000 ms, 512 MB
// Problem: n <= 100 points (x, y), 0 <= x, y <= 1e5. Cover them all with the fewest rectangles
//   that stand on the x-axis and have area A <= 2e6 (overlaps allowed); print that number. A point
//   on the axis (y = 0) needs no rectangle, as in the accepted C++. Bounds from the Luogu mirror P5975.
// Solution: sort by x. cover(range, k) = fewest rectangles for the points of the range above the
//   k-th compressed height. An end point at or below it is dropped; otherwise the leftmost
//   point's rectangle spans it to some point i at the tallest height area / width allows, which
//   only helps when it rises above k: 1 + cover(l..=i, that height) + cover(i+1..=r, k).
//   O(n^4): n^2 ranges, n heights, n splits.
// Bounds: n: 1..=100; area: 1..=2e6
let n: 1..=100 = read()
let area: 1..=2e6 = read()
domain P = 0..n
type Pt = (x: 0..=1e5, y: 0..=1e5)
var pts: [P]Pt = read()
pts.sort()
let (domain H, height) = compress([0] + pts.map(p => p.y).collect())

fn tallest(l: P, i: P) -> H:
    let w = pts[i].x - pts[l].x
    if w == 0: H.last! else: height.last_le(area / w)!

@memo(array)
fn cover(lo: P.cuts, hi: P.cuts, k: H) -> Int:
    let (Some(l), Some(r)) = (lo.r, hi.l) else: return 0
    if l > r: return 0
    if pts[l].y <= height[k]: return cover(l.r, hi, k)
    if pts[r].y <= height[k]: return cover(lo, r.l, k)
    [cover(lo, i.r, tallest(l, i)) + cover(i.r, hi, k) + 1 for i in l..=r if tallest(l, i) > k].min()!

print(cover(P.cuts.first!, P.cuts.last!, H.first!))
