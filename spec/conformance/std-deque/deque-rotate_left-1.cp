// expect: ok, output "YES"
use std.deque.{Deque}
var q = Deque.of([1, 2, 3, 4])
q.rotate_left(1)
print(yes(q.to_list() == [2, 3, 4, 1]))
