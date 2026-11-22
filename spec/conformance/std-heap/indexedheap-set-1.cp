// expect: ok, output "YES"
use std.heap.{IndexedHeap}
domain V = 0..4
var h: IndexedHeap<V, Int> = IndexedHeap()
h.set(V(1), 5)
h.set(V(1), 9)
print(h.get(V(1)) == Some(9))
