use std.geometry.{Vec, Circle, circle_intersect}

let n: Int = read()
var cnt = 0
var acc = 0.0
for _ in 0..n:
    let (x1, y1, r1, x2, y2, r2): (F64, F64, F64, F64, F64, F64) = read()
    let pts = circle_intersect(Circle(Vec(x1, y1), r1), Circle(Vec(x2, y2), r2))
    cnt += pts.len
    for p in pts: acc += p.x + p.y
print(cnt, round(acc * 100.0))
