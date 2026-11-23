// Source: CEOI 2009 P3 (Photo), https://dmoj.ca/problem/ceoi09p3
// Problem: n <= 100 points (x, y), 0 <= x, y <= 1e5. Cover them all with the fewest rectangles
//   that stand on the x-axis and have area A <= 2e6 (overlaps allowed); print that number. A point
//   on the axis (y = 0) needs no rectangle, as in the accepted C++. Bounds from the Luogu mirror P5975.
// Solution: sort by x. dp[range, k] = fewest rectangles for the points of the range above the
//   k-th compressed height. An end point at or below it is dropped; otherwise the leftmost
//   point's rectangle spans it to some point i at the tallest height area / width allows:
//   1 + dp[l..=i, that height] + dp[i+1..=r, k]. O(n^4): n^2 ranges, n heights, n splits.
let n: 1..=100 = read()
let area: 1..=2e6 = read()
domain P = 0..n
type Pt = (x: 0..=1e5, y: 0..=1e5)
let raw: [P]Pt = read()
var pts = raw.copy()
pts.sort()

var ys: List<Int> = [0]
for p in pts:
    ys.push(p.y)
let (H, height) = compress(ys)  // H.first is height 0

// tallest allowed height index for a rectangle spanning pts[l]..pts[i]
fn tallest(l: P, i: P) -> H:
    let w = pts[i].x - pts[l].x
    if w == 0:
        return H.last!
    height.last_le(area / w)!

const INF = 1000000000
// dp[lo, hi, k]: rectangles to cover the points in lo..hi lying above height[k]
var dp: [P.cuts, P.cuts, H]Int = [P.cuts, P.cuts, H] (a, b, _) => if a == b: 0 else: INF
for l in P.rev:
    for r in l..:
        let lo = l.l
        let hi = r.r
        for k in H.rev:
            if pts[l].y <= height[k]:
                dp[lo, hi, k] = dp[l.r, hi, k]
            elif pts[r].y <= height[k]:
                dp[lo, hi, k] = dp[lo, r.l, k]
            else:
                dp[lo, hi, k] = [
                    dp[lo, i.r, tallest(l, i)] + dp[i.r, hi, k] + 1 for i in l..=r
                ].min()!
print(dp[P.cuts.first!, P.cuts.last!, H.first!])
