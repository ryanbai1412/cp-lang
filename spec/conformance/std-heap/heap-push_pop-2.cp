// expect: ok, output "YES"
use std.heap.{Heap}
var h = Heap.of([2])
print(h.push_pop(1) == 1 and h.peek == Some(2))
