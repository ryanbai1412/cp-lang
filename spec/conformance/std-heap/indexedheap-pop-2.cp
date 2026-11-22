// expect: ok, output "YES"
use std.heap.{IndexedHeap}
domain V = 0..4
var h: IndexedHeap<V, Int> = IndexedHeap()
h.set(V(3), 1)
h.set(V(1), 1)
print(h.pop() == Some((V(1), 1)))
