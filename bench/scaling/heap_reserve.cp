// n: 512
// reserve moves the elements already there. At n = 512 the copy and the doubled block
// stay under 128 KB at 4n too, so malloc serves both from the heap and no size pays
// fresh-page faults that the other doesn't.
use std.heap.{MinMaxHeap, MeldHeaps}

let n: Int = read()
let xs = [i * 7919 % n for i in 0..n]
var s = 0
for _ in 0..40000:
    var r = Heap<Int>.of(xs)
    r.reserve(2 * n) // timed: heap.Heap.reserve
    var rx = MaxHeap<Int>.of(xs)
    rx.reserve(2 * n) // timed: heap.MaxHeap.reserve
    var rm = MinMaxHeap<Int>.of(xs)
    rm.reserve(2 * n) // timed: heap.MinMaxHeap.reserve
    var rp = MeldHeaps<Int>()
    s += rp.of(xs).len
    rp.reserve(2 * n) // timed: heap.MeldHeaps.reserve
    s += r.len + rx.len + rm.len
print(s)
