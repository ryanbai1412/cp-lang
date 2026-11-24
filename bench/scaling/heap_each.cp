// n: 4096
// Each walk pops a copy, so it is timed on heaps of xs alone, apart from
// heap.cp, whose 4n working set sits at the edge of L2.
use std.heap.{MinMaxHeap, ErasableHeap, MeldHeap, IndexedHeap}

let n: Int = read()
domain V = 0..n
let reps = 16
let xs = [i * 7919 % n for i in 0..n]
var s = 0

var h = Heap<Int>()
var x = MaxHeap<Int>()
var m = MinMaxHeap<Int>()
var e = ErasableHeap<Int>()
var mk = MeldHeap<Int>()
var ih = IndexedHeap<V, Int>()
for i in 0..n:
    let v = xs[i]
    h.push(v)
    x.push(v)
    m.push(v)
    e.push(v)
    mk.push(v)
    ih.set(V(i), v)
for _ in 0..reps:
    s += h.each().collect().len // timed: heap.Heap.each
    s += x.each().collect().len // timed: heap.MaxHeap.each
    s += m.each().collect().len // timed: heap.MinMaxHeap.each
    s += e.each().collect().len // timed: heap.ErasableHeap.each
    s += mk.each().collect().len // timed: heap.MeldHeap.each
    s += ih.each().collect().len // timed: heap.IndexedHeap.each
print(s)
