// expect: ok, output "YES"
use std.geometry.{arg}
let ok = arg(Pt(1, 1)) == arg(Pt(2, 2)) and arg(Pt(1, 0)) != arg(Pt(-1, 0)) and arg(Pt(0, -3)) == arg(Pt(0, -1)) and arg(Pt(1, 2)) != arg(Pt(2, 1))
print(yes(ok))
