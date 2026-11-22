// expect: ok, output "15 15 YES"
use std.timetree.{TimeTree, TimeStep}
var tt = TimeTree<Int>(5)
tt.add_interval(0, 3, 10)
tt.add_range(1..5, 20)
tt.add_range(2..2, 30)
let a = tt.steps()
var b = List<TimeStep>()
for s in tt.walk(): b.push(s)
print(a.len, b.len, a == b)
