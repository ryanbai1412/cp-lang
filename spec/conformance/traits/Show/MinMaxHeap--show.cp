// expect: ok, output "2 4 7 3"
use std.heap.{MinMaxHeap}
let h = MinMaxHeap.of([2, 7, 4])
print("{h}", h.len)
