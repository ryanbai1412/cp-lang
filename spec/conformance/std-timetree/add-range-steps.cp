// expect: ok, output "7\nQuery 0\nApply 0\nQuery 1\nUndo\nApply 1\nQuery 2\nUndo"
// 1..3 over a 4-leaf tree covers leaves 1 and 2, two separate nodes, so the event is applied twice.
use std.timetree.{TimeTree}
var tt = TimeTree<Str>(3)
tt.add_range(1..3, "e")
print(tt.steps().len)
for s in tt.steps(): print("{s}")
