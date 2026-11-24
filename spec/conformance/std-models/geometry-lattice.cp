// expect: ok, output "ok"
// input: "1 300\n"
// model: Pt
// model: Rect
// model: PointSet
// model: dot
// model: cross
// model: turn
// model: same_dir
// model: angle_less
// model: arg
// model: on_segment
// model: segments_meet
// model: area2
// model: area2_128
// model: locate
// model: pick_interior
// model: pick_boundary
// model: closest_pair2
// model: closest_pair
// model: farthest_pair
// model: manhattan_mst
// model: any_segments_meet
// model: any_segments_meet_idx
// model: rects_union_area
// Exact lattice geometry against brute force on small coordinates: Pt arithmetic against the
// formulas, angle order against atan2, segment tests against a rational parameter solve, locate
// and Pick counts against ray casting over every lattice point, pair searches and the Manhattan
// MST candidates against all-pairs loops and Prim, Rect and the union area against unit cells, and
// PointSet against a list of live flags. Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.geometry.{Location, PointSet, Pt, Rect, angle_less, any_segments_meet, any_segments_meet_idx, arg, area2, area2_128, closest_pair, closest_pair2, cross, dot, farthest_pair, locate, manhattan_mst, on_segment, pick_boundary, pick_interior, rects_union_area, same_dir, segments_meet, turn}
let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
let tau = 2.0 * atan2(0.0, -1.0)
fn rp(mut rng: Rng, k: Int) -> Pt = Pt(rng.int(-k, k), rng.int(-k, k))
fn cr(a: Pt, b: Pt) -> Int = a.x * b.y - a.y * b.x
fn dt(a: Pt, b: Pt) -> Int = a.x * b.x + a.y * b.y
fn sgn(x: Int) -> Int = if x > 0: 1 elif x < 0: -1 else: 0
// The counter-clockwise angle of `p` from `f` in [0, tau), -1 for the zero vector.
fn ang(p: Pt, f: Pt) -> F64:
    if p.x == 0 and p.y == 0: return -1.0
    let t = atan2(F64(cr(f, p)), F64(dt(f, p)))
    if t < -1e-12: t + tau else: t
fn onseg(p: Pt, a: Pt, b: Pt) -> Bool = cr(p - a, b - a) == 0 and dt(p - a, p - b) <= 0
fn meet(a: Pt, b: Pt, c: Pt, d: Pt, strict: Bool) -> Bool:
    var den = cr(b - a, d - c)
    var tn = cr(c - a, d - c)
    var un = cr(c - a, b - a)
    if den == 0:
        if strict: return false
        return onseg(a, c, d) or onseg(b, c, d) or onseg(c, a, b) or onseg(d, a, b)
    if den < 0:
        den = -den
        tn = -tn
        un = -un
    if strict: 0 < tn < den and 0 < un < den else: 0 <= tn <= den and 0 <= un <= den
fn where(poly: List<Pt>, p: Pt) -> Location:
    for i in 0..poly.len:
        if onseg(p, poly[i], poly[(i + 1) % poly.len]): return Location.Boundary
    let q = Pt(p.x + 2000, p.y + 1)
    var hits = 0
    for i in 0..poly.len:
        if meet(p, q, poly[i], poly[(i + 1) % poly.len], false): hits += 1
    if hits % 2 == 1: Location.Inside else: Location.Outside
