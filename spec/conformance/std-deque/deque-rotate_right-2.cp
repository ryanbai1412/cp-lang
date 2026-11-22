// expect: ok, output "YES"
use std.deque.{Deque}
var q = Deque.of([1, 2, 3])
q.rotate_right(4)
print(q.to_list() == [3, 1, 2])
