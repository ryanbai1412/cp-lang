// expect: ok, output "YES"
use std.deque.{Deque}
print(yes(Deque.of([4, 5, 6]).last(1) == Some(5)))
