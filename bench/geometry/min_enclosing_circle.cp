use std.geometry.{Vec, min_enclosing_circle}

let n: Int = read()
var pts: List<Vec> = []
for _ in 0..n:
    let (x, y): (F64, F64) = read()
    pts.push(Vec(x, y))
let (c, r) = min_enclosing_circle(pts)
print(round(c.x * 100.0), round(c.y * 100.0), round(r * 100.0))
