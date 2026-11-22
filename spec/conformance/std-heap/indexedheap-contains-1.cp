// expect: ok, output "YES"
use std.heap.{IndexedHeap}
domain V = 0..4
var h: IndexedHeap<V, Int> = IndexedHeap()
h.set(V(2), 5)
print(yes(h.contains(V(2)) and not h.contains(V(3))))
