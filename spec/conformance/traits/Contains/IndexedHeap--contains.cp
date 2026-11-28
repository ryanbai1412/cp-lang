// expect: ok, output "YES NO"
// engines: next
use std.heap.{IndexedHeap}
domain V = 0..4
var h: IndexedHeap<V, Int> = IndexedHeap()
h.set(V(2), 5)
h.set(V(0), 9)
print(yes(h.contains(V(2))), yes(h.contains(V(1))))
