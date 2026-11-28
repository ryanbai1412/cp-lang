// expect: ok, output "2 5\n0 9"
// engines: next
use std.heap.{IndexedHeap}
domain V = 0..4
var h: IndexedHeap<V, Int> = IndexedHeap()
h.set(V(2), 5)
h.set(V(0), 9)
for (v, k) in h: print(v, k)
