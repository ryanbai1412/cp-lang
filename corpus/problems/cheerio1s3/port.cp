// Source: Cheerio Contest 1 S3 (Stock Trading), https://dmoj.ca/problem/cheerio1s3
// Problem: n <= 5000 points (t_i, p_i) with t increasing, coordinates <= 1e9. Count the pairs
//   i < j whose segment is a trendline: points i..=j all lie on or above it, or all on or below.
// Solution: from each i, sweep j keeping the steepest and the shallowest direction to the points
//   seen so far, compared by cross product: (i, j) counts when j's direction is at least the
//   steepest (all below or on) or at most the shallowest (all above or on). O(n^2).
use std.geometry.{cross}

let n: 1..=2000 = read()
let pts: [0..n]Pt = read()

var pairs = 0
for i in 0..n:
    var top: ?Pt = None  // steepest and shallowest directions so far
    var low: ?Pt = None
    for j in i + 1..n:
        let cur = pts[j] - pts[i]
        let at_top = cross(top ?? cur, cur) >= 0
        let at_low = cross(cur, low ?? cur) >= 0
        if at_top: top = cur
        if at_low: low = cur
        if at_top or at_low: pairs += 1
print(pairs)
