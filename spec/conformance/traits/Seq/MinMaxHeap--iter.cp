// expect: ok, output "2\n4\n7"
// engines: next
use std.heap.{MinMaxHeap}
let h = MinMaxHeap.of([2, 7, 4])
for x in h: print(x)
