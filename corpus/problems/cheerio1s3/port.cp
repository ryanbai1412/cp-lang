// Cheerio Contest 1 S3: points with increasing x. Count the pairs i < j for
// which the slope from i to j is the steepest or the shallowest among the
// slopes from i to every point up to j.
use std.geometry.{cross}

let n: 1..=2000 = read()
let pts: [0..n]Pt = read()

var pairs = 0
for i in 0..n:
    var top: ?Pt = none  // steepest and shallowest directions so far
    var low: ?Pt = none
    for j in i + 1..n:
        let cur = pts[j] - pts[i]
        let at_top = cross(top ?? cur, cur) >= 0
        let at_low = cross(cur, low ?? cur) >= 0
        if at_top: top = cur
        if at_low: low = cur
        if at_top or at_low: pairs += 1
print(pairs)
