// expect: ok, output "ok"
// input: "1 300\n"
// model: polygon_area
// model: polygon_centroid
// model: polygon_cut
// model: polygon_kernel
// model: convex_polygon_intersect
// model: half_plane_intersect
// model: min_enclosing_circle
// model: circle_polygon_area
// model: circle_circle_area
// model: circles_union_area
// Areas against slice integrals: half-plane regions exactly, piece by piece between the x of
// line crossings, and disk regions over 800 vertical lines of chords and polygon crossings.
// Centroids against a triangle fan and the enclosing circle against every pair and triple.
// Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.geometry.{Circle, HalfPlane, Vec, circle_circle_area, circle_polygon_area, circles_union_area, convex_polygon_intersect, half_plane_intersect, min_enclosing_circle, polygon_area, polygon_centroid, polygon_cut, polygon_kernel}
let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
const M = 800
const B = 20.0
fn cr(a: Vec, b: Vec) -> F64 = a.x * b.y - a.y * b.x
fn shoe(poly: List<Vec>) -> F64:
    var s = 0.0
    for i in 0..poly.len: s += cr(poly[i], poly[(i + 1) % poly.len])
    s / 2.0
// The slice integral of `len(x)` over `[-w, w]`.
fn integral(len: F64 -> F64, w: F64 = B) -> F64:
    let h = 2.0 * w / F64(M)
    var s = 0.0
    for i in 0..M: s += len(-w + (F64(i) + 0.5) * h) * h
    s
// The interval of the line `x` inside every closed half-plane `(p, dir)` and the box.
fn hp_iv(cons: List<(Vec, Vec)>, x: F64) -> (F64, F64):
    var (lo, hi) = (-B, B)
    for (p, d) in cons:
        let y = p.y + d.y * (x - p.x) / d.x
        if d.x > 0.0: lo = max(lo, y)
        elif d.x < 0.0: hi = min(hi, y)
        elif -d.y * (x - p.x) < 0.0: hi = lo
    (lo, max(lo, hi))
fn hp_len(cons: List<(Vec, Vec)>, x: F64) -> F64:
    let (lo, hi) = hp_iv(cons, x)
    hi - lo
// The exact area of the region: its slice length is linear between the x of any two lines'
// crossing, a line meeting the box, or a vertical line, so one midpoint per piece is exact.
fn hp_area(cons: List<(Vec, Vec)>) -> F64:
    var xs = [-B, B]
    for (p, d) in cons:
        if d.x == 0.0: xs.push(p.x)
        elif d.y != 0.0:
            for yb in [-B, B]: xs.push(p.x + d.x * (yb - p.y) / d.y)
    for i in 0..cons.len:
        for j in i + 1..cons.len:
            let (p, d) = cons[i]
            let (q, e) = cons[j]
            let den = cr(d, e)
            if den != 0.0: xs.push(p.x + d.x * cr(q - p, e) / den)
    var ok = [x for x in xs if -B <= x <= B]
    ok.sort()
    var s = 0.0
    for i in 1..ok.len: s += hp_len(cons, (ok[i - 1] + ok[i]) / 2.0) * (ok[i] - ok[i - 1])
    s
fn edges(poly: List<Vec>) -> List<(Vec, Vec)> = [(poly[i], poly[(i + 1) % poly.len] - poly[i]) for i in 0..poly.len]
fn chord(c: Circle, x: F64) -> (F64, F64):
    let w = c.r * c.r - (x - c.c.x) * (x - c.c.x)
    if w <= 0.0: (0.0, 0.0) else: (c.c.y - sqrt(w), c.c.y + sqrt(w))
fn poly_ivs(poly: List<Vec>, x: F64) -> List<(F64, F64)>:
    var ys: List<F64> = []
    for i in 0..poly.len:
        let (a, b) = (poly[i], poly[(i + 1) % poly.len])
        if (a.x <= x < b.x) or (b.x <= x < a.x): ys.push(a.y + (b.y - a.y) * (x - a.x) / (b.x - a.x))
    ys.sort()
    [(ys[2 * i], ys[2 * i + 1]) for i in 0..ys.len / 2]
