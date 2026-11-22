// expect: ok, output "YES"
use std.heap.{ErasableHeap}
var h: ErasableHeap<Int> = ErasableHeap()
h.push(1)
h.push(2)
h.remove(1)
print(h.peek() == Some(2))
