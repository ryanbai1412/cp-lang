// here: 5
// Python's `heapq.heappush(h, x)` (self-run mistakes): the fix wrote `Heap.heappush(h, 3)`, which has no such method.
use std.heap.{Heap}
var h: Heap<Int> = Heap()
heapq.heappush(h, 3)
print(heapq.heappop(h))
