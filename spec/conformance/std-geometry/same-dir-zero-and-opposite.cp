// expect: ok, output "NO NO YES"
use std.geometry.{same_dir, Pt}
print(yes(same_dir(Pt(0, 0), Pt(0, 0))), yes(same_dir(Pt(1, 2), Pt(-1, -2))), yes(same_dir(Pt(1, 2), Pt(2, 4))))
