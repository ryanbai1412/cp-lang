// expect: ok, output "ok"
// input: "1 300\n"
// model: OnlineHull
// model: convex_hull
// model: convex_hull_idx
// model: lower_hull
// model: upper_hull
// model: lower_hull_idx
// model: upper_hull_idx
// model: minkowski_sum
// model: minkowski_upper
// model: minkowski_lower
// model: convex_polygons_overlap
// model: locate_convex
// model: diameter2
// model: min_bounding_box
// model: hull_extreme
// model: hull_tangents
// model: hull_width
// model: hull_line_intersect
// model: polygon_triangulate
// Convex hulls of small lattice point sets against gift wrapping (Jarvis march), chains against
// walks along that hull, and every hull query against a loop over all vertices, edges or pairs.
// OnlineHull adds random points against the rebuilt wrap. Reads `seed n`; prints the first op
// that differs.
use std.rand.{Rng}
use std.geometry.{Location, OnlineHull, Pt, convex_hull, convex_hull_idx, convex_polygons_overlap, diameter2, hull_extreme, hull_line_intersect, hull_tangents, hull_width, locate_convex, lower_hull, lower_hull_idx, min_bounding_box, minkowski_lower, minkowski_sum, minkowski_upper, polygon_triangulate, upper_hull, upper_hull_idx}
let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
fn rp(mut rng: Rng, k: Int) -> Pt = Pt(rng.int(-k, k), rng.int(-k, k))
fn cr(a: Pt, b: Pt) -> Int = a.x * b.y - a.y * b.x
fn dt(a: Pt, b: Pt) -> Int = a.x * b.x + a.y * b.y
fn onseg(p: Pt, a: Pt, b: Pt) -> Bool = cr(p - a, b - a) == 0 and dt(p - a, p - b) <= 0
fn key(p: Pt) -> (Int, Int) = (p.x, p.y)
fn srt(xs: List<Pt>) -> List<Pt>:
    var s = [p for p in xs]
    sort(mut s, by: key)
    s
// Gift wrapping from the least (x, y): the next corner leaves no point strictly to the right,
// farthest of the candidates (nearest with `coll`). All points on one line give [min, max].
fn wrap(pts: List<Pt>, coll: Bool) -> List<Pt>:
    if pts.len == 0: return []
    var lo = pts[0]
    var hi = pts[0]
    for p in pts:
        if key(p) < key(lo): lo = p
        if key(p) > key(hi): hi = p
    if pts.all(p => cr(hi - lo, p - lo) == 0): return if lo == hi: [lo] else: [lo, hi]
    var out = [lo]
    var cur = lo
    while true:
        var best: ?Pt = None
        for q in pts:
            if q == cur or pts.any(r => cr(q - cur, r - cur) < 0): continue
            if best == None: best = q
            else:
                let d = dt(q - cur, q - cur)
                let bd = dt(best! - cur, best! - cur)
                if (coll and d < bd) or (not coll and d > bd): best = q
        cur = best!
        if cur == lo: break
        out.push(cur)
    out
// The walk along the counter-clockwise hull from `from` to `to`, both included.
fn chain(h: List<Pt>, from: Pt, to: Pt) -> List<Pt>:
    var i = 0
    while h[i] != from: i += 1
    var out = [h[i]]
    while h[i] != to:
        i = (i + 1) % h.len
        out.push(h[i])
    out
fn where(h: List<Pt>, p: Pt) -> Location:
    if h.len == 0: return Location.Outside
    if h.len == 1: return if p == h[0]: Location.Boundary else: Location.Outside
    var on = false
    for i in 0..h.len:
        let (a, b) = (h[i], h[(i + 1) % h.len])
        if onseg(p, a, b): on = true
        elif cr(b - a, p - a) < 0 or h.len == 2: return Location.Outside
    if on: Location.Boundary else: Location.Inside
fn first_idx(pts: List<Pt>, p: Pt) -> Int:
    var i = 0
    while pts[i] != p: i += 1
    i
