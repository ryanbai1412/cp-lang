// expect: ok, output "1 2"
use std.seq.{OnlineLis}
var t = OnlineLis<Int>()
for x in [3, 1, 2]: t.push(x)
print(t)
