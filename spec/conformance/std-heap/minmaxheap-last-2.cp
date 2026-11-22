// expect: ok, output "YES"
use std.heap.{MinMaxHeap}
print(yes(MinMaxHeap.of([7]).last == Some(7)))
