// expect: ok, output "2 2 -1"
use std.heap.{MeldHeap}
var a = MeldHeap.of([4, 2])
a.meld(MeldHeap.of([]))
var e: MeldHeap<Int> = MeldHeap()
e.meld(MeldHeap.of([2]))
print(a.peek ?? -1, e.pop() ?? -1, e.pop() ?? -1)
