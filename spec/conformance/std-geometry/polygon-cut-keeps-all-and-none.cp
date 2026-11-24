// expect: ok, output "3 0"
use std.geometry.{Vec}
let tri = [Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(0.0, 1.0)]
print(polygon_cut(tri, Vec(0.0, -5.0), Vec(1.0, -5.0)).len, polygon_cut(tri, Vec(0.0, 5.0), Vec(1.0, 5.0)).len)
