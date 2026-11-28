// expect: ok, output "1\n3"
// engines: next
use std.heap.{ErasableHeap}
var h: ErasableHeap<Int> = ErasableHeap()
for x in [3, 1, 2]: h.push(x)
h.remove(2)
for x in h: print(x)
