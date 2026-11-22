// expect: ok, output "YES"
use std.deque.{Deque}
print(yes(Deque.of([7, 8]).items.collect() == [(0, 7), (1, 8)]))
