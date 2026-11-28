// expect: ok, output "0 1 3 4\n0 1 3 4\n5 4"
// The point-index fns walk their input's keys: a list gives `Int` positions, an `[V]Pt` gives `V`s.
use std.geometry.{voronoi_cells}
let pts = [Pt(0, 0), Pt(4, 0), Pt(2, 1), Pt(4, 4), Pt(0, 4)]
print(convex_hull_idx(pts))
domain V = 0..5
let pv: [V]Pt = [V] v => pts[v.ord]
print(convex_hull_idx(pv).map(v => v.ord))
print(voronoi_cells(pts, 100.0).len, voronoi_cells(pv, 100.0)[V(2)].len)
