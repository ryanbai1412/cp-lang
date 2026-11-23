// expect: ok, output "YES NO"
use std.geometry.{arg}
print(yes(arg(Pt(1, 0)) < arg(Pt(0, 1))), yes(arg(Pt(0, -1)) < arg(Pt(-1, 0))))
