use std.geometry.{Vec, polygon_area, polygon_cut}

let n: Int = read()
var poly = [
    Vec(-10000.0, -10000.0),
    Vec(10000.0, -10000.0),
    Vec(10000.0, 10000.0),
    Vec(-10000.0, 10000.0),
]
for _ in 0..n:
    let (ax, ay, bx, by): (F64, F64, F64, F64) = read()
    poly = polygon_cut(poly, Vec(ax, ay), Vec(bx, by))
print(poly.len, round(polygon_area(poly) * 1000.0))
