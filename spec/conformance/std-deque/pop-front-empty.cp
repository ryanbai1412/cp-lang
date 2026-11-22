// expect: ok, output "-1 0"
use std.deque.{Deque}
var q: Deque<Int> = Deque()
print(q.pop_front() ?? -1, q.len)
