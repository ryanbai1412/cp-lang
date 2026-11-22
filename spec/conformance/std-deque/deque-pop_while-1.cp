// expect: ok, output "YES"
use std.deque.{Deque}
var q = Deque.of([5, 1, 2])
print(q.pop_while(x => x < 4) == [2, 1] and q.to_list() == [5])
