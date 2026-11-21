//! Binary heaps: `MinHeap<T>()` pops the smallest first, `MaxHeap<T>()` the largest.
use std.algebra.{Desc, desc}

/// A priority queue on `<`. `push` and `pop` are O(log n), `peek` is O(1); memory is one list.
/// For a mixed order put `desc(x)` in the entry: `MinHeap<(Int, Desc<V>)>()` pops the
/// smallest distance first and, among equals, the largest vertex.
pub struct MinHeap<T>:
    xs: List<T>

    /// Create an empty `MinHeap<T>`. O(1).
    pub fn new() -> MinHeap<T> = MinHeap([])

    /// Build a `MinHeap<T>` from `xs` via Floyd heapify. O(n).
    pub fn of(xs: List<T>) -> MinHeap<T>:
        var h = MinHeap(xs)
        for i in (0..h.xs.len / 2).rev:
            h.sift_down(i)
        h

    /// Number of elements currently in the heap. O(1).
    pub @get fn len(self) -> Int = xs.len
    /// Whether the heap is empty. O(1).
    pub @get fn is_empty(self) -> Bool = xs.is_empty

    /// The smallest element, or `none` if the heap is empty.
    pub @get fn peek(self) -> ?T = if xs.is_empty: none else: xs[0]

    /// Remove all elements from the heap. O(1).
    pub fn clear(mut self):
        xs = []

    /// Move the heap's contents out in O(1), leaving `self` empty.
    pub fn take(mut self) -> MinHeap<T>:
        let out = MinHeap(xs)
        xs = []
        out

    fn sift_down(mut self, start: Int):
        var i = start
        let n = xs.len
        while true:
            var best = i
            let l = 2 * i + 1
            let r = l + 1
            if l < n and xs[l] < xs[best]: best = l
            if r < n and xs[r] < xs[best]: best = r
            if best == i: break
            xs.swap(i, best)
            i = best

    /// Push `x` into the heap. O(log n).
    pub fn push(mut self, x: T):
        xs.push(x)
        var i = xs.len - 1
        while i > 0:
            let p = (i - 1) / 2
            if not (xs[i] < xs[p]): break
            xs.swap(i, p)
            i = p

    /// Remove and return the smallest element, or `none` if the heap is empty.
    pub fn pop(mut self) -> ?T:
        if xs.is_empty: return none
        xs.swap(0, xs.len - 1)
        let out = xs.pop()
        if not xs.is_empty: self.sift_down(0)
        out

/// A priority queue that pops the largest element first. Same costs as `MinHeap`.
pub struct MaxHeap<T>:
    h: MinHeap<Desc<T>>

    /// Create an empty `MaxHeap<T>`. O(1).
    pub fn new() -> MaxHeap<T> = MaxHeap(MinHeap<Desc<T>>())

    /// Build a `MaxHeap<T>` from `xs` via Floyd heapify. O(n).
    pub fn of(xs: List<T>) -> MaxHeap<T> = MaxHeap(MinHeap.of([desc(x) for x in xs]))

    /// Number of elements currently in the heap. O(1).
    pub @get fn len(self) -> Int = h.len
    /// Whether the heap is empty. O(1).
    pub @get fn is_empty(self) -> Bool = h.is_empty

    /// The largest element, or `none` if the heap is empty.
    pub @get fn peek(self) -> ?T = if h.is_empty: none else: h.peek!.x

    /// Remove all elements from the heap. O(1).
    pub fn clear(mut self):
        h.clear()

    /// Move the heap's contents out in O(1), leaving `self` empty.
    pub fn take(mut self) -> MaxHeap<T> = MaxHeap(h.take())

    /// Push `x` into the heap. O(log n).
    pub fn push(mut self, x: T):
        h.push(desc(x))

    /// Remove and return the largest element, or `none` if the heap is empty.
    pub fn pop(mut self) -> ?T = if h.is_empty: none else: h.pop()!.x

