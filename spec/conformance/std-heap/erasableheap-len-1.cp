// expect: ok, output "YES"
use std.heap.{ErasableHeap}
var h: ErasableHeap<Int> = ErasableHeap()
h.push(1)
h.push(1)
h.remove(1)
print(yes(h.len == 1))
