// expect: ok, output "YES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES YES\nYES"
use std.geometry.{Pt, any_segments_meet, Pt3, Vec, Circle, circle_polygon_area, any_segments_meet_idx, convex_hull_3d, convex_hull_3d_idx}
print(yes(convex_hull_3d([Pt3(0, 0, 0), Pt3(1, 0, 0), Pt3(0, 1, 0), Pt3(0, 0, 1)]) == [(0, 1, 3), (0, 2, 1), (0, 3, 2), (1, 2, 3)]))
print(yes(convex_hull_3d([Pt3(0, 0, 0), Pt3(4, 0, 0), Pt3(0, 4, 0), Pt3(0, 0, 4), Pt3(1, 1, 1)]) == [(0, 1, 3), (0, 2, 1), (0, 3, 2), (1, 2, 3)]))
print(yes(convex_hull_3d([Pt3(0, 0, 0), Pt3(1, 0, 0), Pt3(0, 1, 0)]) == []))
print(yes(convex_hull_3d([Pt3(0, 0, 0), Pt3(1, 0, 0), Pt3(0, 1, 0), Pt3(1, 1, 0), Pt3(2, 2, 0)]) == []))
var cube: List<Pt3> = []
for x in 0..3:
    for y in 0..3:
        for z in 0..3: cube.push(Pt3(x, y, z))
cube.push(Pt3(0, 0, 0))
let ch = convex_hull_3d(cube)
var corners: List<Int> = []
for (a, b, c) in ch:
    corners.push(a)
    corners.push(b)
    corners.push(c)
print(yes(ch.len == 12 and corners.all(v => cube[v].x % 2 == 0 and cube[v].y % 2 == 0 and cube[v].z % 2 == 0)))
domain D4 = 0..4
let tet = [Pt3(0, 0, 0), Pt3(1, 0, 0), Pt3(0, 1, 0), Pt3(0, 0, 1)]
let hi = convex_hull_3d_idx([D4] i => tet[i.to_idx])
print(yes(hi.len == 4 and hi[0].0 == D4(0) and hi[0].1 == D4(1) and hi[0].2 == D4(3)))
print(yes(circle_polygon_area(Circle(Vec(0.0, 0.0), 10.0), [Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(1.0, 1.0), Vec(0.0, 1.0)]) == 1.0))
print(yes(circle_polygon_area(Circle(Vec(0.0, 0.0), 10.0), [Vec(0.0, 0.0), Vec(0.0, 1.0), Vec(1.0, 1.0), Vec(1.0, 0.0)]) == -1.0))
let ca = circle_polygon_area(Circle(Vec(0.0, 0.0), 1.0), [Vec(-2.0, -2.0), Vec(2.0, -2.0), Vec(2.0, 2.0), Vec(-2.0, 2.0)])
print(yes(ca > 3.14159265 and ca < 3.14159266))
let ch2 = circle_polygon_area(Circle(Vec(1.0, 0.0), 1.0), [Vec(1.0, -5.0), Vec(5.0, -5.0), Vec(5.0, 5.0), Vec(1.0, 5.0)])
print(yes(ch2 > 1.5707963 and ch2 < 1.5707964))
domain S2 = 0..2
let sg = [(a: Pt(0, 0), b: Pt(1, 1)), (a: Pt(1, 1), b: Pt(2, 0))]
let m = any_segments_meet_idx([S2] i => sg[i.to_idx])
print(yes(m != None and m!.0 == S2(0) and m!.1 == S2(1)))
print(yes(any_segments_meet_idx([S2] i => sg[i.to_idx], strict: true) == None))
fn rnd(mut s: Int, m: Int) -> Int:
    s = (s * 1103515245 + 12345) % 2147483648
    (s / 65536) % m
fn side(a: Pt3, b: Pt3, c: Pt3, p: Pt3) -> Int:
    let u = b - a
    let v = c - a
    let w = p - a
    (u.y * v.z - u.z * v.y) * w.x + (u.z * v.x - u.x * v.z) * w.y + (u.x * v.y - u.y * v.x) * w.z
var seed = 11
var ok = true
var nonempty = 0
for _ in 0..60:
    let n = 4 + rnd(mut seed, 12)
    let ps = [Pt3(rnd(mut seed, 4), rnd(mut seed, 4), rnd(mut seed, 4)) for _ in 0..n]
    let fs = convex_hull_3d(ps)
    if not fs.is_empty: nonempty += 1
    for (a, b, c) in fs:
        if side(ps[a], ps[b], ps[c], ps[c]) != 0 or a > b or a > c: ok = false
        for q in ps:
            if side(ps[a], ps[b], ps[c], q) > 0: ok = false
        for (u, v) in [(a, b), (b, c), (c, a)]:
            if not fs.any(f => (f.0 == v and f.1 == u) or (f.1 == v and f.2 == u) or (f.2 == v and f.0 == u)): ok = false
        for v in [a, b, c]:
            var w = 0
            while w < v:
                if ps[w] == ps[v]: ok = false
                w += 1
print(yes(ok), yes(nonempty > 40))
print(yes(any_segments_meet([(a: Pt(0, 0), b: Pt(1, 1)), (a: Pt(1, 1), b: Pt(2, 0))]) == Some((0, 1))))
