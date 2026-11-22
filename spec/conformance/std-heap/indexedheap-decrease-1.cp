// expect: ok, output "YES"
use std.heap.{IndexedHeap}
domain V = 0..4
var h: IndexedHeap<V, Int> = IndexedHeap()
print(yes(h.decrease(V(1), 5) and not h.decrease(V(1), 5) and h.decrease(V(1), 3) and h.get(V(1)) == Some(3)))
