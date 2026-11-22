// expect: ok, output "YES"
use std.heap.{MinMaxHeap}
var h = MinMaxHeap.of([2, 5])
h.push(9)
print(h.last == Some(9) and h.first == Some(2))
