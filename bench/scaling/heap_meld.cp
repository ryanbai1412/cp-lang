// n: 2048
// At n = 2048 the 4n heaps (about 400 KB together) still fit in one core's L2, so the
// O(log n) sifts are timed in one cache tier; more rounds keep the O(n) lines over the floor.
use std.heap.{MeldHeap, MeldHeaps}

let n: Int = read()
let reps = 1 << 17
let rounds = 64
let xs = [i * 7919 % n for i in 0..n]
var s = 0

for _ in 0..reps:
    s += MeldHeap<Int>().len // timed: heap.MeldHeap.new
    var mh = MeldHeaps<Int>() // timed: heap.MeldHeaps.new
for _ in 0..12 * rounds:
    s += MeldHeap<Int>.of(xs).len // timed: heap.MeldHeap.of
    var pool = MeldHeaps<Int>()
    s += pool.of(xs).len // timed: heap.MeldHeaps.of

// clear's cost does not depend on the length: an emptied heap keeps its storage.
var cl = MeldHeap<Int>()
for i in 0..reps:
    cl.push(i)
    cl.clear() // timed: heap.MeldHeap.clear

for round in 0..rounds:
    var mk = MeldHeap<Int>()
    for v in xs:
        mk.push(v) // timed: heap.MeldHeap.push
    if round == 0:
        for i in 0..reps:
            s += mk.len // timed: heap.MeldHeap.len
            if mk.is_empty: s += 1 // timed: heap.MeldHeap.is_empty
            s += mk.peek() ?? 0 // timed: heap.MeldHeap.peek
            s += mk.top() ?? 0 // timed: heap.MeldHeap.top
            mk.add_all(1) // timed: heap.MeldHeap.add_all
            var gk = mk.take() // timed: heap.MeldHeap.take
            mk = gk.take()
    mk.push_all(xs) // timed: heap.MeldHeap.push_all
    for _ in 0..n:
        s += mk.pop() ?? 0 // timed: heap.MeldHeap.pop
    s += mk.drain().len // timed: heap.MeldHeap.drain

for round in 0..rounds:
    var pool = MeldHeaps<Int>()
    var a = pool.of(xs)
    for i in 0..n:
        pool.push(mut a, i) // timed: heap.MeldHeaps.push
    if round == 0:
        for i in 0..reps:
            s += pool.top(a) ?? 0 // timed: heap.MeldHeaps.top
            s += a.len // timed: heap.Meld.len
            if a.is_empty: s += 1 // timed: heap.Meld.is_empty
            pool.add_all(a, 1) // timed: heap.MeldHeaps.add_all
    for i in 0..n:
        var b = pool.of([i])
        a = pool.meld(a, b) // timed: heap.MeldHeaps.meld
    pool.push_all(mut a, xs) // timed: heap.MeldHeaps.push_all
    for _ in 0..n:
        s += pool.pop(mut a) ?? 0 // timed: heap.MeldHeaps.pop
    s += pool.drain(mut a).len // timed: heap.MeldHeaps.drain
print(s)
