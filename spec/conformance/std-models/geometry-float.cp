// expect: ok, output "ok"
// input: "1 300\n"
// model: Vec
// model: Circle
// model: HalfPlane
// model: close
// model: sign_eps
// model: turn_eps
// model: angle_between
// model: project
// model: reflect
// model: line_dist
// model: seg_dist
// model: line_intersect
// model: segment_intersect
// model: ray_hits_segment
// model: circumcenter
// model: incenter
// model: circle_line_intersect
// model: circle_intersect
// model: circle_tangents
// model: circle_circle_tangents
// model: invert
// Float geometry on integer-valued points and radii, so the brute force can count cases exactly:
// Vec against the formulas, projections, distances and intersections against an exact rational
// parameter solve and the defining properties (on both lines, equidistant, tangent), and circle
// meets against squared-distance comparisons. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.geometry.{Circle, HalfPlane, Location, Vec, angle_between, circle_circle_tangents, circle_intersect, circle_line_intersect, circle_tangents, circumcenter, close, incenter, invert, line_dist, line_intersect, project, ray_hits_segment, reflect, seg_dist, segment_intersect, sign_eps, turn_eps}
let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
fn rv(mut rng: Rng, k: Int) -> Vec = Vec(F64(rng.int(-k, k)), F64(rng.int(-k, k)))
fn cr(a: Vec, b: Vec) -> F64 = a.x * b.y - a.y * b.x
fn dt(a: Vec, b: Vec) -> F64 = a.x * b.x + a.y * b.y
fn nr(a: F64, b: F64) -> Bool = abs(a - b) <= 1e-6
fn vn(a: Vec, b: Vec) -> Bool = nr(a.x, b.x) and nr(a.y, b.y)
fn onseg(p: Vec, a: Vec, b: Vec) -> Bool = cr(p - a, b - a) == 0.0 and dt(p - a, p - b) <= 0.0
fn online(p: Vec, a: Vec, b: Vec) -> Bool = abs(cr(p - a, b - a)) <= 1e-6 * max(1.0, (b - a).len)
fn ldist(p: Vec, a: Vec, b: Vec) -> F64 = abs(cr(b - a, p - a)) / (b - a).len
fn sgn(x: F64) -> Int = if x > 0.0: 1 elif x < 0.0: -1 else: 0
var bad = ""
for k in 0..n:
    if bad != "": break
    let op = rng.int(0, 4)
    let (a, b, c, d) = (rv(mut rng, 5), rv(mut rng, 5), rv(mut rng, 5), rv(mut rng, 5))
    if op == 0:
        let t = F64(rng.int(-4, 4)) / 2.0
        if a + b != Vec(a.x + b.x, a.y + b.y) or a - b != Vec(a.x - b.x, a.y - b.y) or -a != Vec(-a.x, -a.y) or a * t != Vec(a.x * t, a.y * t):
            bad = "op {k}: arithmetic on {a}, {b}"
        elif t != 0.0 and not vn(a / t, Vec(a.x / t, a.y / t)): bad = "op {k}: {a} / {t}"
        elif a.norm2 != dt(a, a) or not nr(a.len, sqrt(dt(a, a))) or a.perp != Vec(-a.y, a.x) or a.perp_cw != Vec(a.y, -a.x):
            bad = "op {k}: norm or perp of {a}"
        elif a.angle != atan2(a.y, a.x) or not vn(Vec.from_angle(t), Vec(cos(t), sin(t))):
            bad = "op {k}: angle of {a}"
        elif a.dot(b) != dt(a, b) or a.cross(b) != cr(a, b) or a.dist2(b) != dt(a - b, a - b) or not nr(a.dist(b), sqrt(dt(a - b, a - b))):
            bad = "op {k}: dot, cross or dist of {a}, {b}"
        elif not vn(a.rot(b), Vec(a.x * b.x - a.y * b.y, a.x * b.y + a.y * b.x)): bad = "op {k}: rot {a} by {b}"
        elif a.len > 0.0 and not (nr(a.unit.len, 1.0) and nr(cr(a.unit, a), 0.0) and dt(a.unit, a) > 0.0): bad = "op {k}: unit of {a}"
        if bad != "": break
        let e = F64(rng.int(0, 3))
        let x = F64(rng.int(-5, 5))
        if close(x, t, eps: e) != (abs(x - t) <= e) or close(x, x + 1e-12) != true: bad = "op {k}: close({x}, {t}, {e})"
        elif sign_eps(x, eps: e) != (if x > e: 1 elif x < -e: -1 else: 0): bad = "op {k}: sign_eps({x}, {e})"
        elif turn_eps(a, b, c, eps: e) != (if cr(b - a, c - a) > e: 1 elif cr(b - a, c - a) < -e: -1 else: 0): bad = "op {k}: turn_eps"
        if bad != "": break
        if a.len > 0.0 and b.len > 0.0:
            let want = acos(max(-1.0, min(1.0, dt(a, b) / (a.len * b.len))))
            let got = angle_between(a, b)
            if not nr(abs(got), want) or (cr(a, b) != 0.0 and sgn(got) != sgn(cr(a, b))): bad = "op {k}: angle_between({a}, {b})"
        let h = HalfPlane.through(a, b + Vec(0.5, 0.0))
        let q = c
        let dir = b + Vec(0.5, 0.0) - a
        if not nr(h.side(q), cr(dir, q - a)): bad = "op {k}: HalfPlane.side"
        let (la, lb, lc) = (F64(rng.int(-3, 3)), F64(rng.int(1, 3)), F64(rng.int(-3, 3)))
        if not nr(HalfPlane.from_line(la, lb, lc).side(q), la * q.x + lb * q.y + lc): bad = "op {k}: from_line({la}, {lb}, {lc})"
        if not nr(HalfPlane(a, dir).side(q), cr(dir, q - a)): bad = "op {k}: HalfPlane"
    elif op == 1:
        // Lines and segments, with `b != a` for the line functions.
        let bb = if b == a: a + Vec(1.0, 0.0) else: b
        let pj = project(c, a, bb)
        let rf = reflect(c, a, bb)
        if not online(pj, a, bb) or not nr(dt(c - pj, bb - a), 0.0): bad = "op {k}: project({c}, {a}, {bb})"
        elif not online((c + rf) * 0.5, a, bb) or not nr(dt(rf - c, bb - a), 0.0): bad = "op {k}: reflect({c}, {a}, {bb})"
        elif not nr(line_dist(c, a, bb), ldist(c, a, bb)): bad = "op {k}: line_dist"
        if bad != "": break
        let len2 = dt(b - a, b - a)
        let t = if len2 == 0.0: 0.0 else: max(0.0, min(1.0, dt(c - a, b - a) / len2))
        if not nr(seg_dist(c, a, b), (a + (b - a) * t).dist(c)): bad = "op {k}: seg_dist({c}, {a}, {b})"
        let den = cr(b - a, d - c)
        let li = line_intersect(a, b, c, d)
        if (li == None) != (den == 0.0): bad = "op {k}: line_intersect None"
        elif li != None and not (online(li!, a, b) and online(li!, c, d)): bad = "op {k}: line_intersect point"
        if bad != "": break
        // Common points of the closed segments: the rational solve, or the shared endpoints.
        var want: List<Vec> = []
        if den != 0.0:
            var (tn, un, dd) = (cr(c - a, d - c), cr(c - a, b - a), den)
            if dd < 0.0: (tn, un, dd) = (-tn, -un, -dd)
            if 0.0 <= tn <= dd and 0.0 <= un <= dd: want.push(a + (b - a) * (tn / dd))
        else:
            var on: List<Vec> = []
            for p in [a, b, c, d]:
                if onseg(p, a, b) and onseg(p, c, d): on.push(p)
            sort(mut on, by: p => (p.x, p.y))
            if on.len > 0:
                want.push(on[0])
                if on[on.len - 1] != on[0]: want.push(on[on.len - 1])
        let si = segment_intersect(a, b, c, d)
        if si.len != want.len or (0..si.len).any(i => not vn(si[i], want[i])): bad = "op {k}: segment_intersect({a}, {b}, {c}, {d}) = {si} want {want}"
        if bad != "": break
        // The ray from `c` in direction `dir` against the segment `a b`.
        let dir = if d == Vec(0.0, 0.0): Vec(1.0, 2.0) else: d
        var hit: ?Vec = None
        let rd = cr(dir, b - a)
        if rd != 0.0:
            var (sn, tn, dd) = (cr(a - c, b - a), cr(a - c, dir), rd)
            if dd < 0.0: (sn, tn, dd) = (-sn, -tn, -dd)
            if sn >= 0.0 and 0.0 <= tn <= dd: hit = c + dir * (sn / dd)
        elif cr(a - c, dir) == 0.0:
            if onseg(c, a, b): hit = c
            else:
                for e in [a, b]:
                    if dt(e - c, dir) >= 0.0 and (hit == None or dt(e - c, dir) < dt(hit! - c, dir)): hit = e
        let rh = ray_hits_segment(c, dir, a, b)
        if (rh == None) != (hit == None) or (rh != None and not vn(rh!, hit!)): bad = "op {k}: ray_hits_segment({c}, {dir}, {a}, {b})"
    elif op == 2:
        let col = cr(b - a, c - a) == 0.0
        let cc = circumcenter(a, b, c)
        let ct = Circle.through(a, b, c)
        if (cc == None) != col or (ct == None) != col: bad = "op {k}: circumcenter None for {a}, {b}, {c}"
        elif cc != None:
            let o = cc!
            if not (nr(o.dist(a), o.dist(b)) and nr(o.dist(a), o.dist(c))): bad = "op {k}: circumcenter({a}, {b}, {c}) = {o}"
            elif not (vn(ct!.c, o) and nr(ct!.r, o.dist(a))): bad = "op {k}: Circle.through"
        if bad != "": break
        let ic = incenter(a, b, c)
        if (ic == None) != col: bad = "op {k}: incenter None"
        elif ic != None:
            let (o, r) = (ic!.c, ic!.r)
            let s = sgn(cr(b - a, c - a))
            let inside = sgn(cr(b - a, o - a)) == s and sgn(cr(c - b, o - b)) == s and sgn(cr(a - c, o - c)) == s
            if not (inside and nr(ldist(o, a, b), r) and nr(ldist(o, b, c), r) and nr(ldist(o, c, a), r)): bad = "op {k}: incenter({a}, {b}, {c})"
        if bad != "": break
        let r = F64(rng.int(0, 4))
        let ci = Circle(a, r)
        let dd = a.dist(b)
        let want = if abs(dd - r) <= 1e-9: Location.Boundary elif dd < r: Location.Inside else: Location.Outside
        if ci.locate(b) != want: bad = "op {k}: {a} {r} locate {b}"
        elif not nr(ci.area, atan2(0.0, -1.0) * r * r) or not nr(ci.circumference, 2.0 * atan2(0.0, -1.0) * r): bad = "op {k}: area of radius {r}"
        if bad != "": break
        if b != a and r > 0.0:
            let iv = invert(b, ci)
            if not (nr(cr(iv - a, b - a), 0.0) and dt(iv - a, b - a) > 0.0 and nr(iv.dist(a) * b.dist(a), r * r)): bad = "op {k}: invert({b}, {a} {r})"
    else:
        // Circle meets, decided exactly on integer centres and radii.
        let (ra, rb) = (F64(rng.int(0, 4)), F64(rng.int(0, 4)))
        let (ca, cb) = (Circle(a, ra), Circle(b, rb))
        let on = (p: Vec, o: Circle) => nr(p.dist(o.c), o.r)
        if c != d and op == 3:
            let cx = cr(d - c, a - c)
            let l2 = dt(d - c, d - c)
            let want = if cx * cx < ra * ra * l2: 2 elif cx * cx == ra * ra * l2: 1 else: 0
            let got = circle_line_intersect(ca, c, d)
            if got.len != want or got.any(p => not (on(p, ca) and online(p, c, d))): bad = "op {k}: circle_line_intersect({a} {ra}, {c}, {d}) = {got}"
            elif got.len == 2 and dt(got[1] - got[0], d - c) <= 0.0: bad = "op {k}: circle_line_intersect order"
        if bad != "": break
        let d2 = dt(b - a, b - a)
        let (lo, hi) = ((ra - rb) * (ra - rb), (ra + rb) * (ra + rb))
        let want = if d2 == 0.0: 0 elif lo < d2 < hi: 2 elif d2 == lo or d2 == hi: 1 else: 0
        let got = circle_intersect(ca, cb)
        if got.len != want or got.any(p => not (on(p, ca) and on(p, cb))): bad = "op {k}: circle_intersect({a} {ra}, {b} {rb}) = {got}"
        elif got.len == 2 and cr(b - a, got[0] - a) >= 0.0: bad = "op {k}: circle_intersect order"
        if bad != "": break
        let p2 = dt(c - a, c - a)
        let wt = if p2 > ra * ra: 2 elif p2 == ra * ra: 1 else: 0
        let tg = circle_tangents(ca, c)
        if tg.len != wt or tg.any(q => not (on(q, ca) and nr(dt(q - a, c - q), 0.0))): bad = "op {k}: circle_tangents({a} {ra}, {c}) = {tg}"
        elif wt == 1 and not vn(tg[0], c): bad = "op {k}: circle_tangents on the circle"
        elif wt == 2 and ra > 0.0 and cr(c - a, tg[0] - a) >= 0.0: bad = "op {k}: circle_tangents order"
        if bad != "": break
        for inner in [false, true]:
            let lim = if inner: hi else: lo
            let wc = if d2 == 0.0: 0 elif d2 > lim: 2 elif d2 == lim: 1 else: 0
            let ts = circle_circle_tangents(ca, cb, inner: inner)
            if ts.len != wc: bad = "op {k}: circle_circle_tangents({a} {ra}, {b} {rb}, inner: {yes(inner)}) = {ts}"
            for (p1, p2) in ts:
                if bad == "" and not (on(p1, ca) and on(p2, cb)): bad = "op {k}: circle_circle_tangents point off its circle"
                elif bad == "" and p1.dist(p2) > 1e-6 and not (nr(ldist(a, p1, p2), ra) and nr(ldist(b, p1, p2), rb)): bad = "op {k}: circle_circle_tangents not tangent"
print(if bad == "": "ok" else: bad)