fn over(a: (F64, F64), b: (F64, F64)) -> F64 = max(0.0, min(a.1, b.1) - max(a.0, b.0))
fn star(mut rng: Rng) -> List<Vec>:
    var poly = [Vec(5.0, 0.0), Vec(0.0, 5.0), Vec(-5.0, 0.0), Vec(0.0, -5.0)]
    for _ in 0..rng.int(0, 6):
        let p = Vec(F64(rng.int(-5, 5)), F64(rng.int(-5, 5)))
        if p != Vec(0.0, 0.0) and not poly.any(q => cr(p, q) == 0.0 and p.x * q.x + p.y * q.y > 0.0): poly.push(p)
    sort(mut poly, by: p => p.angle)
    poly
fn convex(mut rng: Rng) -> List<Vec>:
    let (cx, cy, r) = (F64(rng.int(-4, 4)), F64(rng.int(-4, 4)), F64(rng.int(1, 6)))
    var deg = [rng.int(0, 359) for _ in 0..rng.int(3, 7)]
    deg.sort()
    deg.dedup()
    while deg.len < 3: deg = [0, 120, 240]
    [Vec(cx + r * cos(F64(t) * 0.017453292519943295), cy + r * sin(F64(t) * 0.017453292519943295)) for t in deg]
fn near(a: F64, b: F64, tol: F64) -> Bool = abs(a - b) <= tol
var bad = ""
for k in 0..n:
    if bad != "": break
    let op = rng.int(0, 3)
    if op == 0:
        var p = star(mut rng)
        let ccw = rng.int(0, 1) == 0
        if not ccw: p.reverse()
        let a = shoe(p)
        var (sx, sy) = (0.0, 0.0)
        for i in 1..p.len - 1:
            let t = cr(p[i] - p[0], p[i + 1] - p[0]) / 2.0
            sx += (p[0].x + p[i].x + p[i + 1].x) / 3.0 * t
            sy += (p[0].y + p[i].y + p[i + 1].y) / 3.0 * t
        let g = polygon_centroid(p)
        if polygon_area(p) != a: bad = "op {k}: polygon_area({p})"
        elif not (near(g.x, sx / a, 1e-9) and near(g.y, sy / a, 1e-9)): bad = "op {k}: polygon_centroid({p})"
        if bad != "" or not ccw: continue
        let want = hp_area(edges(p))
        let ker = polygon_kernel(p)
        if not near(shoe(ker), want, 1e-2) or (want > 1e-2 and ker.len < 3): bad = "op {k}: polygon_kernel({p}) area {shoe(ker)} want {want}"
    elif op == 1:
        let (pa, pb) = (convex(mut rng), convex(mut rng))
        let both = edges(pa) + edges(pb)
        let want = hp_area(both)
        let got = convex_polygon_intersect(pa, pb)
        if not near(shoe(got), want, 1e-2): bad = "op {k}: convex_polygon_intersect area {shoe(got)} want {want}"
        if bad != "": break
        let a = Vec(F64(rng.int(-5, 5)), F64(rng.int(-5, 5)))
        let b = a + Vec(F64(rng.int(1, 3)), F64(rng.int(-3, 3)))
        let cut = polygon_cut(pa, a, b)
        let wc = hp_area(edges(pa) + [(a, b - a)])
        if not near(abs(shoe(cut)), wc, 1e-2) or cut.any(v => cr(b - a, v - a) < -1e-6): bad = "op {k}: polygon_cut area {shoe(cut)} want {wc}"
        if bad != "": break
        var hs = [HalfPlane(p, d) for (p, d) in edges(pa)]
        var cons = edges(pa)
        if rng.int(0, 1) == 0:
            hs = []
            cons = []
            for _ in 0..rng.int(1, 5):
                let p = Vec(F64(rng.int(-5, 5)), F64(rng.int(-5, 5)))
                let d = Vec(F64(rng.int(1, 3)) * F64(2 * rng.int(0, 1) - 1), F64(rng.int(-3, 3)))
                hs.push(HalfPlane(p, d))
                cons.push((p, d))
        let wh = hp_area(cons)
        let hp = half_plane_intersect(hs, bound: Some(B))
        if not near(shoe(hp), wh, 1e-2) or (wh > 1e-2 and hp.len < 3): bad = "op {k}: half_plane_intersect area {shoe(hp)} want {wh}"
        elif not near(shoe(half_plane_intersect([HalfPlane(p, d) for (p, d) in edges(pa)])), shoe(pa), 1e-6): bad = "op {k}: half_plane_intersect of a polygon's edges"
    elif op == 2:
        let cs = [Circle(Vec(F64(rng.int(-4, 4)), F64(rng.int(-4, 4))), F64(rng.int(0, 4))) for _ in 0..rng.int(0, 4)]
        let wu = integral(x =>
            var ivs = [chord(c, x) for c in cs]
            ivs.sort()
            var (tot, end) = (0.0, -B)
            for (lo, hi) in ivs:
                if hi > end:
                    tot += hi - max(lo, end)
                    end = hi
            tot
        , w: 8.0)
        if not near(circles_union_area(cs), wu, 5e-2): bad = "op {k}: circles_union_area({cs}) = {circles_union_area(cs)} want {wu}"
        elif cs.len >= 2:
            let wi = integral(x => over(chord(cs[0], x), chord(cs[1], x)), w: 8.0)
            if not near(circle_circle_area(cs[0], cs[1]), wi, 5e-2): bad = "op {k}: circle_circle_area"
        if bad != "" or cs.len == 0: continue
        var p = star(mut rng)
        let sg = if rng.int(0, 1) == 0: 1.0 else: -1.0
        if sg < 0.0: p.reverse()
        let wp = integral(x =>
            var s = 0.0
            for iv in poly_ivs(p, x): s += over(iv, chord(cs[0], x))
            s
        , w: 8.0)
        if not near(circle_polygon_area(cs[0], p), sg * wp, 5e-2): bad = "op {k}: circle_polygon_area({cs[0]}, {p}) want {sg * wp}"
    else:
        let pts = [Vec(F64(rng.int(-5, 5)), F64(rng.int(-5, 5))) for _ in 0..rng.int(0, 7)]
        let mec = min_enclosing_circle(pts)
        var best = if pts.len <= 1: 0.0 else: 1e18
        let fits = (o: Vec, r: F64) => pts.all(q => q.dist(o) <= r + 1e-7)
        for i in 0..pts.len:
            for j in i + 1..pts.len:
                let o = (pts[i] + pts[j]) * 0.5
                if fits(o, o.dist(pts[i])): best = min(best, o.dist(pts[i]))
                for l in j + 1..pts.len:
                    let (a, b, c) = (pts[i], pts[j], pts[l])
                    let dd = 2.0 * cr(b - a, c - a)
                    if dd != 0.0:
                        let (b2, c2) = ((b - a).norm2, (c - a).norm2)
                        let o2 = a + Vec(((c - a).y * b2 - (b - a).y * c2) / dd, ((b - a).x * c2 - (c - a).x * b2) / dd)
                        if fits(o2, o2.dist(a)): best = min(best, o2.dist(a))
        if pts.len == 0 and mec != Circle(Vec(0.0, 0.0), 0.0): bad = "op {k}: min_enclosing_circle([])"
        elif not (fits(mec.c, mec.r + 1e-6) and near(mec.r, best, 1e-6)): bad = "op {k}: min_enclosing_circle({pts}) = {mec} want radius {best}"
print(if bad == "": "ok" else: bad)
