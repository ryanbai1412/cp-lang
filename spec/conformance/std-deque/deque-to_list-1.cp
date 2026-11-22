// expect: ok, output "YES"
use std.deque.{Deque}
var q = Deque.of([2])
q.push_front(1)
print(q.to_list() == [1, 2])
