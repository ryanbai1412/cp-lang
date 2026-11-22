// expect: ok, output "YES"
use std.heap.{Heap}
var h: Heap<Int> = Heap()
h.push(3)
h.push(1)
print(h.pop() == Some(1))
