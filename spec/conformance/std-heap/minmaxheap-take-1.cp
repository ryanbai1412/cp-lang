// expect: ok, output "YES"
use std.heap.{MinMaxHeap}
var h = MinMaxHeap.of([2])
let t = h.take()
print(yes(h.is_empty and t.first == Some(2)))
