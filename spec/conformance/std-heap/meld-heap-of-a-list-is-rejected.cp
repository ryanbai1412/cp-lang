// expect: E0302
// A list is not `MeldHeap`'s key function; the contents go through `MeldHeap.of(xs)`.
use std.heap.{MeldHeap}
let a: List<Int> = [3, 1, 3]
let h = MeldHeap(a) // <- here
