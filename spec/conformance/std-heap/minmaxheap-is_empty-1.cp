// expect: ok, output "YES"
use std.heap.{MinMaxHeap}
print(MinMaxHeap.of([1]).is_empty == false)
