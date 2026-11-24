// n: 2048
// At n = 2048 the 4n heaps (about 400 KB together) still fit in one core's L2, so the
// O(log n) sifts are timed in one cache tier; more rounds keep the O(n) lines over the floor.
let n: Int = read()
let reps = 1 << 17
let rounds = 64
let xs = [i * 7919 % n for i in 0..n]
var s = 0

for _ in 0..reps:
    s += Heap<Int>().len // timed: heap.Heap.new
    s += MaxHeap<Int>().len // timed: heap.MaxHeap.new
for _ in 0..12 * rounds:
    s += Heap<Int>.of(xs).len // timed: heap.Heap.of
    s += MaxHeap<Int>.of(xs).len // timed: heap.MaxHeap.of
for _ in 0..6 * rounds:
    var p = Heap<Int>()
    p.push_all(xs) // timed: heap.Heap.push_all
    var px = MaxHeap<Int>()
    px.push_all(xs) // timed: heap.MaxHeap.push_all
    s += p.len + px.len

for round in 0..rounds:
    var h = Heap<Int>()
    var x = MaxHeap<Int>()
    for v in xs:
        h.push(v) // timed: heap.Heap.push
        x.push(v) // timed: heap.MaxHeap.push
    if round == 0:
        for i in 0..reps:
            s += h.len // timed: heap.Heap.len
            if h.is_empty: s += 1 // timed: heap.Heap.is_empty
            s += h.peek() ?? 0 // timed: heap.Heap.peek
            s += x.len // timed: heap.MaxHeap.len
            if x.is_empty: s += 1 // timed: heap.MaxHeap.is_empty
            s += x.peek() ?? 0 // timed: heap.MaxHeap.peek
        // A loop of its own, so no sift's cache misses spill into the O(1) reads above.
        for i in 0..reps:
            let v = i * 31 % n
            s += h.push_pop(v) // timed: heap.Heap.push_pop
            s += h.replace_top(v) ?? 0 // timed: heap.Heap.replace_top
            s += x.push_pop(v) // timed: heap.MaxHeap.push_pop
            s += x.replace_top(v) ?? 0 // timed: heap.MaxHeap.replace_top
            var g = h.take() // timed: heap.Heap.take
            h = g.take()
            var gx = x.take() // timed: heap.MaxHeap.take
            x = gx.take()
    for _ in 0..n:
        s += h.pop() ?? 0 // timed: heap.Heap.pop
        s += x.pop() ?? 0 // timed: heap.MaxHeap.pop
    h.push_all(xs)
    x.push_all(xs)
    // `drain` costs a pop per element it yields: each loop takes one.
    for _ in 0..n:
        for v in h.drain(): // timed: heap.Heap.drain
            s += v
            break
        for v in x.drain(): // timed: heap.MaxHeap.drain
            s += v
            break

// clear's cost does not depend on the length: an emptied heap keeps its storage.
var c = Heap<Int>()
var cx = MaxHeap<Int>()
for i in 0..reps:
    c.push(i)
    c.clear() // timed: heap.Heap.clear
    cx.push(i)
    cx.clear() // timed: heap.MaxHeap.clear
print(s)
