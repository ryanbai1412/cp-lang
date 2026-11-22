// expect: ok, output "YES"
use std.heap.{MinMaxHeap}
var h = MinMaxHeap.of([3, 1, 4, 1, 5])
print(h.pop_last() == Some(5) and h.pop_first() == Some(1) and h.first == Some(1))
