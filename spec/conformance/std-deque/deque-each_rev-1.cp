// expect: ok, output "YES"
use std.deque.{Deque}
print(Deque.of([1, 2]).each_rev().collect() == [2, 1])
