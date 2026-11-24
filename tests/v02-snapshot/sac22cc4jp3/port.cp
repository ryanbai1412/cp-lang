// Source: SAC '22 Code Challenge 4 Junior P3, https://dmoj.ca/problem/sac22cc4jp3
// Problem: n <= 1e5 points (x, y) off the origin, |x|, |y| <= 1e9, and a whole angle A in
//   0..=359 degrees. Print the 1-based index of the point whose direction from the origin
//   is closest to A; the closest is at least 1e-4 closer than any other.
// Solution: take each direction with atan2 in degrees; the gap to A is the short way round
//   the circle, min(t, 360 - t) with t normalized to 0..360. Keep the first strictly smaller
//   gap, compared as computed with no tolerance, as in the original. O(n).
// original: corpus/submissions/4416076.cpp
use real.float.{PI}

let n: Int = read()
let aim: Int = read()

// the angle between two directions in degrees, 0..=180
fn gap(a: F64, b: F64) -> F64:
    var turn = a - b
    while turn < 0.0: turn += 360
    while turn >= 360.0: turn -= 360
    min(turn, 360 - turn)

var best: ?(gap: F64, id: Int) = None
for id in 1..=n:
    let x: F64 = read()
    let y: F64 = read()
    let g = gap(F64(aim), atan2(y, x) * 180 / PI)
    if best == None or g < best!.gap: best = (gap: g, id:)
print(best!.id)
