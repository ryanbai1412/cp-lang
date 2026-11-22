// expect: ok, output "YES"
use std.heap.{MinMaxHeap}
var h = MinMaxHeap.of([2, 5, 3])
print(h.pop_first() == Some(2) and h.first == Some(3))
