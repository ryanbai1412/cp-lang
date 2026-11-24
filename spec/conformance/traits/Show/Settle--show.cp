// expect: ok, output "1 settled, 2 open"
use std.heap.{Settle}
var s: Settle<Int, Int> = Settle()
s.offer(1, 4)
s.offer(2, 6)
s.offer(3, 1)
s.pop()
print(s)
