// expect: ok, output "9 9 4 -1"
use std.heap.{MaxHeap}
var h = MaxHeap<Int>()
for x in [4, 9, 9]: h.push(x)
print(h.pop() ?? -1, h.pop() ?? -1, h.pop() ?? -1, h.pop() ?? -1)
