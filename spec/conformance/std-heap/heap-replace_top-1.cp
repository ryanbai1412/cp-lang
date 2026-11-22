// expect: ok, output "YES"
use std.heap.{Heap}
var h = Heap.of([2, 5])
print(h.replace_top(9) == Some(2) and h.peek == Some(5))
