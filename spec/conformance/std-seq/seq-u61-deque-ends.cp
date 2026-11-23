// expect: ok, output "1 3 1"
// Acceptance for seq/U61 (push and pop at both ends), handbook/product/std/sequences-usecases.md.
use std.deque.{Deque}
var d: Deque<Int> = Deque()
d.push(2)
d.push_front(1)
d.push(3)
print(d.pop_front()!, d.pop()!, d.len)
