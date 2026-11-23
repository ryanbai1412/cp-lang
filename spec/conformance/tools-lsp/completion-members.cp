// expect: ok
// rule: lsp.2
// tool: lsp completion h.|
// has: "label":"push"
// has: "label":"len"
use std.heap.{MinMaxHeap}
var h = MinMaxHeap<Int>()
h.
