// expect: ok, output "YES"
use std.heap.{Heap}
var h: Heap<Int> = Heap()
print(h.replace_top(4) == None and h.peek == Some(4))
