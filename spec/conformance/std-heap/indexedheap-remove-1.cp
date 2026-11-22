// expect: ok, output "YES"
use std.heap.{IndexedHeap}
domain V = 0..4
var h: IndexedHeap<V, Int> = IndexedHeap()
h.set(V(1), 5)
print(h.remove(V(1)) == Some(5) and h.remove(V(1)) == None)