/// Min-heap supporting O(log n) arbitrary-value `remove(x)` via a lazy deletion heap.
/// `push` and `remove` are O(log n); `peek` and `pop` are O(log n) amortized; `len`, `is_empty`,
/// `clear`, and `take` are O(1).
pub struct ErasableMinHeap<T>:
    live: MinHeap<T>
    dead: MinHeap<T>

    /// Create an empty `ErasableMinHeap<T>`. O(1).
    pub fn new() -> ErasableMinHeap<T> = ErasableMinHeap(MinHeap<T>(), MinHeap<T>())

    /// Number of active elements in the heap. O(1).
    pub @get fn len(self) -> Int = live.len - dead.len

    /// Whether no active elements remain. O(1).
    pub @get fn is_empty(self) -> Bool = self.len == 0

    /// Remove all elements from the heap. O(1).
    pub fn clear(mut self):
        live.clear()
        dead.clear()

    /// Move the heap's contents out in O(1), leaving `self` empty.
    pub fn take(mut self) -> ErasableMinHeap<T> = ErasableMinHeap(live.take(), dead.take())

    fn prune(mut self):
        while not dead.is_empty and not live.is_empty and live.peek! == dead.peek!:
            live.pop()
            dead.pop()

    /// Push `x` into the heap. O(log n).
    pub fn push(mut self, x: T):
        live.push(x)

    /// Schedule one copy of `x` for lazy removal (`x` must be present). O(log n).
    pub fn remove(mut self, x: T):
        dead.push(x)

    /// Return the smallest active element, or `none` if empty. O(log n) amortized.
    pub fn peek(mut self) -> ?T:
        self.prune()
        live.peek

    /// Remove and return the smallest active element, or `none` if empty. O(log n) amortized.
    pub fn pop(mut self) -> ?T:
        self.prune()
        live.pop()

/// Max-heap supporting O(log n) arbitrary-value `remove(x)` via a lazy deletion heap.
pub struct ErasableMaxHeap<T>:
    h: ErasableMinHeap<Desc<T>>

    /// Create an empty `ErasableMaxHeap<T>`. O(1).
    pub fn new() -> ErasableMaxHeap<T> = ErasableMaxHeap(ErasableMinHeap<Desc<T>>())

    /// Number of active elements in the heap. O(1).
    pub @get fn len(self) -> Int = h.len

    /// Whether no active elements remain. O(1).
    pub @get fn is_empty(self) -> Bool = h.is_empty

    /// Remove all elements from the heap. O(1).
    pub fn clear(mut self): h.clear()

    /// Move the heap's contents out in O(1), leaving `self` empty.
    pub fn take(mut self) -> ErasableMaxHeap<T> = ErasableMaxHeap(h.take())

    /// Push `x` into the heap. O(log n).
    pub fn push(mut self, x: T): h.push(desc(x))

    /// Schedule one copy of `x` for lazy removal (`x` must be present). O(log n).
    pub fn remove(mut self, x: T): h.remove(desc(x))

    /// Return the largest active element, or `none` if empty. O(log n) amortized.
    pub fn peek(mut self) -> ?T = if let x = h.peek(): x.x else: none

    /// Remove and return the largest active element, or `none` if empty. O(log n) amortized.
    pub fn pop(mut self) -> ?T = if let x = h.pop(): x.x else: none

