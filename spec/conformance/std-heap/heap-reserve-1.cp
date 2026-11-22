// expect: ok, output "YES"
use std.heap.{Heap}
var h = Heap.of([2])
h.reserve(100)
print(h.peek == Some(2))
