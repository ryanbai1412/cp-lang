// expect: ok, output "3 2 1 3"
use std.heap.{MaxHeap}
let h = MaxHeap.of([1, 3, 2])
print("{h}", h.len)
