// expect: ok, output "1 3 2"
use std.heap.{ErasableHeap}
var h: ErasableHeap<Int> = ErasableHeap()
for x in [3, 1, 2]: h.push(x)
h.remove(2)
print("{h}", h.len)
