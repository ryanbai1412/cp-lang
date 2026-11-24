// expect: ok, output "2 YES YES"
// rule: 08.1.6
// R08.1.6: std's own `Eq` impls hold as table keys and inside composites on both engines:
// `ByAngle` calls same-direction vectors equal, `Mat` compares cells.
use std.geometry.{ByAngle}
let dirs = Set([ByAngle(Pt(1, 0)), ByAngle(Pt(2, 0)), ByAngle(Pt(0, 3))])
let a = Mat<Int>.from_rows([[1, 2], [3, 4]])
print(dirs.len, yes((a, 1) == (Mat<Int>.from_rows([[1, 2], [3, 4]]), 1)), yes([ByAngle(Pt(1, 1))] == [ByAngle(Pt(5, 5))]))