/// Domain-keyed min-heap over `D` supporting O(log n) `set`, `decrease`, `increase`, `remove`,
/// and `pop`, plus O(1) `contains`, `get`, and `peek`.
pub struct IndexedMinHeap<D, K>:
    heap: List<D>
    pos: [D]Int
    keys: [D]?K

    /// Create an empty `IndexedMinHeap<D, K>`. O(D.K).
    pub fn new() -> IndexedMinHeap<D, K> = IndexedMinHeap([], [D] _ => -1, [D] _ => none)

    /// Number of keys currently in the heap. O(1).
    pub @get fn len(self) -> Int = heap.len

    /// Whether the heap is empty. O(1).
    pub @get fn is_empty(self) -> Bool = heap.is_empty

    /// Whether `v` is currently in the heap. O(1).
    pub fn contains(self, v: D) -> Bool = pos[v] >= 0

    /// Current priority of `v`, or `none` if absent. O(1).
    pub fn get(self, v: D) -> ?K = keys[v]

    /// Pair `(v, key)` with the smallest key, or `none` if empty. O(1).
    pub @get fn peek(self) -> ?(D, K):
        if heap.is_empty: return none
        let v = heap[0]
        (v, keys[v]!)

    /// Remove all entries from the heap. O(len).
    pub fn clear(mut self):
        for v in heap:
            pos[v] = -1
            keys[v] = none
        heap = []

    fn swap_at(mut self, i: Int, j: Int):
        let vi = heap[i]
        let vj = heap[j]
        heap[i] = vj
        heap[j] = vi
        pos[vj] = i
        pos[vi] = j

    fn sift_up(mut self, start: Int):
        var i = start
        while i > 0:
            let p = (i - 1) / 2
            if not (keys[heap[i]]! < keys[heap[p]]!): break
            self.swap_at(i, p)
            i = p

    fn sift_down(mut self, start: Int):
        var i = start
        let n = heap.len
        while true:
            var best = i
            let l = 2 * i + 1
            let r = l + 1
            if l < n and keys[heap[l]]! < keys[heap[best]]!: best = l
            if r < n and keys[heap[r]]! < keys[heap[best]]!: best = r
            if best == i: break
            self.swap_at(i, best)
            i = best

    /// Insert `v` with key `k`, or update `v`'s key to `k`. O(log n).
    pub fn set(mut self, v: D, k: K):
        if pos[v] < 0:
            pos[v] = heap.len
            keys[v] = k
            heap.push(v)
            self.sift_up(pos[v])
        else:
            let old = keys[v]!
            keys[v] = k
            if k < old: self.sift_up(pos[v]) else: self.sift_down(pos[v])

    /// Insert `v` with `k`, or lower its key to `k` if `k` is smaller. Returns `true` if updated. O(log n).
    pub fn decrease(mut self, v: D, k: K) -> Bool:
        if pos[v] < 0 or k < keys[v]!:
            self.set(v, k)
            return true
        false

    /// Insert `v` with `k`, or raise its key to `k` if `k` is larger. Returns `true` if updated. O(log n).
    pub fn increase(mut self, v: D, k: K) -> Bool:
        if pos[v] < 0 or keys[v]! < k:
            self.set(v, k)
            return true
        false

    /// Remove `v` from the heap if present; returns its old key or `none`. O(log n).
    pub fn remove(mut self, v: D) -> ?K:
        let i = pos[v]
        if i < 0: return none
        let old = keys[v]!
        let last = heap.len - 1
        self.swap_at(i, last)
        heap.pop()
        pos[v] = -1
        keys[v] = none
        if i < heap.len:
            self.sift_up(i)
            self.sift_down(pos[heap[i]])
        old

    /// Remove and return `(v, key)` with the smallest key, or `none` if empty. O(log n).
    pub fn pop(mut self) -> ?(D, K):
        if heap.is_empty: return none
        let v = heap[0]
        let k = self.remove(v)!
        (v, k)

/// Domain-keyed max-heap over `D` supporting O(log n) `set`, `increase`, `decrease`, `remove`,
/// and `pop`, plus O(1) `contains`, `get`, and `peek`.
pub struct IndexedMaxHeap<D, K>:
    h: IndexedMinHeap<D, Desc<K>>

    /// Create an empty `IndexedMaxHeap<D, K>`. O(D.K).
    pub fn new() -> IndexedMaxHeap<D, K> = IndexedMaxHeap(IndexedMinHeap<D, Desc<K>>())

    /// Number of keys currently in the heap. O(1).
    pub @get fn len(self) -> Int = h.len

    /// Whether the heap is empty. O(1).
    pub @get fn is_empty(self) -> Bool = h.is_empty

    /// Whether `v` is currently in the heap. O(1).
    pub fn contains(self, v: D) -> Bool = h.contains(v)

    /// Current priority of `v`, or `none` if absent. O(1).
    pub fn get(self, v: D) -> ?K = if let k = h.get(v): k.x else: none

    /// Pair `(v, key)` with the largest key, or `none` if empty. O(1).
    pub @get fn peek(self) -> ?(D, K) = if let (v, k) = h.peek: (v, k.x) else: none

    /// Remove all entries from the heap. O(len).
    pub fn clear(mut self): h.clear()

    /// Insert `v` with key `k`, or update `v`'s key to `k`. O(log n).
    pub fn set(mut self, v: D, k: K): h.set(v, desc(k))

    /// Insert `v` with `k`, or raise its key to `k` if `k` is larger. Returns `true` if updated. O(log n).
    pub fn increase(mut self, v: D, k: K) -> Bool = h.decrease(v, desc(k))

    /// Insert `v` with `k`, or lower its key to `k` if `k` is smaller. Returns `true` if updated. O(log n).
    pub fn decrease(mut self, v: D, k: K) -> Bool = h.increase(v, desc(k))

    /// Remove `v` from the heap if present; returns its old key or `none`. O(log n).
    pub fn remove(mut self, v: D) -> ?K = if let k = h.remove(v): k.x else: none

    /// Remove and return `(v, key)` with the largest key, or `none` if empty. O(log n).
    pub fn pop(mut self) -> ?(D, K) = if let (v, k) = h.pop(): (v, k.x) else: none

