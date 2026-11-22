// expect: ok, output "YES"
use std.heap.{MinMaxHeap}
print(yes(MinMaxHeap.of([1]).is_empty == false))
