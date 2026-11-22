// expect: ok, output "YES"
use std.heap.{MinMaxHeap}
print(MinMaxHeap.of([3, 1, 2]).last == Some(3))
