// expect: ok, output "1 2 3 3"
use std.heap.{MeldHeap}
let h = MeldHeap.of([3, 1, 2])
print("{h}", h.len)
