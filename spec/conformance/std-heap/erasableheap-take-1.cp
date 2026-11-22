// expect: ok, output "YES"
use std.heap.{ErasableHeap}
var h: ErasableHeap<Int> = ErasableHeap()
h.push(1)
let t = h.take()
print(h.is_empty and t.len == 1)
