// expect: ok, output "4\n1\n3"
use std.deque.{Deque}
let d = Deque.of([3, 1, 4])
for x in d.rev: print(x)
