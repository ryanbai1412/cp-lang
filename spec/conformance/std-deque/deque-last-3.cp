// expect: ok, output "YES"
use std.deque.{Deque}
print(yes(Deque.of([4]).last(1) == None))
