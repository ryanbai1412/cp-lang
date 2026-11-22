// expect: ok, output "YES"
use std.heap.{MinMaxHeap}
print(yes(MinMaxHeap.of([7]).first == Some(7)))
