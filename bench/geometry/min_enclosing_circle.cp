use std.geometry.{Vec, min_enclosing_circle}

let n: Int = read()
var pts: List<Vec> = []
for _ in 0..n:
    let (x, y): (F64, F64) = read()
    pts.push(Vec(x, y))
let m = min_enclosing_circle(pts)
print(round(m.c.x * 100.0), round(m.c.y * 100.0), round(m.r * 100.0))
