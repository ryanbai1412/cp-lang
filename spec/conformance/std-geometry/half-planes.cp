// expect: ok, output "YES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES"
use std.geometry.{Vec, HalfPlane, half_plane_intersect, polygon_kernel, convex_polygon_intersect}
print(yes(half_plane_intersect([HalfPlane.through(Vec(0.0, 0.0), Vec(1.0, 0.0)), HalfPlane.through(Vec(1.0, 0.0), Vec(1.0, 1.0)), HalfPlane.through(Vec(1.0, 1.0), Vec(0.0, 1.0)), HalfPlane.through(Vec(0.0, 1.0), Vec(0.0, 0.0))]) == [Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(1.0, 1.0), Vec(0.0, 1.0)]))
print(yes(half_plane_intersect([HalfPlane.through(Vec(0.0, 0.0), Vec(1.0, 0.0))]) == []))
print(yes(half_plane_intersect([HalfPlane.through(Vec(0.0, 0.0), Vec(1.0, 0.0))], bound: Some(1.0)) == [Vec(-1.0, 0.0), Vec(1.0, 0.0), Vec(1.0, 1.0), Vec(-1.0, 1.0)]))
print(yes(polygon_kernel([Vec(0.0, 0.0), Vec(2.0, 0.0), Vec(2.0, 2.0), Vec(0.0, 2.0)]) == [Vec(0.0, 0.0), Vec(2.0, 0.0), Vec(2.0, 2.0), Vec(0.0, 2.0)]))
print(yes(convex_polygon_intersect([Vec(0.0, 0.0), Vec(2.0, 0.0), Vec(2.0, 2.0), Vec(0.0, 2.0)], [Vec(1.0, 1.0), Vec(3.0, 1.0), Vec(3.0, 3.0), Vec(1.0, 3.0)]) == [Vec(1.0, 1.0), Vec(2.0, 1.0), Vec(2.0, 2.0), Vec(1.0, 2.0)]))
print(yes(convex_polygon_intersect([Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(0.0, 1.0)], [Vec(5.0, 5.0), Vec(6.0, 5.0), Vec(5.0, 6.0)]) == []))
let sq = [Vec(0.0, 0.0), Vec(4.0, 0.0), Vec(4.0, 4.0), Vec(0.0, 4.0)]
let notch = [Vec(0.0, 0.0), Vec(4.0, 0.0), Vec(4.0, 4.0), Vec(2.0, 1.0), Vec(0.0, 4.0)]
let k = polygon_kernel(notch)
print(yes(k.len == 3 and abs(k[0].x - 4.0 / 3.0) < 1e-9 and abs(k[1].x - 8.0 / 3.0) < 1e-9 and k[2] == Vec(2.0, 1.0)))
print(yes(polygon_kernel(sq) == sq))
print(yes(half_plane_intersect([HalfPlane.through(Vec(0.0, 0.0), Vec(1.0, 0.0)), HalfPlane.through(Vec(0.0, -1.0), Vec(-1.0, -1.0))], bound: Some(5.0)) == []))
print(yes(convex_polygon_intersect(sq, [Vec(4.0, 0.0), Vec(6.0, 0.0), Vec(6.0, 4.0), Vec(4.0, 4.0)]) == []))
fn rnd(mut s: Int, m: Int) -> Int:
    s = (s * 1103515245 + 12345) % 2147483648
    s % m
fn inside(poly: List<Vec>, q: Vec) -> Int:
    if poly.len < 3: return -1
    var mn = 1000000.0
    for i in 0..poly.len:
        let a = poly[i]
        let b = poly[(i + 1) % poly.len]
        let c = (b - a).cross(q - a) / a.dist(b)
        if c < mn: mn = c
    if mn > 1e-6: 1 else: if mn < -1e-6: -1 else: 0
fn tri(mut seed: Int) -> List<Vec>:
    let a = Vec(F64(rnd(mut seed, 21) - 10), F64(rnd(mut seed, 21) - 10))
    let b = Vec(F64(rnd(mut seed, 21) - 10), F64(rnd(mut seed, 21) - 10))
    let c = Vec(F64(rnd(mut seed, 21) - 10), F64(rnd(mut seed, 21) - 10))
    if (b - a).cross(c - a) >= 0.0: [a, b, c] else: [a, c, b]
var seed = 7
var ok = true
for _ in 0..150:
    let p = tri(mut seed)
    let q = tri(mut seed)
    if (p[1] - p[0]).cross(p[2] - p[0]) == 0.0 or (q[1] - q[0]).cross(q[2] - q[0]) == 0.0: continue
    let r = convex_polygon_intersect(p, q)
    for _ in 0..30:
        let z = Vec(F64(rnd(mut seed, 2001) - 1000) / 97.0, F64(rnd(mut seed, 2001) - 1000) / 97.0)
        let a = inside(p, z)
        let b = inside(q, z)
        let c = inside(r, z)
        if a == 1 and b == 1 and c != 1: ok = false
        if (a == -1 or b == -1) and c == 1: ok = false
print(yes(ok))
