// expect: ok, output "YES"
use std.heap.{Heap}
var h = Heap.of([3, 1, 2])
print(h.pop() == Some(1) and h.len == 2)
