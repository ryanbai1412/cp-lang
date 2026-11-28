// expect: ok, output "3\n2\n1"
// engines: next
use std.heap.{MaxHeap}
let h = MaxHeap.of([1, 3, 2])
for x in h: print(x)
