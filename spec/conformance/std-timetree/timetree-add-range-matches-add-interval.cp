// expect: ok, output "7\n7"
use std.timetree.{TimeTree}
var tt = TimeTree<Str>(3)
tt.add_interval(1, 3, "e")
print(tt.steps().len)
var t2 = TimeTree<Str>(3)
t2.add_range(1..3, "e")
print(t2.steps().len)
