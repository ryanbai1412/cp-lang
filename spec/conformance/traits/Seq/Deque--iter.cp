// expect: ok, output "3\n1\n4"
use std.deque.{Deque}
let d = Deque.of([3, 1, 4])
for x in d: print(x)
