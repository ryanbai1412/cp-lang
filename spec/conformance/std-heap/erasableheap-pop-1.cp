// expect: ok, output "YES"
use std.heap.{ErasableHeap}
var h: ErasableHeap<Int> = ErasableHeap()
h.push(5)
h.push(2)
print(h.pop() == Some(2) and h.len == 1)
