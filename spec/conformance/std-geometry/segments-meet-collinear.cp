// expect: ok, output "YES NO YES"
use std.geometry.{Pt, segments_meet}
print(yes(segments_meet(Pt(0, 0), Pt(2, 0), Pt(1, 0), Pt(3, 0))), yes(segments_meet(Pt(0, 0), Pt(1, 0), Pt(2, 0), Pt(3, 0))), yes(segments_meet(Pt(0, 0), Pt(1, 0), Pt(1, 0), Pt(2, 0))))
