// expect: ok, output "YES"
use std.heap.{MinMaxHeap}
print(MinMaxHeap.of([3, 1, 2]).first == Some(1))
