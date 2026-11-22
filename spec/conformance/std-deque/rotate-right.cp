// expect: ok, output "3 1 2"
use std.deque.{Deque}
var q = Deque.of([1, 2, 3])
q.rotate_right(1)
print(q.to_list())
