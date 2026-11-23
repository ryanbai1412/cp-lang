// expect: ok, output "YES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES\nYES"
use std.geometry.{Pt3, Vec, Circle, Rect, close, minkowski_lower, convex_polygons_overlap, hull_width, polygon_centroid, segment_intersect, ray_hits_segment, circle_line_intersect, circle_intersect, circle_tangents, circle_circle_tangents, circle_circle_area, polyhedron_volume6, rects_union_area}
print(yes(minkowski_lower([Pt(0, 0), Pt(1, -1), Pt(2, 0)], [Pt(0, 0), Pt(1, 0)]) == [Pt(0, 0), Pt(1, -1), Pt(2, -1), Pt(3, 0)]))
print(yes(convex_polygons_overlap([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)], [Pt(1, 1), Pt(3, 1), Pt(3, 3), Pt(1, 3)]) == true))
print(yes(convex_polygons_overlap([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)], [Pt(2, 2), Pt(3, 2), Pt(3, 3)]) == true))
print(yes(convex_polygons_overlap([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)], [Pt(5, 5), Pt(6, 5), Pt(6, 6)]) == false))
print(yes(hull_width([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)]) == 2.0))
print(yes(hull_width([Pt(0, 0), Pt(4, 0), Pt(4, 1), Pt(0, 1)]) == 1.0))
print(yes(polygon_centroid([Vec(0.0, 0.0), Vec(2.0, 0.0), Vec(2.0, 2.0), Vec(0.0, 2.0)]) == Vec(1.0, 1.0)))
print(yes(segment_intersect(Vec(0.0, 0.0), Vec(2.0, 2.0), Vec(0.0, 2.0), Vec(2.0, 0.0)) == [Vec(1.0, 1.0)]))
print(yes(segment_intersect(Vec(0.0, 0.0), Vec(2.0, 0.0), Vec(3.0, 0.0), Vec(1.0, 0.0)) == [Vec(1.0, 0.0), Vec(2.0, 0.0)]))
print(yes(segment_intersect(Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(0.0, 1.0), Vec(1.0, 1.0)) == []))
print(yes(ray_hits_segment(Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(2.0, -1.0), Vec(2.0, 1.0)) == Some(Vec(2.0, 0.0))))
print(yes(ray_hits_segment(Vec(0.0, 0.0), Vec(-1.0, 0.0), Vec(2.0, -1.0), Vec(2.0, 1.0)) == None))
print(yes(ray_hits_segment(Vec(0.0, 0.0), Vec(1.0, 0.0), Vec(3.0, 0.0), Vec(5.0, 0.0)) == Some(Vec(3.0, 0.0))))
print(yes(circle_line_intersect(Circle(Vec(0.0, 0.0), 5.0), Vec(-10.0, 3.0), Vec(10.0, 3.0)) == [Vec(-4.0, 3.0), Vec(4.0, 3.0)]))
print(yes(circle_line_intersect(Circle(Vec(0.0, 0.0), 5.0), Vec(10.0, 3.0), Vec(-10.0, 3.0)) == [Vec(4.0, 3.0), Vec(-4.0, 3.0)]))
print(yes(circle_line_intersect(Circle(Vec(0.0, 0.0), 5.0), Vec(-10.0, 5.0), Vec(10.0, 5.0)) == [Vec(0.0, 5.0)]))
print(yes(circle_line_intersect(Circle(Vec(0.0, 0.0), 5.0), Vec(-10.0, 9.0), Vec(10.0, 9.0)) == []))
print(yes(circle_intersect(Circle(Vec(0.0, 0.0), 5.0), Circle(Vec(8.0, 0.0), 5.0)) == [Vec(4.0, -3.0), Vec(4.0, 3.0)]))
print(yes(circle_intersect(Circle(Vec(0.0, 0.0), 1.0), Circle(Vec(2.0, 0.0), 1.0)) == [Vec(1.0, 0.0)]))
print(yes(circle_intersect(Circle(Vec(0.0, 0.0), 1.0), Circle(Vec(5.0, 0.0), 1.0)) == []))
print(yes(circle_tangents(Circle(Vec(0.0, 0.0), 1.0), Vec(1.0, 0.0)) == [Vec(1.0, 0.0)]))
print(yes(circle_tangents(Circle(Vec(0.0, 0.0), 2.0), Vec(1.0, 0.0)) == []))
fn ex0() -> Bool:
    let t = circle_tangents(Circle(Vec(0.0, 0.0), 3.0), Vec(5.0, 0.0))
    t.len == 2 and close(t[0].x, 1.8) and close(t[0].y, -2.4) and close(t[1].y, 2.4)
print(yes(ex0()))
print(yes(circle_circle_tangents(Circle(Vec(0.0, 0.0), 1.0), Circle(Vec(4.0, 0.0), 1.0)) == [(Vec(0.0, -1.0), Vec(4.0, -1.0)), (Vec(0.0, 1.0), Vec(4.0, 1.0))]))
print(yes(circle_circle_tangents(Circle(Vec(0.0, 0.0), 1.0), Circle(Vec(4.0, 0.0), 1.0), inner: true).len == 2))
print(yes(circle_circle_tangents(Circle(Vec(0.0, 0.0), 2.0), Circle(Vec(1.0, 0.0), 2.0), inner: true) == []))
print(yes(circle_circle_area(Circle(Vec(0.0, 0.0), 1.0), Circle(Vec(5.0, 0.0), 1.0)) == 0.0))
print(yes(circle_circle_area(Circle(Vec(0.0, 0.0), 2.0), Circle(Vec(0.0, 0.0), 1.0)) == atan2(0.0, -1.0)))
print(yes(close(circle_circle_area(Circle(Vec(0.0, 0.0), 1.0), Circle(Vec(1.0, 0.0), 1.0)), 1.228369698608757)))
print(yes(polyhedron_volume6([Pt3(0, 0, 0), Pt3(1, 0, 0), Pt3(0, 1, 0), Pt3(0, 0, 1)], [(0, 1, 3), (0, 2, 1), (0, 3, 2), (1, 2, 3)]) == 1))
print(yes(polyhedron_volume6([Pt3(0, 0, 0), Pt3(1, 0, 0), Pt3(0, 1, 0), Pt3(0, 0, 1)], [(0, 3, 1), (0, 1, 2), (0, 2, 3), (1, 3, 2)]) == -1))
print(yes(rects_union_area([Rect(0..2, 0..2), Rect(1..3, 1..3)]) == 7))
print(yes(rects_union_area([Rect(0..2, 0..2), Rect(0..2, 0..2)]) == 4))
print(yes(rects_union_area([]) == 0))
var seed = 12345
fn rnd(mut seed: Int, n: Int) -> Int:
    seed = (seed * 1103515245 + 12345) % 2147483648
    seed % n
var ok = true
for _ in 0..60:
    var rs: List<Rect> = []
    for _ in 0..rnd(mut seed, 6):
        let x0 = rnd(mut seed, 8)
        let y0 = rnd(mut seed, 8)
        rs.push(Rect(x0..x0 + rnd(mut seed, 5), y0..y0 + rnd(mut seed, 5)))
    var brute = 0
    for x in 0..12:
        for y in 0..12:
            if rs.any(r => r.x.start <= x and x < r.x.end and r.y.start <= y and y < r.y.end): brute += 1
    if rects_union_area(rs) != brute: ok = false
print(yes(ok))
