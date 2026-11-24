// expect: ok, output "ok"
// input: "1 300\n"
// model: Pt3
// model: Vec3
// model: Plane3
// model: convex_hull_3d
// model: convex_hull_3d_idx
// model: polyhedron_volume6
// model: line_plane_intersect
// model: sphere_dist
// 3D points and vectors against coordinate formulas, rotations about the axes against the 2D
// rotation of the other two coordinates, and convex hulls of points with no four coplanar
// against every triple with all points on one side. Reads `seed n`; prints the first op that
// differs.
use std.rand.{Rng}
use std.geometry.{Plane3, Pt3, Vec3, convex_hull_3d, convex_hull_3d_idx, line_plane_intersect, polyhedron_volume6, sphere_dist}
let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
fn rp(mut rng: Rng, k: Int) -> Pt3 = Pt3(rng.int(-k, k), rng.int(-k, k), rng.int(-k, k))
fn fv(p: Pt3) -> Vec3 = Vec3(F64(p.x), F64(p.y), F64(p.z))
fn near(a: F64, b: F64) -> Bool = abs(a - b) <= 1e-6 * max(1.0, abs(b))
fn vnear(a: Vec3, b: Vec3) -> Bool = near(a.x, b.x) and near(a.y, b.y) and near(a.z, b.z)
fn det(a: Pt3, b: Pt3, c: Pt3) -> Int = a.x * (b.y * c.z - b.z * c.y) - a.y * (b.x * c.z - b.z * c.x) + a.z * (b.x * c.y - b.y * c.x)
// Six times the signed volume of the tetrahedron (a, b, c, d).
fn orient(a: Pt3, b: Pt3, c: Pt3, d: Pt3) -> Int = det(b - a, c - a, d - a)
fn col(a: Pt3, b: Pt3, c: Pt3) -> Bool:
    let (u, v) = (b - a, c - a)
    u.y * v.z == u.z * v.y and u.z * v.x == u.x * v.z and u.x * v.y == u.y * v.x
