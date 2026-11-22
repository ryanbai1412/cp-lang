// expect: ok, output "YES"
use std.heap.{IndexedHeap}
domain V = 0..4
var h: IndexedHeap<V, Int> = IndexedHeap()
h.set(V(2), 5)
h.set(V(0), 7)
print(yes(h.pop() == Some((V(2), 5)) and h.len == 1))
