// expect: ok, output "1 2 3 3"
use std.heap.{Heap}
let h = Heap.of([3, 1, 2])
print("{h}", h.len)
