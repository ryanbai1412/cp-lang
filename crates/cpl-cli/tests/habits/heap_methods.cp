use std.heap.{Heap}
var h: Heap<Int> = Heap()
heapq.heappush(h, 2)  // <- here
print(h.pop()!)
