// expect: ok, output "0 3 2"
use std.deque.{Deque}
var q = Deque.of([1, 2])
q.push_front(0)
q.push(3)
print(q.pop_front()!, q.pop()!, q.len)
