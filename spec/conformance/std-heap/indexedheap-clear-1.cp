// expect: ok, output "YES"
use std.heap.{IndexedHeap}
domain V = 0..4
var h: IndexedHeap<V, Int> = IndexedHeap()
h.set(V(1), 5)
h.clear()
print(h.get(V(1)) == None)
