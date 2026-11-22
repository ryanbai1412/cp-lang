// expect: ok, output "YES YES YES"
use std.geometry.{locate_convex, Location, Pt}
let sq = [Pt(0, 0), Pt(4, 0), Pt(4, 4), Pt(0, 4)]
print(yes(locate_convex(sq, Pt(4, 4)) == Location.Boundary), yes(locate_convex(sq, Pt(2, 0)) == Location.Boundary), yes(locate_convex(sq, Pt(5, 2)) == Location.Outside))
