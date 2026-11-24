// expect: ok, output "ok"
// input: "1 400\n"
// model: polygon_triangulate
// Random star-shaped lattice polygons, with collinear and level vertices, against their area: n - 2
// counter-clockwise triangles whose areas sum to the polygon's and whose centroids lie inside it.
// Reads `seed n`; prints the first polygon that fails.
use std.geometry.{Location, Pt, locate, polygon_triangulate}
use std.rand.{Rng}
let seed0 = read<Int>()
let n = read<Int>()
var rng = Rng(seed0)
fn cr(a: Pt, b: Pt) -> Int = a.x * b.y - a.y * b.x
let dirs = [Pt(1, 0), Pt(2, 1), Pt(1, 1), Pt(1, 2), Pt(0, 1), Pt(-1, 2), Pt(-1, 1), Pt(-2, 1), Pt(-1, 0), Pt(-2, -1), Pt(-1, -1), Pt(-1, -2), Pt(0, -1), Pt(1, -2), Pt(1, -1), Pt(2, -1)]
var bad = ""
var tested = 0
for k in 0..n:
    if bad != "": break
    var poly: List<Pt> = []
    for d in dirs:
        if rng.int(0, 2) > 0: poly.push(d * rng.int(1, 6))
    let m = poly.len
    if m < 3 or (0..m).any(i => cr(poly[i], poly[(i + 1) % m]) <= 0): continue
    tested += 1
    let tri = polygon_triangulate(poly)
    var a2 = 0
    for i in 0..m: a2 += cr(poly[i], poly[(i + 1) % m])
    let big = [p * 3 for p in poly]
    var tot = 0
    for (i, j, l) in tri:
        let t = cr(poly[j] - poly[i], poly[l] - poly[i])
        if t <= 0 or locate(big, poly[i] + poly[j] + poly[l]) == Location.Outside: bad = "{poly}: triangle {(i, j, l)}"
        tot += t
    if bad == "" and (tri.len != m - 2 or tot != a2): bad = "{poly}: {tri.len} triangles, area2 {tot} want {a2}"
print(if bad != "": bad elif tested < n / 4: "only {tested} polygons" else: "ok")
