// n: 2048
// At n = 2048 the 4n heaps (about 400 KB together) still fit in one core's L2, so the
// O(log n) sifts are timed in one cache tier; more rounds keep the O(n) lines over the floor.
use std.heap.{MinMaxHeap, ErasableHeap}

let n: Int = read()
let reps = 1 << 17
let rounds = 64
let xs = [i * 7919 % n for i in 0..n]
var s = 0

for _ in 0..reps:
    s += MinMaxHeap<Int>().len // timed: heap.MinMaxHeap.new
    s += ErasableHeap<Int>().len // timed: heap.ErasableHeap.new
for _ in 0..12 * rounds:
    s += MinMaxHeap<Int>.of(xs).len // timed: heap.MinMaxHeap.of

for round in 0..rounds:
    var m = MinMaxHeap<Int>()
    var e = ErasableHeap<Int>()
    for v in xs:
        m.push(v) // timed: heap.MinMaxHeap.push
        e.push(v) // timed: heap.ErasableHeap.push
    if round == 0:
        for i in 0..reps:
            s += m.len // timed: heap.MinMaxHeap.len
            if m.is_empty: s += 1 // timed: heap.MinMaxHeap.is_empty
            s += m.first() ?? 0 // timed: heap.MinMaxHeap.first
            s += m.last() ?? 0 // timed: heap.MinMaxHeap.last
            s += e.len // timed: heap.ErasableHeap.len
            if e.is_empty: s += 1 // timed: heap.ErasableHeap.is_empty
        // A loop of its own, so no sift's cache misses spill into the O(1) reads above.
        for i in 0..reps:
            let v = i * 31 % n
            e.push(n + v)
            e.remove(n + v) // timed: heap.ErasableHeap.remove
            s += e.peek() ?? 0 // timed: heap.ErasableHeap.peek
            var gm = m.take() // timed: heap.MinMaxHeap.take
            m = gm.take()
            var ge = e.take() // timed: heap.ErasableHeap.take
            e = ge.take()
    for _ in 0..n:
        s += m.pop_first() ?? 0 // timed: heap.MinMaxHeap.pop_first
        s += m.pop_last() ?? 0 // timed: heap.MinMaxHeap.pop_last
        s += e.pop() ?? 0 // timed: heap.ErasableHeap.pop
    s += m.len + e.len

// clear's cost does not depend on the length: an emptied heap keeps its storage.
var cm = MinMaxHeap<Int>()
var ce = ErasableHeap<Int>()
for i in 0..reps:
    cm.push(i)
    cm.clear() // timed: heap.MinMaxHeap.clear
    ce.push(i)
    ce.clear() // timed: heap.ErasableHeap.clear
print(s)
