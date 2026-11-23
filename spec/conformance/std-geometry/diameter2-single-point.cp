// expect: ok, output "0 0 2"
use std.geometry.{diameter2}
print(diameter2([]), diameter2([Pt(5, 5)]), diameter2([Pt(0, 0), Pt(1, 1)]))
