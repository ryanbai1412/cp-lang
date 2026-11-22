// expect: ok, output "YES"
use std.deque.{Deque}
print(Deque.of([4, 5, 6]).last == Some(6))
