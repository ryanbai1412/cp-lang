// expect: E0302 at 4
// A list is not `MinMaxHeap`'s key function; the contents go through `MinMaxHeap.of(xs)`.
use std.heap.{MinMaxHeap}
let a: List<Int> = [3, 1, 3]
let h = MinMaxHeap(a)
