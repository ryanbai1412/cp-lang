use std.geometry.{Pt3, convex_hull_3d}

let n: Int = read()
var pts: List<Pt3> = []
for _ in 0..n:
    let (x, y, z): (Int, Int, Int) = read()
    pts.push(Pt3(x, y, z))
let faces = convex_hull_3d(pts)
print(faces.len)
