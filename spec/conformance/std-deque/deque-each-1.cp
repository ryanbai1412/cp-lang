// expect: ok, output "YES"
use std.deque.{Deque}
print(Deque.of([1, 2]).each().collect() == [1, 2])
