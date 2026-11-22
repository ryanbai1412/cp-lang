// expect: ok, output "-1 1"
use std.heap.{MeldHeap}
var h: MeldHeap<Int> = MeldHeap()
h.add_all(5)
let before = h.peek ?? -1
h.push(1)
print(before, h.peek ?? -1)