var bad = ""
for k in 0..n:
    if bad != "": break
    let op = rng.int(0, 2)
    if op == 0:
        let (a, b) = (rp(mut rng, 50), rp(mut rng, 50))
        let m = rng.int(-5, 5)
        let ab = b - a
        if a + b != Pt3(a.x + b.x, a.y + b.y, a.z + b.z) or ab != Pt3(b.x - a.x, b.y - a.y, b.z - a.z) or -a != Pt3(-a.x, -a.y, -a.z) or a * m != Pt3(a.x * m, a.y * m, a.z * m): bad = "op {k}: Pt3 arithmetic {a} {b}"
        elif a.norm2 != a.x * a.x + a.y * a.y + a.z * a.z or a.norm2_128 != I128(a.norm2) or a.len != sqrt(F64(a.norm2)) or a.to_vec3 != fv(a): bad = "op {k}: Pt3 norms {a}"
        elif a.dot(b) != a.x * b.x + a.y * b.y + a.z * b.z or a.dot128(b) != I128(a.dot(b)): bad = "op {k}: Pt3.dot {a} {b}"
        elif a.cross(b).dot(a) != 0 or a.cross(b).dot(b) != 0 or a.cross(b).dot(Pt3(1, 2, 3)) != det(a, b, Pt3(1, 2, 3)): bad = "op {k}: Pt3.cross {a} {b}"
        elif a.dist2(b) != ab.dot(ab) or a.dist(b) != sqrt(F64(ab.dot(ab))): bad = "op {k}: Pt3.dist {a} {b}"
        elif a.manhattan(b) != abs(ab.x) + abs(ab.y) + abs(ab.z) or a.chebyshev(b) != [abs(ab.x), abs(ab.y), abs(ab.z)].max()!: bad = "op {k}: Pt3 metrics {a} {b}"
        if bad != "": break
        let (u, v) = (fv(a), fv(b))
        let s = F64(m) / 4.0
        if u + v != fv(a + b) or v - u != fv(ab) or -u != fv(-a) or u * F64(m) != fv(a * m) or not vnear(u / 4.0, Vec3(u.x / 4.0, u.y / 4.0, u.z / 4.0)): bad = "op {k}: Vec3 arithmetic {u} {v}"
        elif u.norm2 != F64(a.norm2) or not near(u.len, sqrt(F64(a.norm2))) or u.dot(v) != F64(a.dot(b)) or u.cross(v) != fv(a.cross(b)): bad = "op {k}: Vec3 products {u} {v}"
        elif u.dist2(v) != F64(a.dist2(b)) or not near(u.dist(v), a.dist(b)): bad = "op {k}: Vec3.dist {u} {v}"
        elif a != Pt3(0, 0, 0) and not (near(u.unit.len, 1.0) and vnear(u.unit * u.len, u)): bad = "op {k}: Vec3.unit {u}"
        if bad != "": break
        let t = F64(rng.int(-628, 628)) / 100.0
        let (c, sn) = (cos(t), sin(t))
        let ax = rng.int(0, 2)
        let sg = F64(2 * rng.int(0, 1) - 1)
        let axis = if ax == 0: Vec3(sg, 0.0, 0.0) elif ax == 1: Vec3(0.0, sg, 0.0) else: Vec3(0.0, 0.0, sg)
        let st = sn * sg
        let want = if ax == 0: Vec3(u.x, u.y * c - u.z * st, u.y * st + u.z * c) elif ax == 1: Vec3(u.z * st + u.x * c, u.y, u.z * c - u.x * st) else: Vec3(u.x * c - u.y * st, u.x * st + u.y * c, u.z)
        if not vnear(u.rot(axis, t), want): bad = "op {k}: {u}.rot({axis}, {t}) = {u.rot(axis, t)} want {want}"
        elif b != Pt3(0, 0, 0):
            let w = v.unit
            let r = u.rot(w, t)
            if not (near(r.len, u.len) and near(r.dot(w), u.dot(w)) and vnear(r.rot(w, -t), u)): bad = "op {k}: {u}.rot about {w}"
        if bad != "": break
        let sd = sphere_dist(u, v, r: s)
        let wd = if a == Pt3(0, 0, 0) or b == Pt3(0, 0, 0): 0.0 else: s * acos(max(-1.0, min(1.0, u.dot(v) / (u.len * v.len))))
        if abs(sd - wd) > 1e-6: bad = "op {k}: sphere_dist({u}, {v}, r: {s}) = {sd} want {wd}"
    elif op == 1:
        let (a, b, c) = (rp(mut rng, 4), rp(mut rng, 4), rp(mut rng, 4))
        let p = fv(rp(mut rng, 6))
        let pl = Plane3.through(fv(a), fv(b), fv(c))
        let m = (b - a).cross(c - a)
        if (pl == None) != (m == Pt3(0, 0, 0)): bad = "op {k}: Plane3.through({a}, {b}, {c})"
        elif pl != None:
            let h = pl!
            let on = (q: Vec3) => abs(h.n.dot(q) - h.d) <= 1e-9
            if not (near(h.n.len, 1.0) and h.n.dot(fv(m)) > 0.0 and on(fv(a)) and on(fv(b)) and on(fv(c))): bad = "op {k}: Plane3.through({a}, {b}, {c}) = {h}"
            elif not near(h.dist(p), fv(m).dot(p - fv(a)) / fv(m).len) or not (on(h.project(p)) and vnear((p - h.project(p)).cross(h.n), Vec3(0.0, 0.0, 0.0))): bad = "op {k}: Plane3 dist or project of {p}"
        if bad != "": break
        let nn = rp(mut rng, 3)
        let pl2 = Plane3(fv(nn), F64(rng.int(-5, 5)))
        let (la, lb) = (fv(rp(mut rng, 5)), fv(rp(mut rng, 5)))
        let den = pl2.n.dot(lb - la)
        let got = line_plane_intersect(la, lb, pl2)
        if (got == None) != (den == 0.0): bad = "op {k}: line_plane_intersect({la}, {lb}, {pl2})"
        elif got != None:
            let q = got!
            if not (abs(pl2.n.dot(q) - pl2.d) <= 1e-6 and vnear((q - la).cross(lb - la), Vec3(0.0, 0.0, 0.0))): bad = "op {k}: line_plane_intersect({la}, {lb}, {pl2}) = {q}"
    else:
        // Distinct points (and some repeats) with no three collinear and no four coplanar.
        var pts: List<Pt3> = []
        for _ in 0..rng.int(0, 8):
            let p = rp(mut rng, 1000)
            var ok = true
            for i in 0..pts.len:
                for j in i + 1..pts.len:
                    if col(pts[i], pts[j], p): ok = false
                    for l in j + 1..pts.len:
                        if orient(pts[i], pts[j], pts[l], p) == 0: ok = false
            if ok and not pts.any(q => q == p): pts.push(p)
        let distinct = pts.len
        for _ in 0..rng.int(0, 2):
            if distinct > 0: pts.push(pts[rng.int(0, distinct - 1)])
        var want: List<(Int, Int, Int)> = []
        if distinct >= 4:
            for i in 0..distinct:
                for j in i + 1..distinct:
                    for l in j + 1..distinct:
                        let s = [orient(pts[i], pts[j], pts[l], pts[q]) for q in 0..distinct if q != i and q != j and q != l]
                        if s.all(v => v < 0): want.push((i, j, l))
                        elif s.all(v => v > 0): want.push((i, l, j))
        want.sort()
        let got = convex_hull_3d(pts)
        if got != want: bad = "op {k}: convex_hull_3d({pts}) = {got} want {want}"
        let hi = [(a.ord, b.ord, c.ord) for (a, b, c) in convex_hull_3d_idx(pts)]
        if bad != "": break
        if hi != want: bad = "op {k}: convex_hull_3d_idx({pts})"
        if bad != "": break
        var v6 = 0
        for (i, j, l) in want: v6 += det(pts[i], pts[j], pts[l])
        if polyhedron_volume6(pts, want) != I128(v6) or (want.len > 0 and v6 <= 0): bad = "op {k}: polyhedron_volume6 of the hull"
        elif pts.len > 0:
            let faces = [(rng.int(0, pts.len - 1), rng.int(0, pts.len - 1), rng.int(0, pts.len - 1)) for _ in 0..rng.int(0, 4)]
            var w = 0
            for (i, j, l) in faces: w += det(pts[i], pts[j], pts[l])
            if polyhedron_volume6(pts, faces) != I128(w): bad = "op {k}: polyhedron_volume6({pts}, {faces})"
        let flat = [Pt3(rng.int(-5, 5), rng.int(-5, 5), 7) for _ in 0..rng.int(0, 6)]
        if convex_hull_3d(flat) != []: bad = "op {k}: convex_hull_3d of coplanar {flat}"
print(if bad == "": "ok" else: bad)
