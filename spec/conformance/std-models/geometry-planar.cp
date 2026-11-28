// expect: ok, output "ok"
// input: "1 300\n"
// model: delaunay
// model: voronoi_cells
// model: planar_faces
// model: point_location_planar
// Points with no three collinear and no four cocircular. Delaunay against every triangle whose
// circumcircle holds no other point; Voronoi cells against the exact area of the half-planes
// nearer to their point; faces of random subgraphs of the triangulation against a walk that
// turns clockwise at each vertex; and point location against ray casting over those faces.
// Reads `seed n`; prints the first op that differs.
use std.rand.{Rng}
use std.geometry.{PI, Pt, Vec, delaunay, planar_faces, point_location_planar, voronoi_cells}
let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
fn cr(a: Pt, b: Pt) -> Int = a.x * b.y - a.y * b.x
fn dt(a: Pt, b: Pt) -> Int = a.x * b.x + a.y * b.y
fn onseg(p: Pt, a: Pt, b: Pt) -> Bool = cr(p - a, b - a) == 0 and dt(p - a, p - b) <= 0
// Positive when `d` is strictly inside the circle through the counter-clockwise `a`, `b`, `c`.
fn incircle(a: Pt, b: Pt, c: Pt, d: Pt) -> Int:
    let (u, v, w) = (a - d, b - d, c - d)
    let (x, y, z) = (u.x * u.x + u.y * u.y, v.x * v.x + v.y * v.y, w.x * w.x + w.y * w.y)
    u.x * (v.y * z - y * w.y) - u.y * (v.x * z - y * w.x) + x * (v.x * w.y - v.y * w.x)
fn cw(a: Vec, b: Vec) -> F64 = a.x * b.y - a.y * b.x
fn hp_len(cons: List<(Vec, Vec)>, x: F64, bb: F64) -> F64:
    var (lo, hi) = (-bb, bb)
    for (p, d) in cons:
        if d.x > 0.0: lo = max(lo, p.y + d.y * (x - p.x) / d.x)
        elif d.x < 0.0: hi = min(hi, p.y + d.y * (x - p.x) / d.x)
        elif -d.y * (x - p.x) < 0.0: hi = lo
    max(0.0, hi - lo)
// The exact area of half-planes `(p, dir)` (left of `dir`) in the box `[-bb, bb]^2`: the slice
// length is linear between line crossings, so one midpoint per piece is exact.
fn hp_area(cons: List<(Vec, Vec)>, bb: F64) -> F64:
    var xs = [-bb, bb]
    for (p, d) in cons:
        if d.x == 0.0: xs.push(p.x)
        elif d.y != 0.0:
            for yb in [-bb, bb]: xs.push(p.x + d.x * (yb - p.y) / d.y)
    for i in 0..cons.len:
        for j in i + 1..cons.len:
            let (p, d) = cons[i]
            let (q, e) = cons[j]
            if cw(d, e) != 0.0: xs.push(p.x + d.x * cw(q - p, e) / cw(d, e))
    var ok = [x for x in xs if -bb <= x <= bb]
    ok.sort()
    var s = 0.0
    for i in 1..ok.len: s += hp_len(cons, (ok[i - 1] + ok[i]) / 2.0, bb) * (ok[i] - ok[i - 1])
    s
