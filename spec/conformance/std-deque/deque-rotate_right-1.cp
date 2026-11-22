// expect: ok, output "YES"
use std.deque.{Deque}
var q = Deque.of([1, 2, 3, 4])
q.rotate_right(1)
print(yes(q.to_list() == [4, 1, 2, 3]))
