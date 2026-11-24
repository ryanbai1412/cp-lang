// n: 2048
// At n = 2048 the 4n heaps (about 400 KB together) still fit in one core's L2, so the
// O(log n) sifts are timed in one cache tier; more rounds keep the O(n) lines over the floor.
use std.heap.{IndexedHeap, Settle}

let n: Int = read()
domain V = 0..n
let reps = 1 << 17
let rounds = 64
var s = 0

for _ in 0..48 * rounds:
    var ih = IndexedHeap<V, Int>() // timed: heap.IndexedHeap.new
    for i in 0..n:
        ih.set(V(i), i)
    ih.clear() // timed: heap.IndexedHeap.clear
for round in 0..rounds:
    var ih = IndexedHeap<V, Int>()
    for i in 0..n:
        ih.set(V(i * 7919 % n), i * 13 % n) // timed: heap.IndexedHeap.set
    if round == 0:
        for i in 0..reps:
            let v = V(i % n)
            s += ih.len // timed: heap.IndexedHeap.len
            if ih.is_empty: s += 1 // timed: heap.IndexedHeap.is_empty
            if ih.contains(v): s += 1 // timed: heap.IndexedHeap.contains
            s += ih.get(v) ?? 0 // timed: heap.IndexedHeap.get
            s += (ih.peek() ?? (v, 0)).1 // timed: heap.IndexedHeap.peek
            if ih.decrease(v, (ih.get(v) ?? 0) - 1): s += 1 // timed: heap.IndexedHeap.decrease
            if ih.increase(v, (ih.get(v) ?? 0) + 1): s += 1 // timed: heap.IndexedHeap.increase
    for i in 0..n / 2:
        s += ih.remove(V(i * 2)) ?? 0 // timed: heap.IndexedHeap.remove
    for _ in 0..n / 4:
        s += (ih.pop() ?? (V(0), 0)).1 // timed: heap.IndexedHeap.pop

var got = 0
for round in 0..rounds:
    var st = Settle<V, Int>()
    for i in 0..n:
        st.offer(V(i * 7919 % n), i % 1000) // timed: heap.Settle.offer
    while let (k, p) = st.pop(): // timed: heap.Settle.pop
        got += p
    if round == 0:
        for i in 0..reps:
            if st.settled(V(i % n)): s += 1 // timed: heap.Settle.settled
            s += st.get(V(i % n)) ?? 0 // timed: heap.Settle.get
print(s, got)