fn vec(p: Pt) -> Vec = Vec(F64(p.x), F64(p.y))
var bad = ""
for k in 0..n:
    if bad != "": break
    var pts: List<Pt> = []
    for _ in 0..rng.int(0, 8):
        let p = Pt(rng.int(-6, 6), rng.int(-6, 6))
        var ok = not pts.any(q => q == p)
        for i in 0..pts.len:
            for j in i + 1..pts.len:
                if cr(pts[j] - pts[i], p - pts[i]) == 0: ok = false
                for l in j + 1..pts.len:
                    if incircle(pts[i], pts[j], pts[l], p) == 0: ok = false
        if ok: pts.push(p)
    domain G = 0..pts.len
    let gp = [G] g => pts[g.ord]
    var tris: List<(Int, Int, Int)> = []
    for i in 0..pts.len:
        for j in i + 1..pts.len:
            for l in j + 1..pts.len:
                let t = if cr(pts[j] - pts[i], pts[l] - pts[i]) > 0: (i, j, l) else: (i, l, j)
                if (0..pts.len).all(q => q == i or q == j or q == l or incircle(pts[t.0], pts[t.1], pts[t.2], pts[q]) < 0): tris.push(t)
    tris.sort()
    let got = [(a.ord, b.ord, c.ord) for (a, b, c) in delaunay(gp)]
    if got != tris: bad = "op {k}: delaunay({pts}) = {got} want {tris}"
    elif delaunay([Pt(i, 2 * i + 1) for i in 0..pts.len]) != []: bad = "op {k}: delaunay of collinear points"
    if bad != "": break
    let bound = F64(rng.int(1, 8))
    let cells = voronoi_cells(gp, bound)
    for i in 0..pts.len:
        if bad != "": break
        let cell = cells[G(i)]
        var cons: List<(Vec, Vec)> = []
        for j in 0..pts.len:
            if j != i:
                let m = (vec(pts[i]) + vec(pts[j])) * 0.5
                let nv = vec(pts[i]) - vec(pts[j])
                cons.push((m, Vec(nv.y, -nv.x)))
        let want = hp_area(cons, bound)
        var area = 0.0
        for t in 0..cell.len: area += cw(cell[t], cell[(t + 1) % cell.len]) / 2.0
        // The first corner is the least by `(y, x)`, up to rounding.
        let first = cell.len == 0 or cell.all(v => v.y > cell[0].y + 1e-9 or (v.y >= cell[0].y - 1e-9 and v.x >= cell[0].x - 1e-9))
        let inside = cell.all(v => abs(v.x) <= bound + 1e-6 and abs(v.y) <= bound + 1e-6 and cons.all(c => cw(c.1, v - c.0) >= -1e-6))
        if abs(area - want) > 1e-6 or not inside or not first or (want > 1e-9) != (cell.len >= 3): bad = "op {k}: voronoi_cells({pts}, {bound})[{i}] = {cell}, area {area} want {want}"
    if bad != "": break
    // A random planar subgraph: some edges of the triangulation.
    var es: List<(Int, Int)> = []
    for (a, b, c) in tris:
        for (u, v) in [(a, b), (b, c), (c, a)]:
            let e = (min(u, v), max(u, v))
            if not es.any(f => f == e): es.push(e)
    let edges = [if rng.int(0, 1) == 0: (u, v) else: (v, u) for (u, v) in es if rng.int(0, 3) != 0]
    var adj: List<List<Int>> = [[] for _ in 0..pts.len]
    for (u, v) in edges:
        adj[u].push(v)
        adj[v].push(u)
    let ang = (v: Int, w: Int) => atan2(F64(pts[w].y - pts[v].y), F64(pts[w].x - pts[v].x))
    var used: List<(Int, Int)> = []
    var want: List<List<Int>> = []
    for v0 in 0..pts.len:
        for w0 in adj[v0]:
            if used.any(e => e == (v0, w0)): continue
            var cyc: List<Int> = []
            var (u, v) = (v0, w0)
            while true:
                used.push((u, v))
                cyc.push(u)
                // The next neighbour of `v` clockwise after `u`: the largest angle below `u`'s.
                let au = ang(v, u)
                var best = u
                var bd: F64 = 1e9
                for w in adj[v]:
                    if w == u: continue
                    var d = au - ang(v, w)
                    if d <= 0.0: d += 2.0 * PI
                    if d < bd:
                        bd = d
                        best = w
                (u, v) = (v, best)
                if (u, v) == (v0, w0): break
            // A vertex met twice (at a bridge) starts the face at its lexicographically least turn.
            var best_rot = cyc.clone()
            for r in 0..cyc.len:
                let rot = [cyc[(r + t) % cyc.len] for t in 0..cyc.len]
                if rot < best_rot: best_rot = rot
            want.push(best_rot)
    want.sort()
    let faces = [[v.ord for v in f] for f in planar_faces(gp, [(G(u), G(v)) for (u, v) in edges])]
    if faces != want: bad = "op {k}: planar_faces({pts}, {edges}) = {faces} want {want}"
    if bad != "": break
    let qs = [Pt(rng.int(-7, 7), rng.int(-7, 7)) for _ in 0..rng.int(0, 6)]
    let loc = point_location_planar(gp, [(G(u), G(v)) for (u, v) in edges], qs)
    for t in 0..qs.len:
        let q = qs[t]
        var best: ?Int = None
        var best_area = 0
        if not edges.any(e => onseg(q, pts[e.0], pts[e.1])) and not pts.any(p => p == q):
            for f in 0..want.len:
                let cyc = want[f]
                var a2 = 0
                var inside = false
                for i in 0..cyc.len:
                    let (a, b) = (pts[cyc[i]], pts[cyc[(i + 1) % cyc.len]])
                    a2 += cr(a, b)
                    if (a.y > q.y) != (b.y > q.y) and (cr(b - a, q - a) > 0) == (b.y > a.y): inside = not inside
                if a2 > 0 and inside and (best == None or a2 < best_area):
                    best = Some(f)
                    best_area = a2
        if loc[t] != best:
            bad = "op {k}: point_location_planar({pts}, {edges}, {q}) = {if loc[t] == None: -1 else: loc[t]!} want {if best == None: -1 else: best!}"
            break
print(if bad == "": "ok" else: bad)