fn md(a: Pt, b: Pt) -> Int = abs(a.x - b.x) + abs(a.y - b.y)
var ps_pts = [rp(mut rng, 6) for _ in 0..30]
var ps = PointSet.of(ps_pts)
var alive = List.filled(30, true)
var bad = ""
for k in 0..n:
    if bad != "": break
    let op = rng.int(0, 6)
    if op == 0:
        let (a, b, c) = (rp(mut rng, 9), rp(mut rng, 9), rp(mut rng, 9))
        let m = rng.int(-3, 3)
        let s = a + b
        let dd = a - b
        if s != Pt(a.x + b.x, a.y + b.y) or dd != Pt(a.x - b.x, a.y - b.y) or -a != Pt(-a.x, -a.y) or a * m != Pt(a.x * m, a.y * m):
            bad = "op {k}: arithmetic on {a}, {b}"
        elif a.norm2 != dt(a, a) or a.norm2_128 != I128(dt(a, a)) or abs(a.len - sqrt(F64(dt(a, a)))) > 1e-9:
            bad = "op {k}: norm of {a}"
        elif a.perp != Pt(-a.y, a.x) or a.perp_cw != Pt(a.y, -a.x) or a.angle != atan2(F64(a.y), F64(a.x)):
            bad = "op {k}: perp or angle of {a}"
        elif a.to_vec.x != F64(a.x) or a.to_vec.y != F64(a.y) or "{a}" != "{a.x} {a.y}":
            bad = "op {k}: to_vec or show of {a}"
        elif dot(a, b) != dt(a, b) or cross(a, b) != cr(a, b) or a.dot128(b) != I128(dt(a, b)) or a.cross128(b) != I128(cr(a, b)):
            bad = "op {k}: dot or cross of {a}, {b}"
        elif a.dist2(b) != dt(dd, dd) or a.dist2_128(b) != I128(dt(dd, dd)) or abs(a.dist(b) - sqrt(F64(dt(dd, dd)))) > 1e-9:
            bad = "op {k}: dist of {a}, {b}"
        elif a.manhattan(b) != md(a, b) or a.chebyshev(b) != max(abs(dd.x), abs(dd.y)):
            bad = "op {k}: manhattan or chebyshev of {a}, {b}"
        elif turn(a, b, c) != sgn(cr(b - a, c - a)) or same_dir(a, b) != (cr(a, b) == 0 and dt(a, b) > 0):
            bad = "op {k}: turn or same_dir of {a}, {b}, {c}"
        if bad != "": break
        var g = 1
        for t in 1..=max(abs(a.x), abs(a.y)):
            if a.x % t == 0 and a.y % t == 0: g = t
        if a.unit_dir() != (if a == Pt(0, 0): a else: Pt(a.x / g, a.y / g)): bad = "op {k}: unit_dir of {a}"
        let f = if c == Pt(0, 0): Pt(1, 0) else: c
        let (aa, ab) = (ang(a, f), ang(b, f))
        if angle_less(a, b, from: f) != (aa < ab - 1e-9) or angle_less(a, b) != (ang(a, Pt(1, 0)) < ang(b, Pt(1, 0)) - 1e-9):
            bad = "op {k}: angle_less({a}, {b}, from: {f})"
        elif (arg(a) == arg(b)) != (abs(ang(a, Pt(1, 0)) - ang(b, Pt(1, 0))) < 1e-9) or (arg(a) < arg(b)) != angle_less(a, b):
            bad = "op {k}: arg of {a}, {b}"
        if bad != "": break
        var xs = [rp(mut rng, 4) for _ in 0..rng.int(0, 8)]
        let before = xs.len
        sort(mut xs, by: arg)
        for i in 1..xs.len:
            if ang(xs[i], Pt(1, 0)) < ang(xs[i - 1], Pt(1, 0)) - 1e-9: bad = "op {k}: sort by arg gives {xs}"
        if xs.len != before: bad = "op {k}: sort by arg changed the length"
    elif op == 1:
        let (a, b, c, d) = (rp(mut rng, 3), rp(mut rng, 3), rp(mut rng, 3), rp(mut rng, 3))
        let strict = rng.int(0, 1) == 1
        let want_on = onseg(c, a, b) and (not strict or (c != a and c != b))
        if on_segment(c, a, b, strict: strict) != want_on: bad = "op {k}: on_segment({c}, {a}, {b}, strict: {yes(strict)})"
        elif segments_meet(a, b, c, d, strict: strict) != meet(a, b, c, d, strict):
            bad = "op {k}: segments_meet({a}, {b}, {c}, {d}, strict: {yes(strict)})"
    elif op == 2:
        // A random star-shaped polygon around the origin, always simple.
        var poly = [Pt(6, 0), Pt(0, 6), Pt(-6, 0), Pt(0, -6)]
        for _ in 0..rng.int(0, 6):
            let p = rp(mut rng, 6)
            if p != Pt(0, 0) and not poly.any(q => same_dir(p, q)): poly.push(p)
        sort(mut poly, by: arg)
        if rng.int(0, 1) == 1: poly.reverse()
        var a2 = 0
        for i in 0..poly.len: a2 += cr(poly[i], poly[(i + 1) % poly.len])
        if area2(poly) != a2 or area2_128(poly) != I128(a2): bad = "op {k}: area2 of {poly}"
        var (inner, border) = (0, 0)
        for x in -6..=6:
            for y in -6..=6:
                let w = where(poly, Pt(x, y))
                if w == Location.Inside: inner += 1
                if w == Location.Boundary: border += 1
                if rng.int(0, 9) == 0 and locate(poly, Pt(x, y)) != w: bad = "op {k}: locate({poly}, {x} {y})"
        if pick_interior(poly) != inner or pick_boundary(poly) != border: bad = "op {k}: pick counts of {poly}"
    elif op == 3:
        let pts = [rp(mut rng, 8) for _ in 0..rng.int(2, 9)]
        var (ci, cj, cd) = (0, 1, pts[0].dist2(pts[1]))
        var (fi, fj, fd) = (0, 1, cd)
        for i in 0..pts.len:
            for j in i + 1..pts.len:
                let d = dt(pts[i] - pts[j], pts[i] - pts[j])
                if d < cd: (ci, cj, cd) = (i, j, d)
                if d > fd: (fi, fj, fd) = (i, j, d)
        if closest_pair2(pts) != cd or closest_pair(pts) != (ci, cj, cd): bad = "op {k}: closest pair of {pts}"
        elif farthest_pair(pts) != (fi, fj, fd): bad = "op {k}: farthest pair of {pts}"
        if bad != "": break
        // Prim's algorithm for the weight; the edges must join everything and be sorted.
        var inside = List.filled(pts.len, false)
        var best = List.filled(pts.len, 1 << 40)
        best[0] = 0
        var total = 0
        for _ in 0..pts.len:
            var u = -1
            for v in 0..pts.len:
                if not inside[v] and (u == -1 or best[v] < best[u]): u = v
            inside[u] = true
            total += best[u]
            for v in 0..pts.len: best[v] = min(best[v], md(pts[u], pts[v]))
        let es = manhattan_mst(pts)
        var comp = [i for i in 0..pts.len]
        var got = 0
        // Kruskal over the candidate edges, which come sorted by weight.
        for e in es:
            if e.w != md(pts[e.u], pts[e.v]) or e.u >= e.v: bad = "op {k}: mst candidate {e.u} {e.v} {e.w}"
            let (cu, cv) = (comp[e.u], comp[e.v])
            if cu != cv:
                got += e.w
                for i in 0..pts.len:
                    if comp[i] == cv: comp[i] = cu
        if es.len > 4 * pts.len or got != total or comp.any(c => c != comp[0]): bad = "op {k}: manhattan_mst of {pts}"
        for i in 1..es.len:
            if (es[i - 1].w, es[i - 1].u, es[i - 1].v) > (es[i].w, es[i].u, es[i].v): bad = "op {k}: mst order"
    elif op == 4:
        var segs: List<(a: Pt, b: Pt)> = []
        for _ in 0..rng.int(0, 6): segs.push((rp(mut rng, 6), rp(mut rng, 6)))
        let strict = rng.int(0, 1) == 1
        var any = false
        for i in 0..segs.len:
            for j in i + 1..segs.len:
                if meet(segs[i].a, segs[i].b, segs[j].a, segs[j].b, strict): any = true
        for got in [any_segments_meet(segs, strict: strict), any_segments_meet_idx(segs, strict: strict)]:
            if (got != None) != any: bad = "op {k}: any_segments_meet({segs}, strict: {yes(strict)})"
            elif got != None:
                let (i, j) = got!
                if not (i < j and meet(segs[i].a, segs[i].b, segs[j].a, segs[j].b, strict)): bad = "op {k}: any_segments_meet pair {i} {j}"
    elif op == 5:
        var rects: List<Rect> = []
        for _ in 0..rng.int(0, 5):
            let (x0, y0) = (rng.int(-6, 6), rng.int(-6, 6))
            rects.push(Rect(x0..x0 + rng.int(-1, 5), y0..y0 + rng.int(-1, 5)))
        var cells = 0
        for x in -6..12:
            for y in -6..12:
                if rects.any(r => r.x.start <= x < r.x.end and r.y.start <= y < r.y.end): cells += 1
        if rects_union_area(rects) != cells: bad = "op {k}: rects_union_area({rects}) want {cells}"
        for i in 0..rects.len:
            let r = rects[i]
            let o = rects[(i + 1) % rects.len]
            let w = max(r.x.end - r.x.start, 0)
            let h = max(r.y.end - r.y.start, 0)
            let p = rp(mut rng, 8)
            let lo = Rect(max(r.x.start, o.x.start)..min(r.x.end, o.x.end), max(r.y.start, o.y.start)..min(r.y.end, o.y.end))
            let meets = lo.x.start < lo.x.end and lo.y.start < lo.y.end
            if r.area != w * h or r.is_empty != (w * h == 0): bad = "op {k}: area of {r}"
            elif r.contains(p) != (r.x.start <= p.x < r.x.end and r.y.start <= p.y < r.y.end): bad = "op {k}: {r} contains {p}"
            elif r.intersect(o) != (if meets: Some(lo) else: None): bad = "op {k}: {r} intersect {o}"
    else:
        if ps.len == 0:
            ps_pts = [rp(mut rng, 6) for _ in 0..30]
            ps = PointSet.of(ps_pts)
            alive = List.filled(30, true)
        let i = rng.int(0, 29)
        if rng.int(0, 1) == 0:
            if ps.remove(i) != alive[i]: bad = "op {k}: remove({i})"
            alive[i] = false
        else:
            let (x0, y0) = (rng.int(-7, 6), rng.int(-7, 6))
            let (xr, yr) = (x0..x0 + rng.int(0, 7), y0..y0 + rng.int(0, 7))
            var want: List<Int> = []
            for j in 0..30:
                if alive[j] and xr.start <= ps_pts[j].x < xr.end and yr.start <= ps_pts[j].y < yr.end:
                    want.push(j)
                    alive[j] = false
            if ps.drain(xr, yr).sorted() != want: bad = "op {k}: drain({xr}, {yr})"
        if ps.len != alive.count(a => a) or ps.contains(i) != alive[i]: bad = "op {k}: len or contains({i})"
print(if bad == "": "ok" else: bad)
