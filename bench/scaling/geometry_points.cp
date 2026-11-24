// The O(1) point, vector, shape and predicate fns of std.geometry, on inputs that vary per call.
use std.geometry.*
let n: Int = read()
var acc = 0
var f = 0.0
let m = 1 << 17
for i in 0..m:
    let p = Pt(i % 97 + 1, i % 89 + 2)
    let q = Pt(i % 83 - 40, i % 79 + 3)
    let r = Pt(i % 71 + 5, i % 73 - 30)
    let s = Pt(i % 67 - 20, i % 61 - 25)
    acc += p.norm2 // timed: geometry.Pt.norm2
    if p.norm2_128 > I128(5): acc += 1 // timed: geometry.Pt.norm2_128
    f += p.len // timed: geometry.Pt.len
    acc += p.perp.x // timed: geometry.Pt.perp
    acc += p.perp_cw.y // timed: geometry.Pt.perp_cw
    f += p.angle // timed: geometry.Pt.angle
    f += p.to_vec.x // timed: geometry.Pt.to_vec
    if p.dot128(q) > I128(0): acc += 1 // timed: geometry.Pt.dot128
    if p.cross128(q) > I128(0): acc += 1 // timed: geometry.Pt.cross128
    acc += p.dist2(q) // timed: geometry.Pt.dist2
    if p.dist2_128(q) > I128(9): acc += 1 // timed: geometry.Pt.dist2_128
    f += p.dist(q) // timed: geometry.Pt.dist
    acc += p.manhattan(q) // timed: geometry.Pt.manhattan
    acc += p.chebyshev(q) // timed: geometry.Pt.chebyshev
    acc += turn(p, q, r) // timed: geometry.turn
    if same_dir(p, q): acc += 1 // timed: geometry.same_dir
    if angle_less(p, q): acc += 1 // timed: geometry.angle_less
    if arg(p) == arg(q): acc += 1 // timed: geometry.arg
    if on_segment(r, p, q): acc += 1 // timed: geometry.on_segment
    if segments_meet(p, q, r, s): acc += 1 // timed: geometry.segments_meet
    acc += dot(p, q) // timed: geometry.dot
    acc += cross(p, q) // timed: geometry.cross
    let a = Vec(F64(i % 97) + 0.5, F64(i % 89) - 3.25)
    let b = a + Vec(F64(i % 7) + 1.5, F64(i % 9) - 4.0)
    let c = Vec(F64(i % 71) + 2.0, F64(i % 73) - 9.5)
    let d = c + Vec(F64(i % 5) + 1.0, 2.0)
    f += a.norm2 // timed: geometry.Vec.norm2
    f += a.len // timed: geometry.Vec.len
    f += a.perp.x // timed: geometry.Vec.perp
    f += a.perp_cw.y // timed: geometry.Vec.perp_cw
    f += a.angle // timed: geometry.Vec.angle
    f += Vec.from_angle(F64(i) * 0.001).x // timed: geometry.Vec.from_angle
    f += a.unit.x // timed: geometry.Vec.unit
    f += a.rot(b.unit).x // timed: geometry.Vec.rot
    f += a.dot(b) // timed: geometry.Vec.dot
    f += a.cross(b) // timed: geometry.Vec.cross
    f += a.dist2(b) // timed: geometry.Vec.dist2
    f += a.dist(b) // timed: geometry.Vec.dist
    if close(a.x, b.x): acc += 1 // timed: geometry.close
    acc += sign_eps(a.x - b.x) // timed: geometry.sign_eps
    acc += turn_eps(a, b, c) // timed: geometry.turn_eps
    f += angle_between(a, b) // timed: geometry.angle_between
    f += project(c, a, b).x // timed: geometry.project
    f += reflect(c, a, b).x // timed: geometry.reflect
    f += line_dist(c, a, b) // timed: geometry.line_dist
    f += seg_dist(c, a, b) // timed: geometry.seg_dist
    if line_intersect(a, b, c, d) != None: acc += 1 // timed: geometry.line_intersect
    acc += segment_intersect(a, b, c, d).len // timed: geometry.segment_intersect
    if ray_hits_segment(a, b - a, c, d) != None: acc += 1 // timed: geometry.ray_hits_segment
    if circumcenter(a, b, c) != None: acc += 1 // timed: geometry.circumcenter
    if Circle.through(a, b, c) != None: acc += 1 // timed: geometry.Circle.through
    if incenter(a, b, c) != None: acc += 1 // timed: geometry.incenter
    let ca = Circle(a, F64(i % 13) + 5.0)
    let cb = Circle(b, F64(i % 11) + 4.0)
    f += ca.area // timed: geometry.Circle.area
    f += ca.circumference // timed: geometry.Circle.circumference
    if ca.locate(c) == Location.Inside: acc += 1 // timed: geometry.Circle.locate
    acc += circle_line_intersect(ca, c, d).len // timed: geometry.circle_line_intersect
    acc += circle_intersect(ca, cb).len // timed: geometry.circle_intersect
    acc += circle_tangents(ca, d).len // timed: geometry.circle_tangents
    acc += circle_circle_tangents(ca, cb).len // timed: geometry.circle_circle_tangents
    f += circle_circle_area(ca, cb) // timed: geometry.circle_circle_area
    f += invert(c, ca).x // timed: geometry.invert
    let h = HalfPlane.through(a, b) // timed: geometry.HalfPlane.through
    let g = HalfPlane.from_line(a.x, b.y, c.x) // timed: geometry.HalfPlane.from_line
    f += h.side(c) + g.side(d) // timed: geometry.HalfPlane.side
    let u = Pt3(i % 97 + 1, i % 89 - 2, i % 83 + 3)
    let v = Pt3(i % 79 - 4, i % 73 + 5, i % 71 - 6)
    acc += u.norm2 // timed: geometry.Pt3.norm2
    if u.norm2_128 > I128(5): acc += 1 // timed: geometry.Pt3.norm2_128
    f += u.len // timed: geometry.Pt3.len
    f += u.to_vec3.x // timed: geometry.Pt3.to_vec3
    acc += u.dot(v) // timed: geometry.Pt3.dot
    if u.dot128(v) > I128(0): acc += 1 // timed: geometry.Pt3.dot128
    acc += u.cross(v).x // timed: geometry.Pt3.cross
    acc += u.dist2(v) // timed: geometry.Pt3.dist2
    f += u.dist(v) // timed: geometry.Pt3.dist
    acc += u.manhattan(v) // timed: geometry.Pt3.manhattan
    acc += u.chebyshev(v) // timed: geometry.Pt3.chebyshev
    let x = u.to_vec3
    let y = v.to_vec3
    let z = Vec3(F64(i % 7) + 0.5, 2.0, F64(i % 5) - 9.0)
    f += x.norm2 // timed: geometry.Vec3.norm2
    f += x.len // timed: geometry.Vec3.len
    f += x.unit.x // timed: geometry.Vec3.unit
    f += x.dot(y) // timed: geometry.Vec3.dot
    f += x.cross(y).x // timed: geometry.Vec3.cross
    f += x.dist2(y) // timed: geometry.Vec3.dist2
    f += x.dist(y) // timed: geometry.Vec3.dist
    f += x.rot(y, F64(i) * 0.001).x // timed: geometry.Vec3.rot
    f += sphere_dist(x, y) // timed: geometry.sphere_dist
    let pl = Plane3.through(x, y, z) // timed: geometry.Plane3.through
    if pl != None:
        f += pl!.dist(z) // timed: geometry.Plane3.dist
        f += pl!.project(z).x // timed: geometry.Plane3.project
        if line_plane_intersect(z, x + y, pl!) != None: acc += 1 // timed: geometry.line_plane_intersect
    let rc = Rect(i % 50..i % 50 + 30, i % 40..i % 40 + 20)
    let ro = Rect(i % 30..i % 30 + 40, i % 20..i % 20 + 35)
    acc += rc.area // timed: geometry.Rect.area
    if rc.is_empty: acc += 1 // timed: geometry.Rect.is_empty
    if rc.contains(p): acc += 1 // timed: geometry.Rect.contains
    if rc.intersect(ro) != None: acc += 1 // timed: geometry.Rect.intersect
// unit_dir's cost is in the coordinates' size, so they grow with n
for i in 0..m:
    acc += Pt(n * (i % 97 + 1), n * (i % 89 + 2)).unit_dir().x // timed: geometry.Pt.unit_dir
print(acc, yes(f > 0.0))
