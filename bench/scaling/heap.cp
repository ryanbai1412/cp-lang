use std.heap.{MinMaxHeap, ErasableHeap, MeldHeap, MeldHeaps, IndexedHeap, Settle}

let n: Int = read()
domain V = 0..n
let reps = 1 << 17
let rounds = 8
let xs = [i * 7919 % n for i in 0..n]
var s = 0

for _ in 0..reps:
    s += Heap<Int>().len // timed: heap.Heap.new
    s += MaxHeap<Int>().len // timed: heap.MaxHeap.new
    s += MinMaxHeap<Int>().len // timed: heap.MinMaxHeap.new
    s += ErasableHeap<Int>().len // timed: heap.ErasableHeap.new
    s += MeldHeap<Int>().len // timed: heap.MeldHeap.new
    var mh = MeldHeaps<Int>() // timed: heap.MeldHeaps.new
for _ in 0..12 * rounds:
    s += Heap<Int>.of(xs).len // timed: heap.Heap.of
    s += MaxHeap<Int>.of(xs).len // timed: heap.MaxHeap.of
    s += MinMaxHeap<Int>.of(xs).len // timed: heap.MinMaxHeap.of
    s += MeldHeap<Int>.of(xs).len // timed: heap.MeldHeap.of
    var pool = MeldHeaps<Int>()
    s += pool.of(xs).len // timed: heap.MeldHeaps.of
// Storage grows by moving the elements already there.
for _ in 0..128 * rounds:
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
for _ in 0..6 * rounds:
    var p = Heap<Int>()
    p.push_all(xs) // timed: heap.Heap.push_all
    var px = MaxHeap<Int>()
    px.push_all(xs) // timed: heap.MaxHeap.push_all
    s += p.len + px.len

for round in 0..rounds:
    var h = Heap<Int>()
    var x = MaxHeap<Int>()
    var m = MinMaxHeap<Int>()
    var e = ErasableHeap<Int>()
    for v in xs:
        h.push(v) // timed: heap.Heap.push
        x.push(v) // timed: heap.MaxHeap.push
        m.push(v) // timed: heap.MinMaxHeap.push
        e.push(v) // timed: heap.ErasableHeap.push
    if round == 0:
        for i in 0..reps:
            s += h.len // timed: heap.Heap.len
            if h.is_empty: s += 1 // timed: heap.Heap.is_empty
            s += h.peek() ?? 0 // timed: heap.Heap.peek
            s += x.len // timed: heap.MaxHeap.len
            if x.is_empty: s += 1 // timed: heap.MaxHeap.is_empty
            s += x.peek() ?? 0 // timed: heap.MaxHeap.peek
            s += m.len // timed: heap.MinMaxHeap.len
            if m.is_empty: s += 1 // timed: heap.MinMaxHeap.is_empty
            s += m.first() ?? 0 // timed: heap.MinMaxHeap.first
            s += m.last() ?? 0 // timed: heap.MinMaxHeap.last
            s += e.len // timed: heap.ErasableHeap.len
            if e.is_empty: s += 1 // timed: heap.ErasableHeap.is_empty
            let v = i * 31 % n
            s += h.push_pop(v) // timed: heap.Heap.push_pop
            s += h.replace_top(v) ?? 0 // timed: heap.Heap.replace_top
            s += x.push_pop(v) // timed: heap.MaxHeap.push_pop
            s += x.replace_top(v) ?? 0 // timed: heap.MaxHeap.replace_top
            e.push(n + v)
            e.remove(n + v) // timed: heap.ErasableHeap.remove
            s += e.peek() ?? 0 // timed: heap.ErasableHeap.peek
            var g = h.take() // timed: heap.Heap.take
            h = g.take()
            var gx = x.take() // timed: heap.MaxHeap.take
            x = gx.take()
            var gm = m.take() // timed: heap.MinMaxHeap.take
            m = gm.take()
            var ge = e.take() // timed: heap.ErasableHeap.take
            e = ge.take()
    for _ in 0..n:
        s += h.pop() ?? 0 // timed: heap.Heap.pop
        s += x.pop() ?? 0 // timed: heap.MaxHeap.pop
        s += m.pop_first() ?? 0 // timed: heap.MinMaxHeap.pop_first
        s += m.pop_last() ?? 0 // timed: heap.MinMaxHeap.pop_last
        s += e.pop() ?? 0 // timed: heap.ErasableHeap.pop
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
    s += m.len + e.len

// clear's cost does not depend on the length: an emptied heap keeps its storage.
var c = Heap<Int>()
var cx = MaxHeap<Int>()
var cm = MinMaxHeap<Int>()
var ce = ErasableHeap<Int>()
var cl = MeldHeap<Int>()
for i in 0..reps:
    c.push(i)
    c.clear() // timed: heap.Heap.clear
    cx.push(i)
    cx.clear() // timed: heap.MaxHeap.clear
    cm.push(i)
    cm.clear() // timed: heap.MinMaxHeap.clear
    ce.push(i)
    ce.clear() // timed: heap.ErasableHeap.clear
    cl.push(i)
    cl.clear() // timed: heap.MeldHeap.clear

for _ in 0..24 * rounds:
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
