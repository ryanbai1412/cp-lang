// expect: ok, output "YES"
use std.heap.{ErasableHeap}
var h: ErasableHeap<Int> = ErasableHeap()
h.push(4)
print(h.peek() == Some(4))
