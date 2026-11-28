// expect: ok, output "1\n2\n3"
// engines: next
use std.heap.{MeldHeap}
let h = MeldHeap.of([3, 1, 2])
for x in h: print(x)
