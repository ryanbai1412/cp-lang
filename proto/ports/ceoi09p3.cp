// cp: 0.1 8ef64b9+dirty
let n: 1..=100 = read()
let area: 1..=3e5 = read()
domain P = 0..n
type Pt = (x: 0..=3e5, y: 1..=3e5)
let raw: [P]Pt = read()
let pts = sorted(raw)

var ys: List<Int> = [0]
for p in pts:
    ys.push(p.y)
let (H, height) = compress(ys)              // H.first is height 0

// tallest allowed height index for a rectangle spanning pts[l]..pts[i]
fn tallest(l: P, i: P) -> H:
    let w = pts[i].x - pts[l].x
    if w == 0:
        return H.last
    height.last_le(area / w)!

const INF = 1000000000
// dp[lo, hi, k]: rectangles to cover the points in lo..hi lying above height[k]
var dp: [P.bounds, P.bounds, H]Int = [P.bounds, P.bounds, H] (a, b, _) => if a == b: 0 else: INF
for l in P.rev:
    for r in l..=P.last:
        let lo = l.before
        let hi = r.after
        for k in H.rev:
            if pts[l].y <= height[k]:
                dp[lo, hi, k] = dp[l.after, hi, k]
            elif pts[r].y <= height[k]:
                dp[lo, hi, k] = dp[lo, r.before, k]
            else:
                dp[lo, hi, k] = min(dp[lo, i.after, tallest(l, i)] + dp[i.after, hi, k] + 1 for i in l..=r)!
print(dp[P.bounds.first, P.bounds.last, H.first])
