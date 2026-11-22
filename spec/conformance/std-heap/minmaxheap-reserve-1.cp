// expect: ok, output "YES"
use std.heap.{MinMaxHeap}
var h = MinMaxHeap.of([2])
h.reserve(100)
print(h.first == Some(2))
