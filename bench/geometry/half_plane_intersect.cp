use std.geometry.{Vec, HalfPlane, half_plane_intersect, polygon_area}

let n: Int = read()
var planes: List<HalfPlane> = []
for _ in 0..n:
    let (px, py, dx, dy): (F64, F64, F64, F64) = read()
    planes.push(HalfPlane(Vec(px, py), Vec(dx, dy)))
let poly = half_plane_intersect(planes)
print(poly.len, round(polygon_area(poly) * 1000.0))
