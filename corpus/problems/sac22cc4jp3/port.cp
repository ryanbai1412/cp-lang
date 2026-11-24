// Source: SAC '22 Code Challenge 4 Junior P3, https://dmoj.ca/problem/sac22cc4jp3
// Problem: n <= 1e5 points (x, y) off the origin, |x|, |y| <= 1e9, and a whole angle A in
//   0..=359 degrees. Print the 1-based index of the point whose direction from the origin
//   is closest to A; the closest is at least 1e-4 closer than any other.
// Solution: take each direction with atan2 in degrees; the gap to A is the short way round
//   the circle, min(t, 360 - t) with t the difference reduced into 0..360. The first point with
//   the smallest gap wins. O(n).
// original: corpus/submissions/4416076.cpp
// Bounds: n: 1..=1e5; aim: 0..=359; points.x, points.y: -1e9..=1e9
use real.float.{PI}

let n: 1..=1e5 = read()
let aim: 0..=359 = read()
domain Point = 1..=n
let points: [Point](x: -1e9..=1e9, y: -1e9..=1e9) = read()

// the angle between direction `d` and the aim, in degrees, 0..=180
fn gap(d: F64) -> F64:
    let turn = abs(d - aim)  // 0..540: atan2 gives -180..=180
    let within = if turn >= 360.0: turn - 360 else: turn
    min(within, 360 - within)

print(points.argmin(by: p => gap(atan2(F64(p.y), F64(p.x)) * 180 / PI))!)
