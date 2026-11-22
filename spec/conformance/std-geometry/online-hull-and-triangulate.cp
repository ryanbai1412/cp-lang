// expect: ok, output "3 16 YES YES YES YES\n2 4"
// rule: 08.1.1
use std.geometry.{OnlineHull, Pt, Location, polygon_triangulate}
var h = OnlineHull()
let _ = h.add(Pt(0, 0))
let _ = h.add(Pt(4, 0))
let _ = h.add(Pt(0, 4))
let added_in = h.add(Pt(1, 1))
print(
    h.len,
    h.area2,
    yes(not added_in),
    yes(h.locate(Pt(1, 1)) == Location.Inside),
    yes(h.locate(Pt(2, 0)) == Location.Boundary),
    yes(h.extreme(Pt(1, 1)) == Some(Pt(4, 0))),
)
let sq = polygon_triangulate([Pt(0, 0), Pt(2, 0), Pt(2, 2), Pt(0, 2)])
let lshape = polygon_triangulate([Pt(0, 0), Pt(2, 0), Pt(2, 1), Pt(1, 1), Pt(1, 2), Pt(0, 2)])
print(sq.len, lshape.len)
