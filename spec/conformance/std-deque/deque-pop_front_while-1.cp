// expect: ok, output "YES"
use std.deque.{Deque}
var q = Deque.of([1, 2, 5, 3])
print(yes(q.pop_front_while(x => x < 4) == [1, 2] and q.to_list() == [5, 3]))
