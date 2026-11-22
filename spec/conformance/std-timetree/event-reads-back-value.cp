// expect: ok, output "e"
use std.timetree.{TimeTree}
var tt = TimeTree<Str>(2)
tt.add_interval(0, 2, "e")
print(tt.event(0))
