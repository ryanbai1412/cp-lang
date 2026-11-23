use std.geometry.{convex_hull}

let n: Int = read()
var pts: List<Pt> = []
for _ in 0..n:
    let (x, y): (Int, Int) = read()
    pts.push(Pt(x, y))
let h = convex_hull(pts)
var checksum = 0
for p in h:
    checksum += p.x + p.y
print(h.len, checksum)
