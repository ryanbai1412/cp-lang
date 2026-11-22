// expect: ok, output "YES"
use std.heap.{MinMaxHeap}
var h = MinMaxHeap.of([2])
h.clear()
print(yes(h.first == None))
