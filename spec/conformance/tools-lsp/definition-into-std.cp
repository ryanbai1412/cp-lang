// expect: ok
// rule: lsp.3
// tool: lsp definition MinMax|Heap<Int>
// has: std/heap.cp"}
use std.heap.{MinMaxHeap}
var h = MinMaxHeap<Int>()
h.push(3)
print(h.len)
