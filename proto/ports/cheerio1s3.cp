// cp: 0.1 1f59ac0+dirty
// Cheerio Contest 1 S3: points with increasing x. Count the pairs i < j for
// which the slope from i to j is the steepest or the shallowest among the
// slopes from i to every point up to j.
let n: 1..=2000 = read()
domain P = 0..n
type Point = (x: Int, y: Int)
fn left_to_right() -> [P]Point: [P] i => (3 * i.idx + rand(0, 2), rand(-6, 6))
let pts: [P]Point = read(gen: left_to_right) where all(pts[P(k)].x < pts[P(k + 1)].x for k in 0..n - 1)

type Slope = (dy: Int, dx: Int)         // dx > 0
fn steeper(a: Slope, b: Slope) -> Bool: a.dy * b.dx > b.dy * a.dx

var pairs = 0
for i in P:
    var top: ?Slope = none
    var low: ?Slope = none
    for j in P:
        if j > i:
            let cur: Slope = (pts[j].y - pts[i].y, pts[j].x - pts[i].x)
            let at_top = not steeper(top ?? cur, cur)
            let at_low = not steeper(cur, low ?? cur)
            if at_top: top = cur
            if at_low: low = cur
            if at_top or at_low: pairs += 1
print(pairs)
