// expect: ok, output "YES"
use std.heap.{IndexedHeap}
domain V = 0..4
var h: IndexedHeap<V, Int> = IndexedHeap()
h.set(V(2), 5)
print(h.get(V(2)) == Some(5) and h.get(V(0)) == None)
