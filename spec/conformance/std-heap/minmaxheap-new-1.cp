// expect: ok, output "YES"
use std.heap.{MinMaxHeap}
var h: MinMaxHeap<Int> = MinMaxHeap()
h.push(2)
h.push(7)
print(yes(h.first == Some(2) and h.last == Some(7)))