@test
fn pops_in_sorted_order():
    var h = MinHeap<Int>()
    var g = MaxHeap<Int>()
    var xs = [rand(-50, 50) for _ in 0..40]
    for x in xs:
        h.push(x)
        g.push(x)
    assert h.len == 40 and g.len == 40
    sort(mut xs)
    for i in 0..40:
        assert h.peek == xs[i] and h.pop() == xs[i]
        assert g.peek == xs[39 - i] and g.pop() == xs[39 - i]
    assert h.pop() == none and h.is_empty and h.peek == none
    assert g.pop() == none and g.is_empty and g.peek == none

@test
fn desc_gives_a_mixed_order():
    var h = MinHeap<(Int, Desc<Str>)>()
    for (d, name) in [(2, "b"), (1, "a"), (2, "z"), (1, "m")]: h.push((d, desc(name)))
    var out: List<Str> = []
    while let (d, name) = h.pop(): out.push(name.x)
    assert out == ["m", "a", "z", "b"]

@test
fn clear_empties_the_heap():
    var h = MaxHeap<Int>()
    for x in [3, 1, 4]: h.push(x)
    h.clear()
    assert h.is_empty and h.len == 0
    h.push(7)
    assert h.pop() == 7

@test
fn take_and_small_into_large_swap():
    var a = MinHeap<Int>()
    var b = MinHeap<Int>()
    for x in [9, 2]: a.push(x)
    for x in [5, 1, 8, 3, 7]: b.push(x)
    if a.len < b.len: swap(mut a, mut b)
    while let x = b.pop(): a.push(x)
    assert b.is_empty and a.len == 7
    var moved = a.take()
    assert a.is_empty and moved.len == 7
    var got: List<Int> = []
    while let x = moved.pop(): got.push(x)
    assert got == [1, 2, 3, 5, 7, 8, 9]
    var hi = MaxHeap<Int>()
    for x in [4, 1, 9, 6, 2]: hi.push(x)
    var hi_moved = hi.take()
    assert hi.is_empty and hi_moved.pop() == 9 and hi_moved.pop() == 6

@test
fn heap_of_and_erasable_and_indexed_heaps():
    let xs = [rand(-50, 50) for _ in 0..35]
    var sorted_xs = copy(xs)
    sorted_xs.sort()
    var hmin = MinHeap<Int>.of(xs)
    var hmax = MaxHeap<Int>.of(xs)
    for i in 0..xs.len:
        assert hmin.pop() == sorted_xs[i]
        assert hmax.pop() == sorted_xs[xs.len - 1 - i]

    var emin = ErasableMinHeap<Int>()
    var emax = ErasableMaxHeap<Int>()
    for x in [5, 1, 9, 1, 7, 3]:
        emin.push(x)
        emax.push(x)
    emin.remove(1)
    emin.remove(7)
    emax.remove(9)
    emax.remove(1)
    assert emin.len == 4 and emin.peek() == 1 and emin.pop() == 1 and emin.pop() == 3
    assert emax.len == 4 and emax.peek() == 7 and emax.pop() == 7 and emax.pop() == 5
    emin.clear()
    assert emin.is_empty and emin.peek() == none and emin.pop() == none

    domain V = 0..8
    var ih = IndexedMinHeap<V, Int>()
    assert ih.is_empty and ih.peek == none
    assert ih.decrease(V(2), 10)
    assert not ih.decrease(V(2), 15)
    assert ih.decrease(V(2), 4)
    ih.set(V(5), 2)
    ih.set(V(1), 7)
    assert ih.len == 3 and ih.contains(V(5)) and ih.get(V(2)) == 4
    assert ih.peek == (V(5), 2)
    assert ih.pop() == (V(5), 2)
    assert not ih.contains(V(5)) and ih.get(V(5)) == none
    assert ih.increase(V(2), 9) and ih.get(V(2)) == 9
    assert ih.remove(V(1)) == 7 and ih.remove(V(1)) == none
    assert ih.pop() == (V(2), 9) and ih.is_empty

    var xh = IndexedMaxHeap<V, Int>()
    xh.set(V(0), 3)
    assert xh.increase(V(0), 8)
    assert not xh.increase(V(0), 5)
    xh.set(V(3), 12)
    assert xh.peek == (V(3), 12) and xh.pop() == (V(3), 12)
    assert xh.pop() == (V(0), 8) and xh.is_empty
