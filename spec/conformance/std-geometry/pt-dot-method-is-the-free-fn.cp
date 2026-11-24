// expect: E0203
// has: "dot(Pt(1, 2), Pt(3, 4))"
// `Pt` keeps `dot` and `cross` as free fns only: the method spelling's error rewrites the call to the free fn, so `cpl fix` migrates it.
use std.geometry.{dot}
print(Pt(1, 2).dot(Pt(3, 4))) // <- here
