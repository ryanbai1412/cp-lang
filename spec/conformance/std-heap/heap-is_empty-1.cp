// expect: ok, output "YES"
use std.heap.{Heap}
print(yes(Heap.of([3]).is_empty == false))
