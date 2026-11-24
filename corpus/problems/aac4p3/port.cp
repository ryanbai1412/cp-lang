// Source: An Animal Contest 4 P3 (Snowy Slopes), https://dmoj.ca/problem/aac4p3
// limits: 2000 ms, 256 MB
// Problem: 2 <= n <= 1e5 points with distinct x and distinct y (coordinates 1..=1e9) and
//   m <= 20 slopes k_i / d_i (nonzero, |k|, |d| <= 1e9). Count the pairs of points whose line
//   has one of the given slopes.
// Solution: reduce each slope to lowest terms with a positive denominator, so equal slopes
//   coincide. Points on one line of slope a / b share b * y - a * x, so per distinct slope tally
//   the points by that key and add c choose 2 per line: O(m n) expected.
// Bounds: n: 2..=1e5; m: 1..=20; points.x, points.y: 1..=1e9; slopes.k, slopes.d: -1e9..=1e9
let (n, m): (2..=1e5, 1..=20) = read()
let points: [n](x: 1..=1e9, y: 1..=1e9) = read()
let slopes: [m](k: -1e9..=1e9, d: -1e9..=1e9) = read()
assert all(s.k != 0 and s.d != 0 for s in slopes)

// k / d in lowest terms with a positive denominator, so equal slopes coincide
fn lowest(k: Int, d: Int) -> (a: Int, b: Int):
    let g = if d > 0: gcd(k, d) else: -gcd(k, d)
    (a: k / g, b: d / g)

let directions = Set(lowest(s.k, s.d) for s in slopes)
// points on one line of slope a / b share b * y - a * x
print(sum(
    c * (c - 1) / 2
    for (a, b) in directions
    for c in points.tally(by: p => b * p.y - a * p.x).values
))