var oh = OnlineHull()
var added: List<Pt> = []
var bad = ""
for k in 0..n:
    if bad != "": break
    let op = rng.int(0, 3)
    let pts = [rp(mut rng, 4) for _ in 0..rng.int(0, 8)]
    if op == 0:
        let coll = rng.int(0, 1) == 1
        let h = wrap(pts, coll)
        let flat = h.len <= 2
        if (not coll or not flat) and convex_hull(pts, collinear: coll) != h: bad = "op {k}: convex_hull({pts}, collinear: {yes(coll)}) want {h}"
        if bad != "" or pts.len == 0: continue
        let hi = convex_hull_idx(pts, collinear: coll)
        if (not coll or not flat) and ([pts[i] for i in hi] != h or hi.any(i => first_idx(pts, pts[i]) != i)): bad = "op {k}: convex_hull_idx"
        if bad != "": break
        var sp = srt(pts)
        let lo = sp[0]
        let top = sp[sp.len - 1]
        let ends_unique = sp.count(p => p.x == lo.x) == 1 and sp.count(p => p.x == top.x) == 1
        if coll and (flat or not ends_unique): continue
        let lw = if flat: h else: chain(h, lo, top)
        let up = if flat: h else: chain(h, top, lo).reversed()
        if lower_hull(sp, collinear: coll) != lw or upper_hull(sp, collinear: coll) != up: bad = "op {k}: lower/upper_hull({sp}, collinear: {yes(coll)}) want {lw} {up}"
        elif [sp[i] for i in lower_hull_idx(sp, collinear: coll)] != lw or [sp[i] for i in upper_hull_idx(sp, collinear: coll)] != up: bad = "op {k}: hull_idx chains"
        elif lower_hull_idx(sp, collinear: coll).any(i => first_idx(sp, sp[i]) != i): bad = "op {k}: lower_hull_idx keeps a later duplicate"
    elif op == 1:
        // Queries on a hull with at least three corners.
        let h = wrap(pts + [Pt(-5, -5), Pt(5, -4), Pt(0, 5)], false)
        let p = rp(mut rng, 7)
        let dir = rp(mut rng, 3)
        if locate_convex(h, p) != where(h, p): bad = "op {k}: locate_convex({h}, {p})"
        elif locate_convex([h[0]], p) != where([h[0]], p) or locate_convex([h[0], h[1]], p) != where([h[0], h[1]], p): bad = "op {k}: locate_convex on a short hull"
        if bad != "": break
        var (d2, ext) = (0, 0)
        for i in 0..h.len:
            if dt(h[i], dir) > dt(h[ext], dir): ext = i
            for j in 0..h.len: d2 = max(d2, dt(h[i] - h[j], h[i] - h[j]))
        if diameter2(h) != d2: bad = "op {k}: diameter2({h})"
        elif hull_extreme(h, dir) != ext: bad = "op {k}: hull_extreme({h}, {dir})"
        if bad != "": break
        var ba: F64 = 1e18
        var bp: F64 = 1e18
        var bw: F64 = 1e18
        for i in 0..h.len:
            let e = h[(i + 1) % h.len] - h[i]
            let l = sqrt(F64(dt(e, e)))
            var lo = 0.0
            var hi = 0.0
            var up = 0.0
            for q in h:
                let along = F64(dt(q - h[i], e)) / l
                lo = min(lo, along)
                hi = max(hi, along)
                up = max(up, F64(cr(e, q - h[i])) / l)
            let (ar, pe) = ((hi - lo) * up, 2.0 * (hi - lo + up))
            if ar < ba: ba = ar
            if pe < bp: bp = pe
            if up < bw: bw = up
        let bb = min_bounding_box(h)
        if abs(bb.area - ba) > 1e-6 or abs(bb.perimeter - bp) > 1e-6: bad = "op {k}: min_bounding_box({h})"
        elif abs(hull_width(h) - bw) > 1e-6: bad = "op {k}: hull_width({h})"
        if bad != "": break
        if where(h, p) == Location.Outside:
            let ok_l = [i for i in 0..h.len if h.all(q => cr(h[i] - p, q - p) <= 0)]
            let ok_r = [i for i in 0..h.len if h.all(q => cr(h[i] - p, q - p) >= 0)]
            let near = (is: List<Int>) =>
                var m = is[0]
                for i in is:
                    if dt(h[i] - p, h[i] - p) < dt(h[m] - p, h[m] - p): m = i
                m
            let t = hull_tangents(h, p)
            if t.left != near(ok_l) or t.right != near(ok_r): bad = "op {k}: hull_tangents({h}, {p}) = {t.left} {t.right}"
        if bad != "": break
        let b = if dir == Pt(0, 0): p + Pt(1, 1) else: p + dir
        let s = [cr(b - p, q - p) for q in h]
        let cross = s.any(v => v > 0) and s.any(v => v < 0)
        let li = hull_line_intersect(h, p, b)
        if (li != None) != cross: bad = "op {k}: hull_line_intersect({h}, {p}, {b})"
        elif li != None:
            let (i, j) = li!
            let hits = (e: Int) => s[e] * s[(e + 1) % h.len] <= 0
            if not (i < j and hits(i) and hits(j)): bad = "op {k}: hull_line_intersect edges {i} {j}"
    elif op == 2:
        // Minkowski sums, overlap and triangulation of hulls.
        let a = wrap(pts + [rp(mut rng, 4)], false)
        let b = wrap([rp(mut rng, 4) for _ in 0..rng.int(1, 6)], false)
        let sums = [p + q for p in a, q in b]
        var ms = wrap(sums, false)
        if ms.len > 2:
            var st = 0
            for i in 0..ms.len:
                if (ms[i].y, ms[i].x) < (ms[st].y, ms[st].x): st = i
            ms = [ms[(st + i) % ms.len] for i in 0..ms.len]
        if ms.len > 2 and minkowski_sum(a, b) != ms: bad = "op {k}: minkowski_sum({a}, {b}) want {ms}"
        if bad != "": break
        let ab = wrap(a + b, false)
        let ov = a.any(p => where(b, p) != Location.Outside) or b.any(p => where(a, p) != Location.Outside) or (0..a.len).any(i => (0..b.len).any(j =>
            let (p, q, r, t) = (a[i], a[(i + 1) % a.len], b[j], b[(j + 1) % b.len])
            let (d1, d2, d3, d4) = (cr(q - p, r - p), cr(q - p, t - p), cr(t - r, p - r), cr(t - r, q - r))
            d1 * d2 < 0 and d3 * d4 < 0
        ))
        if a.len >= 3 and b.len >= 3 and convex_polygons_overlap(a, b) != ov: bad = "op {k}: convex_polygons_overlap({a}, {b})"
        if bad != "": break
        let ua = upper_hull(srt(a))
        let ub = upper_hull(srt(b))
        let la = lower_hull(srt(a))
        let lb = lower_hull(srt(b))
        let wu = upper_hull(srt([p + q for p in ua, q in ub]))
        let wl = lower_hull(srt([p + q for p in la, q in lb]))
        if minkowski_upper(ua, ub) != wu or minkowski_lower(la, lb) != wl: bad = "op {k}: minkowski_upper/lower of {a}, {b}"
        if bad != "" or ab.len < 3: continue
        let tri = polygon_triangulate(ab)
        var tot = 0
        var a2 = 0
        for i in 0..ab.len: a2 += cr(ab[i], ab[(i + 1) % ab.len])
        for (i, j, l) in tri:
            let t = cr(ab[j] - ab[i], ab[l] - ab[i])
            if t < 0: bad = "op {k}: polygon_triangulate clockwise triangle"
            tot += t
        if tri.len != ab.len - 2 or tot != a2: bad = "op {k}: polygon_triangulate({ab})"
    else:
        if rng.int(0, 9) == 0:
            oh.clear()
            added = []
        let p = rp(mut rng, 6)
        let was = where(wrap(added, false), p)
        if oh.add(p) != (was == Location.Outside): bad = "op {k}: OnlineHull.add({p}) after {added}"
        added.push(p)
        let h = wrap(added, false)
        let q = rp(mut rng, 7)
        let dir = rp(mut rng, 3)
        var a2 = 0
        for i in 0..h.len: a2 += cr(h[i], h[(i + 1) % h.len])
        var ext = 0
        for i in 0..h.len:
            if dt(h[i], dir) > dt(h[ext], dir): ext = i
        if oh.vertices() != h or oh.iter().collect() != h: bad = "op {k}: OnlineHull vertices {oh.vertices()} want {h}"
        elif oh.len != h.len or oh.is_empty or oh.area2 != a2: bad = "op {k}: OnlineHull len or area2"
        elif oh.locate(q) != where(h, q): bad = "op {k}: OnlineHull.locate({q})"
        elif oh.extreme(dir) != Some(h[ext]): bad = "op {k}: OnlineHull.extreme({dir})"
print(if bad == "": "ok" else: bad)
