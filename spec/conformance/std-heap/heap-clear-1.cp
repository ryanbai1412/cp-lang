// expect: ok, output "YES"
use std.heap.{Heap}
var h = Heap.of([1, 2])
h.clear()
print(yes(h.pop() == None))
