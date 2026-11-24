// expect: ok
// gap: tools 2026-11-24T05Z
// rule: bundle.6
// tool: vendor {case}
// `cpl vendor` copies the pinned libraries a program uses next to it, so the program rebuilds byte for byte without the store.
use std.heap.{MinMaxHeap}
var h = MinMaxHeap<Int>()
h.push(3)
print(h.len)
