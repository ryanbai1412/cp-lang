// expect: ok, output "YES"
use std.heap.{MinMaxHeap}
var h = MinMaxHeap.of([2, 5, 3])
print(h.pop_last() == Some(5) and h.last == Some(3))
