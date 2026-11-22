// expect: ok, output "YES"
use std.heap.{Heap}
print(Heap.of([3, 1, 2]).peek == Some(1))
