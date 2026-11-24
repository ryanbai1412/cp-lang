// expect: ok, output "2 4\n3 YES NO 2\n1 3\n0"
// A drain removes every live point of the half-open rectangle; a removed or drained point never comes back.
use std.geometry.{Pt, PointSet}
var s = PointSet.of([Pt(1, 1), Pt(2, 5), Pt(3, 3), Pt(5, 2), Pt(4, 4)])
print(s.drain(2..5, 2..5).sorted())
print(s.len, yes(s.remove(0)), yes(s.remove(0)), s.len)
print(s.drain(0..10, 0..10).sorted())
print(s.drain(0..10, 0..10).len)
