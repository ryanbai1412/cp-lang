use std.geometry.{Vec, Circle, circle_tangents, circle_circle_tangents}

let n: Int = read()
var cnt = 0
var acc = 0.0
for _ in 0..n:
    let (x1, y1, r1, x2, y2, r2): (F64, F64, F64, F64, F64, F64) = read()
    let c1 = Circle(Vec(x1, y1), r1)
    let c2 = Circle(Vec(x2, y2), r2)
    let pt_ts = circle_tangents(c1, c2.c)
    cnt += pt_ts.len
    for p in pt_ts: acc += p.x + p.y
    let out_ts = circle_circle_tangents(c1, c2)
    cnt += out_ts.len
    for (a, b) in out_ts: acc += a.x + b.y
    let in_ts = circle_circle_tangents(c1, c2, inner: true)
    cnt += in_ts.len
    for (a, b) in in_ts: acc += a.x + b.y
print(cnt, round(acc * 100.0))
