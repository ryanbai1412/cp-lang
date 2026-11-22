// expect: ok, output "YES"
use std.heap.{IndexedHeap}
domain V = 0..4
var h: IndexedHeap<V, Int> = IndexedHeap()
print(yes(h.increase(V(1), 5) and not h.increase(V(1), 2) and h.get(V(1)) == Some(5)))
