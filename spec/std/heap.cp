//! Binary heaps (round 5): `Heap<T>(by:)` pops the smallest key first and `Heap(by: desc)` the
//! largest. One type per shape: `Heap`, `ErasableHeap`, `IndexedHeap`, `MeldHeap`, `MinMaxHeap`.
//!
//! Ties: no heap here is stable. Among elements with equal keys under `by:`, which one `peek`
//! shows and `pop` returns is unspecified (but `peek` always shows the one `pop` returns next).
//! Without `by:`, equal elements are indistinguishable, so this never shows. For a fixed order
//! among equals, put the tie-breaker in the entry: `(d, v)`, or `(d, desc(v))`.
use std.algebra.{Desc, desc}

/// A priority queue on `<`. `push` and `pop` are O(log n), `peek` is O(1); memory is one list.
/// For a mixed order put `desc(x)` in the entry: `Heap<(Int, Desc<V>)>()` pops the
/// smallest distance first and, among equals, the largest vertex.
pub struct Heap<T>:
    xs: List<T>
    lt: ?((T, T) -> Bool)

    /// Create an empty `Heap<T>`, optionally ordered by key extractor `by:` (W7).
    ///
    /// With `by: f` the heap compares `f(x)`, and `f` is called on every comparison, so it
    /// should be cheap (a field, `desc`). Without `by:` it compares the elements themselves.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h: Heap<Int> = Heap(); h.push(3); h.push(1); h.pop() == Some(1)`
    /// example: `var h: Heap<Int> = Heap(by: desc); h.push(1); h.push(3); h.pop() == Some(3)`
    pub fn new<K>(by: ?(T -> K) = None) -> Heap<T>:
        if let f = by: return Heap(xs: [], lt: Some((a, b) => f(a) < f(b)))
        Heap(xs: [], lt: None)

    /// Build a `Heap<T>` from `xs` via Floyd heapify.
    ///
    /// `by:` means what it means in `new`; without it pops come out in the order of
    /// `xs.sorted()`, and `Heap.of(xs, by: desc)` is the O(n) max-heap (`MaxHeap.of`).
    ///
    /// cost: O(n)
    /// empty: an empty heap
    /// traps: never
    /// example: `var h = Heap.of([3, 1, 2]); h.pop() == Some(1) and h.len == 2`
    /// example: `var h = Heap.of([3, 1, 2], by: desc); h.pop() == Some(3)`
    pub fn of<K>(xs: List<T>, by: ?(T -> K) = None) -> Heap<T>:
        var h: Heap<T> = Heap.new(by: by)
        h.xs = xs
        var i = h.xs.len / 2
        while i > 0:
            i -= 1
            h.sift_down(i)
        h

    /// Number of elements currently in the heap.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Heap.of([3, 1]).len == 2`
    @get pub fn len(self) -> Int:
        self.xs.len
    /// Whether the heap is empty.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Heap.of([3]).is_empty == false`
    @get pub fn is_empty(self) -> Bool:
        self.xs.len == 0

    /// The smallest element, or `None` if the heap is empty.
    ///
    /// With `by: desc` the smallest key is the largest element.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// traps: never
    /// example: `Heap.of([3, 1, 2]).peek == Some(1)`
    @get pub fn peek(self) -> ?T:
        if self.xs.len == 0: return None
        Some(self.xs[0])

    /// Remove all elements from the heap.
    ///
    /// The order `by:` is kept.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h = Heap.of([1, 2]); h.clear(); h.pop() == None`
    pub fn clear(mut self):
        self.xs = []

    /// Move the heap's contents out in O(1), leaving `self` empty.
    ///
    /// Both heaps keep the order `by:`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h = Heap.of([2, 1]); let t = h.take(); h.is_empty and t.peek == Some(1)`
    pub fn take(mut self) -> Heap<T>:
        var out: List<T> = []
        swap(mut out, mut self.xs)
        Heap(xs: out, lt: self.lt)

    fn less(self, a: T, b: T) -> Bool:
        if let f = self.lt: return f(a, b)
        a < b

    // Moves a hole down instead of swapping: one write per level, as std::priority_queue.
    fn sift_down(mut self, start: Int):
        let n = self.xs.len
        let x = self.xs[start]
        var i = start
        while 2 * i + 1 < n:
            var c = 2 * i + 1
            if c + 1 < n and self.less(self.xs[c + 1], self.xs[c]): c += 1
            if not self.less(self.xs[c], x): break
            self.xs[i] = self.xs[c]
            i = c
        self.xs[i] = x

    /// Push `x` into the heap.
    ///
    /// cost: O(log n), amortized over list growth
    /// traps: never
    /// example: `var h = Heap.of([5]); h.push(2); h.peek == Some(2)`
    pub fn push(mut self, x: T):
        var i = self.xs.len
        self.xs.push(x)
        while i > 0:
            let p = (i - 1) / 2
            if not self.less(x, self.xs[p]): break
            self.xs[i] = self.xs[p]
            i = p
        self.xs[i] = x

    /// Remove and return the smallest element, or `None` if the heap is empty.
    ///
    /// cost: O(log n)
    /// empty: `None`
    /// traps: never
    /// example: `var h = Heap.of([2, 1]); h.pop() == Some(1) and h.pop() == Some(2) and h.pop() == None`
    pub fn pop(mut self) -> ?T:
        if self.xs.len == 0: return None
        let top = self.xs[0]
        let last = self.xs.pop()!
        if self.xs.len > 0:
            self.xs[0] = last
            self.sift_down(0)
        Some(top)

    /// Reserve backing storage for at least `cap` elements.
    ///
    /// cost: O(cap) when the storage grows, O(1) otherwise
    /// traps: never
    /// example: `var h = Heap.of([2]); h.reserve(100); h.peek == Some(2)`
    pub fn reserve(mut self, cap: Int):
        self.xs.reserve(cap)

    /// Push `x` and pop the minimum in one sift.
    ///
    /// Returns `x` itself, leaving the heap unchanged, when the heap is empty or the key of
    /// `x` is no greater than the top's; otherwise returns the old top and keeps `x`.
    ///
    /// cost: O(log n); O(1) when `x` is returned
    /// empty: returns `x`; the heap stays empty
    /// traps: never
    /// example: `var h = Heap.of([2, 5]); h.push_pop(3) == 2 and h.peek == Some(3)`
    /// example: `var h = Heap.of([2]); h.push_pop(1) == 1 and h.peek == Some(2)`
    pub fn push_pop(mut self, x: T) -> T:
        if self.xs.len == 0 or not self.less(self.xs[0], x): return x
        let top = self.xs[0]
        self.xs[0] = x
        self.sift_down(0)
        top

    /// Replace the minimum with `x` and return the old minimum (or `None` if empty).
    ///
    /// The old top is returned even when `x` is smaller than it. On an empty heap `x` is
    /// pushed and the result is `None`.
    ///
    /// cost: O(log n)
    /// empty: pushes `x` and returns `None`
    /// traps: never
    /// example: `var h = Heap.of([2, 5]); h.replace_top(9) == Some(2) and h.peek == Some(5)`
    /// example: `var h: Heap<Int> = Heap(); h.replace_top(4) == None and h.peek == Some(4)`
    pub fn replace_top(mut self, x: T) -> ?T:
        if self.xs.len == 0:
            self.xs.push(x)
            return None
        let top = self.xs[0]
        self.xs[0] = x
        self.sift_down(0)
        Some(top)

    /// Push all elements of `items`.
    ///
    /// cost: O(k log n) for `k` items
    /// empty: an empty `items` does nothing
    /// traps: never
    /// example: `var h = Heap.of([4]); h.push_all([3, 1]); h.peek == Some(1) and h.len == 3`
    pub fn push_all(mut self, items: List<T>):
        for x in items: self.push(x)

    /// Pop and yield all elements in ascending order, leaving the heap empty.
    ///
    /// Each step is one `pop`. A loop that stops early leaves the elements not yet yielded in
    /// the heap.
    ///
    /// cost: O(log n) per step
    /// empty: yields nothing
    /// order: ascending key order, the order `pop` would return them
    /// traps: never
    /// example: `var h = Heap.of([3, 1, 2]); h.drain().collect() == [1, 2, 3] and h.is_empty`
    pub fn drain(mut self) -> yields T:
        while let x = self.pop(): yield x

/// Min-heap supporting O(log n) arbitrary-value `remove(x)` via a lazy deletion heap.
/// `push` and `remove` are O(log n); `peek` and `pop` are O(log n) amortized; `len`, `is_empty`,
/// `clear`, and `take` are O(1).
pub struct ErasableHeap<T>:
    live: Heap<T>
    dead: Heap<T>

    /// Create an empty `ErasableHeap<T>`, optionally ordered by key extractor `by:` (W7).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h: ErasableHeap<Int> = ErasableHeap(); h.push(2); h.push(1); h.remove(1); h.pop() == Some(2)`
    pub fn new<K>(by: ?(T -> K) = None) -> ErasableHeap<T>:
        ErasableHeap(Heap.new(by: by), Heap.new(by: by))

    /// Number of active elements in the heap.
    ///
    /// Pushed elements minus removed ones, counting copies.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h: ErasableHeap<Int> = ErasableHeap(); h.push(1); h.push(1); h.remove(1); h.len == 1`
    @get pub fn len(self) -> Int:
        live.len - dead.len

    /// Whether no active elements remain.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h: ErasableHeap<Int> = ErasableHeap(); h.push(1); h.remove(1); h.is_empty`
    @get pub fn is_empty(self) -> Bool:
        live.len == dead.len

    /// Remove all elements from the heap.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h: ErasableHeap<Int> = ErasableHeap(); h.push(1); h.clear(); h.pop() == None`
    pub fn clear(mut self):
        live.clear()
        dead.clear()

    /// Move the heap's contents out in O(1), leaving `self` empty.
    ///
    /// Pending removals move with the contents.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h: ErasableHeap<Int> = ErasableHeap(); h.push(1); let t = h.take(); h.is_empty and t.len == 1`
    pub fn take(mut self) -> ErasableHeap<T>:
        let out = ErasableHeap(live.take(), dead.take())
        out

    fn prune(mut self):
        while dead.len > 0 and live.len > 0:
            let a = live.peek!
            let b = dead.peek!
            if live.less(b, a) or live.less(a, b): break
            live.pop()
            dead.pop()

    /// Push `x` into the heap.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `var h: ErasableHeap<Int> = ErasableHeap(); h.push(4); h.peek() == Some(4)`
    pub fn push(mut self, x: T):
        live.push(x)

    /// Schedule one copy of `x` for lazy removal (`x` must be present).
    ///
    /// One copy goes, even when several are present. The copy leaves at once as far as `len`,
    /// `peek` and `pop` can see; its storage is freed when it would reach the top. Removing a
    /// value that is not present corrupts the heap: debug builds trap when they find such a
    /// value surfacing.
    ///
    /// cost: O(log n)
    /// traps: never in release; debug traps when `x` is found not to be present
    /// example: `var h: ErasableHeap<Int> = ErasableHeap(); h.push(1); h.push(1); h.push(3); h.remove(1); h.pop() == Some(1) and h.pop() == Some(3)`
    pub fn remove(mut self, x: T):
        dead.push(x)

    /// Return the smallest active element, or `None` if empty.
    ///
    /// Takes `mut self` because it first discards removed values sitting on top.
    ///
    /// cost: O(log n) amortized
    /// empty: `None`
    /// traps: never
    /// example: `var h: ErasableHeap<Int> = ErasableHeap(); h.push(1); h.push(2); h.remove(1); h.peek() == Some(2)`
    pub fn peek(mut self) -> ?T:
        self.prune()
        live.peek

    /// Remove and return the smallest active element, or `None` if empty.
    ///
    /// cost: O(log n) amortized
    /// empty: `None`
    /// traps: never
    /// example: `var h: ErasableHeap<Int> = ErasableHeap(); h.push(5); h.push(2); h.pop() == Some(2) and h.len == 1`
    pub fn pop(mut self) -> ?T:
        self.prune()
        live.pop()

/// Domain-keyed min-heap over `D` supporting O(log n) `set`, `decrease`, `increase`, `remove`,
/// and `pop`, plus O(1) `contains`, `get`, and `peek`.
///
/// Each `v: D` is in the heap at most once, with one key `K`. Ties between equal keys go to the
/// smaller `v` in `D`'s order, so the pop order is fully determined. Memory is O(|D|).
pub struct IndexedHeap<D, K>:
    heap: List<D>
    pos: [D]Int
    keys: [D]?K

    /// Create an empty `IndexedHeap<D, K>`.
    ///
    /// cost: O(|D|)
    /// traps: never
    /// example: `domain V = 0..4; var h: IndexedHeap<V, Int> = IndexedHeap(); h.len == 0`
    pub fn new() -> IndexedHeap<D, K>:
        IndexedHeap([], [D] _ => -1, [D] _ => None)

    /// Number of keys currently in the heap.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..4; var h: IndexedHeap<V, Int> = IndexedHeap(); h.set(V(1), 5); h.set(V(1), 6); h.len == 1`
    @get pub fn len(self) -> Int:
        heap.len

    /// Whether the heap is empty.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..4; var h: IndexedHeap<V, Int> = IndexedHeap(); h.is_empty`
    @get pub fn is_empty(self) -> Bool:
        heap.len == 0

    /// Whether `v` is currently in the heap.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..4; var h: IndexedHeap<V, Int> = IndexedHeap(); h.set(V(2), 5); h.contains(V(2)) and not h.contains(V(3))`
    pub fn contains(self, v: D) -> Bool:
        pos[v] >= 0

    /// Current priority of `v`, or `None` if absent.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..4; var h: IndexedHeap<V, Int> = IndexedHeap(); h.set(V(2), 5); h.get(V(2)) == Some(5) and h.get(V(0)) == None`
    pub fn get(self, v: D) -> ?K:
        keys[v]

    /// Pair `(v, key)` with the smallest key, or `None` if empty.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// traps: never
    /// example: `domain V = 0..4; var h: IndexedHeap<V, Int> = IndexedHeap(); h.set(V(3), 1); h.set(V(1), 1); h.peek == Some((V(1), 1))`
    @get pub fn peek(self) -> ?(D, K):
        if heap.len == 0: return None
        let v = heap[0]
        Some((v, keys[v]!))

    /// Remove all entries from the heap.
    ///
    /// cost: O(len)
    /// traps: never
    /// example: `domain V = 0..4; var h: IndexedHeap<V, Int> = IndexedHeap(); h.set(V(1), 5); h.clear(); h.get(V(1)) == None`
    pub fn clear(mut self):
        for v in heap:
            pos[v] = -1
            keys[v] = None
        heap.clear()

    fn before(self, a: D, b: D) -> Bool:
        let ka = keys[a]!
        let kb = keys[b]!
        ka < kb or (not (kb < ka) and a.to_idx < b.to_idx)

    fn swap_at(mut self, i: Int, j: Int):
        let a = heap[i]
        let b = heap[j]
        heap[i] = b
        heap[j] = a
        pos[b] = i
        pos[a] = j

    fn sift_up(mut self, start: Int):
        var i = start
        while i > 0:
            let p = (i - 1) / 2
            if not self.before(heap[i], heap[p]): break
            self.swap_at(i, p)
            i = p

    fn sift_down(mut self, start: Int):
        let n = heap.len
        var i = start
        while 2 * i + 1 < n:
            var c = 2 * i + 1
            if c + 1 < n and self.before(heap[c + 1], heap[c]): c += 1
            if not self.before(heap[c], heap[i]): break
            self.swap_at(i, c)
            i = c

    /// Insert `v` with key `k`, or update `v`'s key to `k`.
    ///
    /// The new key may be larger or smaller than the old one.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `domain V = 0..4; var h: IndexedHeap<V, Int> = IndexedHeap(); h.set(V(1), 5); h.set(V(1), 9); h.get(V(1)) == Some(9)`
    pub fn set(mut self, v: D, k: K):
        keys[v] = Some(k)
        if pos[v] < 0:
            pos[v] = heap.len
            heap.push(v)
            self.sift_up(heap.len - 1)
        else:
            self.sift_up(pos[v])
            self.sift_down(pos[v])

    /// Insert `v` with `k`, or lower its key to `k` if `k` is smaller. Returns `true` if updated.
    ///
    /// The relaxation step of Dijkstra. An equal key is not smaller: it returns `false`.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `domain V = 0..4; var h: IndexedHeap<V, Int> = IndexedHeap(); h.decrease(V(1), 5) and not h.decrease(V(1), 5) and h.decrease(V(1), 3) and h.get(V(1)) == Some(3)`
    pub fn decrease(mut self, v: D, k: K) -> Bool:
        if let old = keys[v]:
            if not (k < old): return false
        self.set(v, k)
        true

    /// Insert `v` with `k`, or raise its key to `k` if `k` is larger. Returns `true` if updated.
    ///
    /// An equal key is not larger: it returns `false`.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `domain V = 0..4; var h: IndexedHeap<V, Int> = IndexedHeap(); h.increase(V(1), 5) and not h.increase(V(1), 2) and h.get(V(1)) == Some(5)`
    pub fn increase(mut self, v: D, k: K) -> Bool:
        if let old = keys[v]:
            if not (old < k): return false
        self.set(v, k)
        true

    /// Remove `v` from the heap if present; returns its old key or `None`.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `domain V = 0..4; var h: IndexedHeap<V, Int> = IndexedHeap(); h.set(V(1), 5); h.remove(V(1)) == Some(5) and h.remove(V(1)) == None`
    pub fn remove(mut self, v: D) -> ?K:
        let old = keys[v]
        if old == None: return None
        let i = pos[v]
        let last = heap.len - 1
        self.swap_at(i, last)
        heap.pop()
        pos[v] = -1
        keys[v] = None
        if i < heap.len:
            self.sift_up(i)
            self.sift_down(i)
        old

    /// Remove and return `(v, key)` with the smallest key, or `None` if empty.
    ///
    /// cost: O(log n)
    /// empty: `None`
    /// traps: never
    /// example: `domain V = 0..4; var h: IndexedHeap<V, Int> = IndexedHeap(); h.set(V(2), 5); h.set(V(0), 7); h.pop() == Some((V(2), 5)) and h.len == 1`
    /// example: `domain V = 0..4; var h: IndexedHeap<V, Int> = IndexedHeap(); h.set(V(3), 1); h.set(V(1), 1); h.pop() == Some((V(1), 1))`
    pub fn pop(mut self) -> ?(D, K):
        if heap.len == 0: return None
        let v = heap[0]
        let k = keys[v]!
        self.remove(v)
        Some((v, k))

/// Double-ended priority queue (min-max heap).
/// `first` and `last` are O(1); `push`, `pop_first`, and `pop_last` are O(log n).
fn on_min_level(i: Int) -> Bool:
    bit_width(i + 1) % 2 == 1

pub struct MinMaxHeap<T>:
    _xs: List<T>
    lt: ?((T, T) -> Bool)

    /// Create an empty `MinMaxHeap<T>`, optionally ordered by key extractor `by:` (W7).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h: MinMaxHeap<Int> = MinMaxHeap(); h.push(2); h.push(7); h.first == Some(2) and h.last == Some(7)`
    pub fn new<K>(by: ?(T -> K) = None) -> MinMaxHeap<T>:
        if let f = by: return MinMaxHeap(_xs: [], lt: Some((a, b) => f(a) < f(b)))
        MinMaxHeap(_xs: [], lt: None)

    /// Build a `MinMaxHeap<T>` from `xs` in linear time.
    ///
    /// The heap orders the elements themselves (there is no `by:`).
    ///
    /// cost: O(n)
    /// empty: an empty heap
    /// traps: never
    /// example: `var h = MinMaxHeap.of([3, 1, 4, 1, 5]); h.pop_last() == Some(5) and h.pop_first() == Some(1) and h.first == Some(1)`
    pub fn of(xs: List<T>) -> MinMaxHeap<T>:
        var h = MinMaxHeap(_xs: xs, lt: None)
        var i = xs.len / 2
        while i > 0:
            i -= 1
            h.trickle_down(i)
        h

    /// Reserve backing storage for at least `cap` elements.
    ///
    /// cost: O(cap) when the storage grows, O(1) otherwise
    /// traps: never
    /// example: `var h = MinMaxHeap.of([2]); h.reserve(100); h.first == Some(2)`
    pub fn reserve(mut self, cap: Int):
        _xs.reserve(cap)

    /// Number of elements currently in the heap.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `MinMaxHeap.of([1, 1]).len == 2`
    @get pub fn len(self) -> Int:
        _xs.len

    /// Whether the heap is empty.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `MinMaxHeap.of([1]).is_empty == false`
    @get pub fn is_empty(self) -> Bool:
        _xs.len == 0

    /// The smallest element, or `None` if empty.
    ///
    /// With one element, `first` and `last` are the same element.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// ties: unspecified among equal keys under `by:`; `pop_first` returns the one shown
    /// traps: never
    /// example: `MinMaxHeap.of([3, 1, 2]).first == Some(1)`
    /// example: `MinMaxHeap.of([7]).first == Some(7)`
    @get pub fn first(self) -> ?T:
        if _xs.len == 0: return None
        Some(_xs[0])

    /// The largest element, or `None` if empty.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// ties: unspecified among equal keys under `by:`; `pop_last` returns the one shown
    /// traps: never
    /// example: `MinMaxHeap.of([3, 1, 2]).last == Some(3)`
    /// example: `MinMaxHeap.of([7]).last == Some(7)`
    @get pub fn last(self) -> ?T:
        if _xs.len == 0: return None
        Some(_xs[self.max_at()])

    /// Insert `x`.
    ///
    /// cost: O(log n), amortized over list growth
    /// traps: never
    /// example: `var h = MinMaxHeap.of([2, 5]); h.push(9); h.last == Some(9) and h.first == Some(2)`
    pub fn push(mut self, x: T):
        _xs.push(x)
        let i = _xs.len - 1
        if i == 0: return
        let p = (i - 1) / 2
        if on_min_level(i):
            if self.less(_xs[p], _xs[i]):
                self.swap(i, p)
                self.bubble_up(p, false)
            else: self.bubble_up(i, true)
        else:
            if self.less(_xs[i], _xs[p]):
                self.swap(i, p)
                self.bubble_up(p, true)
            else: self.bubble_up(i, false)

    /// Remove and return the smallest element, or `None` if empty.
    ///
    /// cost: O(log n)
    /// empty: `None`
    /// traps: never
    /// example: `var h = MinMaxHeap.of([2, 5, 3]); h.pop_first() == Some(2) and h.first == Some(3)`
    pub fn pop_first(mut self) -> ?T:
        if _xs.len == 0: return None
        let x = _xs[0]
        self.remove_at(0)
        Some(x)

    /// Remove and return the largest element, or `None` if empty.
    ///
    /// cost: O(log n)
    /// empty: `None`
    /// traps: never
    /// example: `var h = MinMaxHeap.of([2, 5, 3]); h.pop_last() == Some(5) and h.last == Some(3)`
    pub fn pop_last(mut self) -> ?T:
        if _xs.len == 0: return None
        let i = self.max_at()
        let x = _xs[i]
        self.remove_at(i)
        Some(x)

    /// Remove all elements.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h = MinMaxHeap.of([2]); h.clear(); h.first == None`
    pub fn clear(mut self):
        _xs.clear()

    /// Move contents out in O(1), leaving `self` empty.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h = MinMaxHeap.of([2]); let t = h.take(); h.is_empty and t.first == Some(2)`
    pub fn take(mut self) -> MinMaxHeap<T>:
        let out = MinMaxHeap(_xs: _xs, lt: lt)
        _xs = []
        out

    fn less(self, a: T, b: T) -> Bool:
        if let f = lt: return f(a, b)
        a < b

    fn swap(mut self, i: Int, j: Int):
        let t = _xs[i]
        _xs[i] = _xs[j]
        _xs[j] = t

    fn max_at(self) -> Int:
        if _xs.len <= 2: return _xs.len - 1
        if self.less(_xs[1], _xs[2]): 2 else: 1

    fn bubble_up(mut self, start: Int, min_side: Bool):
        var i = start
        while i > 2:
            let g = ((i - 1) / 2 - 1) / 2
            let better = if min_side: self.less(_xs[i], _xs[g]) else: self.less(_xs[g], _xs[i])
            if not better: break
            self.swap(i, g)
            i = g

    fn remove_at(mut self, i: Int):
        let n = _xs.len - 1
        _xs[i] = _xs[n]
        _xs.pop()
        if i < n: self.trickle_down(i)

    fn trickle_down(mut self, start: Int):
        let min_side = on_min_level(start)
        let n = _xs.len
        var i = start
        while 2 * i + 1 < n:
            var m = 2 * i + 1
            for c in [2 * i + 2, 4 * i + 3, 4 * i + 4, 4 * i + 5, 4 * i + 6]:
                if c < n:
                    let better = if min_side: self.less(_xs[c], _xs[m]) else: self.less(_xs[m], _xs[c])
                    if better: m = c
            let wins = if min_side: self.less(_xs[m], _xs[i]) else: self.less(_xs[i], _xs[m])
            if not wins: break
            self.swap(m, i)
            if m <= 2 * i + 2: break
            let p = (m - 1) / 2
            let flip = if min_side: self.less(_xs[p], _xs[m]) else: self.less(_xs[m], _xs[p])
            if flip: self.swap(m, p)
            i = m


/// Meldable min-heap (skew / leftist heap) with O(log n) `meld` and O(1) lazy offset `add_all`.
/// `peek` / `top` is O(1); `push`, `pop`, and `meld` are O(log n).
pub struct MeldHeap<T>:
    _len: Int = 0
    _root: Int = -1
    _keys: List<T> = []
    _add: List<?T> = []
    _kids: List<(Int, Int)> = []
    _rank: List<Int> = []
    lt: ?((T, T) -> Bool) = None

    /// Create an empty `MeldHeap<T>`, optionally ordered by key extractor `by:` (W7).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h: MeldHeap<Int> = MeldHeap(); h.push(3); h.push(1); h.pop() == Some(1)`
    pub fn new<K>(by: ?(T -> K) = None) -> MeldHeap<T>:
        if let f = by: return MeldHeap(lt: Some((a, b) => f(a) < f(b)))
        MeldHeap()

    /// Build a `MeldHeap<T>` from `xs`.
    ///
    /// The heap orders the elements themselves (there is no `by:`).
    ///
    /// cost: O(n)
    /// empty: an empty heap
    /// traps: never
    /// example: `MeldHeap.of([4, 2, 3]).peek == Some(2)`
    pub fn of(xs: List<T>) -> MeldHeap<T>:
        var h: MeldHeap<T> = MeldHeap()
        var q = [h.node(x) for x in xs]
        var i = 0
        while i + 1 < q.len:
            q.push(h.merge(q[i], q[i + 1]))
            i += 2
        if q.len > 0: h._root = q[q.len - 1]
        h._len = xs.len
        h

    /// Number of elements currently in the heap.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `MeldHeap.of([4, 2]).len == 2`
    @get pub fn len(self) -> Int = _len

    /// Whether the heap is empty.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `MeldHeap.of([4]).is_empty == false`
    @get pub fn is_empty(self) -> Bool = _len == 0

    /// The smallest element, or `None` if empty.
    ///
    /// The value includes every `add_all` applied since it was pushed.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// traps: never
    /// example: `var h = MeldHeap.of([4, 2]); h.add_all(1); h.peek == Some(3)`
    @get pub fn peek(self) -> ?T = if _root < 0: None else: Some(_keys[_root])

    /// The smallest element, or `None` if empty (alias for `peek`).
    ///
    /// cost: O(1)
    /// empty: `None`
    /// ties: as `peek`: unspecified among equal keys under `by:`
    /// traps: never
    /// see: `peek`
    /// example: `MeldHeap.of([4, 2]).top == Some(2)`
    @get pub fn top(self) -> ?T = self.peek

    /// Push `x` into the heap.
    ///
    /// `x` is stored as given: earlier `add_all` offsets do not apply to it.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `var h = MeldHeap.of([1, 5]); h.add_all(10); h.push(3); h.drain() == [3, 11, 15]`
    pub fn push(mut self, x: T):
        let t = self.node(x)
        _root = self.merge(_root, t)
        _len += 1

    /// Push all elements of `items`.
    ///
    /// cost: O(k log n) for `k` items
    /// empty: an empty `items` does nothing
    /// traps: never
    /// example: `var h = MeldHeap.of([4]); h.push_all([3, 1]); h.drain() == [1, 3, 4]`
    pub fn push_all(mut self, items: List<T>):
        for x in items: self.push(x)

    /// Remove and return the smallest element, or `None` if empty.
    ///
    /// cost: O(log n)
    /// empty: `None`
    /// traps: never
    /// example: `var h = MeldHeap.of([4, 2]); h.pop() == Some(2) and h.pop() == Some(4) and h.pop() == None`
    pub fn pop(mut self) -> ?T:
        if _root < 0: return None
        let t = _root
        self.push_down(t)
        let (a, b) = _kids[t]
        _root = self.merge(a, b)
        _len -= 1
        Some(_keys[t])

    /// Meld `other` into `self` (consuming `other`).
    ///
    /// Every element keeps the value its own heap reported: pending `add_all` offsets apply only
    /// to the elements they were added to.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `var a = MeldHeap.of([4, 2]); a.meld(MeldHeap.of([3, 1])); a.drain() == [1, 2, 3, 4]`
    pub fn meld(mut self, other: MeldHeap<T>):
        if other._root < 0: return
        let base = _keys.len
        for i in 0..other._keys.len:
            let (a, b) = other._kids[i]
            _keys.push(other._keys[i])
            _add.push(other._add[i])
            _kids.push((if a < 0: a else: a + base, if b < 0: b else: b + base))
            _rank.push(other._rank[i])
        _root = self.merge(_root, other._root + base)
        _len += other._len

    /// Meld `other` into `self`, leaving `other` empty.
    ///
    /// `meld` for an `other` you keep using: small-to-large merging over a tree.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `var a = MeldHeap.of([4]); var b = MeldHeap.of([1]); a.absorb(mut b); a.peek == Some(1) and b.is_empty`
    pub fn absorb(mut self, mut other: MeldHeap<T>):
        self.meld(other)
        other.clear()

    /// Add `delta` lazily to every element (for additive types).
    ///
    /// Only the elements in the heap now change; later pushes do not. Adding one value to
    /// every element keeps their order; with `by:`, the key must keep that order too, or later
    /// pops are unspecified.
    ///
    /// cost: O(1)
    /// empty: does nothing
    /// traps: never (overflow wraps in release, traps in debug)
    /// example: `var h = MeldHeap.of([1, 5]); h.add_all(10); h.pop() == Some(11)`
    pub fn add_all(mut self, delta: T):
        if _root < 0: return
        _keys[_root] = _keys[_root] + delta
        _add[_root] = Some(plus(_add[_root], delta))

    /// Remove all elements.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h = MeldHeap.of([1]); h.clear(); h.pop() == None`
    pub fn clear(mut self):
        _len = 0
        _root = -1
        _keys = []
        _add = []
        _kids = []
        _rank = []

    /// Move contents out in O(1), leaving `self` empty.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h = MeldHeap.of([1]); let t = h.take(); h.is_empty and t.peek == Some(1)`
    pub fn take(mut self) -> MeldHeap<T>:
        let out = MeldHeap(_len: _len, _root: _root, _keys: _keys, _add: _add, _kids: _kids, _rank: _rank, lt: lt)
        self.clear()
        out

    /// Pop and return all elements in ascending order, leaving `self` empty.
    ///
    /// cost: O(n log n)
    /// empty: `[]`
    /// order: ascending key order, the order `pop` would return them; a new `List`, keyed from 0
    /// traps: never
    /// example: `var h = MeldHeap.of([3, 1, 2]); h.drain() == [1, 2, 3] and h.is_empty`
    pub fn drain(mut self) -> List<T>:
        var out: List<T> = []
        while _root >= 0: out.push(self.pop()!)
        out

    fn less(self, a: T, b: T) -> Bool:
        if let f = lt: return f(a, b)
        a < b

    fn node(mut self, x: T) -> Int:
        _keys.push(x)
        _add.push(None)
        _kids.push((-1, -1))
        _rank.push(1)
        _keys.len - 1

    fn rank_of(self, t: Int) -> Int = if t < 0: 0 else: _rank[t]

    // A node's pending add is already in its own key; it still owes its children.
    fn push_down(mut self, t: Int):
        if let d = _add[t]:
            let (a, b) = _kids[t]
            for c in [a, b]:
                if c >= 0:
                    _keys[c] = _keys[c] + d
                    _add[c] = Some(plus(_add[c], d))
            _add[t] = None

    fn merge(mut self, a: Int, b: Int) -> Int:
        if a < 0: return b
        if b < 0: return a
        let x = if self.less(_keys[b], _keys[a]): b else: a
        let y = if x == a: b else: a
        self.push_down(x)
        let (l, r) = _kids[x]
        let m = self.merge(r, y)
        _kids[x] = if self.rank_of(l) < self.rank_of(m): (m, l) else: (l, m)
        _rank[x] = self.rank_of(_kids[x].1) + 1
        x


fn plus<T>(a: ?T, d: T) -> T:
    if let x = a: return x + d
    d

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
    assert h.pop() == None and h.is_empty and h.peek == None
    assert g.pop() == None and g.is_empty and g.peek == None

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
fn heap_of_and_erasable_and_indexed_heaps() = todo

/// `Heap<T>()`: pops the smallest first.
pub type MinHeap<T> = Heap<T>

/// `Heap<T>(by: desc)`: pops the largest first.
pub type MaxHeap<T> = Heap<T>

/// A keyed frontier where each key pops exactly once. `offer(k, p)` keeps the best priority seen
/// for a key that has not popped yet and is ignored after it has; `pop` returns the best key
/// and settles it. The shape of Dijkstra, Prim and "settle answers from the largest down",
/// without a side array of done flags or a lazy-deletion `continue`. O(log n) per offer and pop.
///
/// `best` picks the better of two priorities and must return one of its arguments, like `min`
/// and `max`; with any other function the popped priorities are unspecified. Keys are hashed.
pub struct Settle<K, P>(best: (P, P) -> P = min):
    cur: HashMap<K, P> = HashMap()
    done: HashMap<K, Bool> = HashMap()
    q: List<(P, K)> = []

    fn better(self, a: Int, b: Int) -> Bool:
        let (x, y) = (q[a].0, q[b].0)
        x != y and best(x, y) == x

    fn sift_up(mut self, i0: Int):
        var i = i0
        while i > 0 and self.better(i, (i - 1) / 2):
            q.swap(i, (i - 1) / 2)
            i = (i - 1) / 2

    fn sift_down(mut self):
        var i = 0
        while true:
            var c = 2 * i + 1
            if c >= q.len: return
            if c + 1 < q.len and self.better(c + 1, c): c += 1
            if not self.better(c, i): return
            q.swap(i, c)
            i = c

    /// Offer priority `p` for key `k`: an unsettled key keeps `best(old, p)` (or `p` when it
    /// had None); a settled key ignores the offer.
    ///
    /// cost: O(log n) amortized, `n` the offers so far
    /// traps: never
    /// example: `var s: Settle<Char, Int> = Settle(); s.offer('a', 5); s.offer('a', 2); s.offer('a', 4); s.get('a') == Some(2)`
    pub fn offer(mut self, k: K, p: P):
        if done.get(k) != None: return
        if let old = cur.get(k):
            let b = best(old, p)
            if b == old: return
            cur[k] = b
        else:
            cur[k] = p
        q.push((cur[k], k))
        self.sift_up(q.len - 1)
    /// Settle and return the unsettled key whose priority is best, with that priority, or
    /// `None` when every offered key has settled.
    ///
    /// cost: O(log n) amortized, `n` the offers so far
    /// empty: `None`
    /// ties: unspecified among keys with equal priorities, as in `Heap`
    /// traps: never
    /// example: `var s: Settle<Char, Int> = Settle(); s.offer('a', 5); s.offer('b', 3); s.pop() == Some(('b', 3)) and s.settled('b')`
    /// example: `var s: Settle<Char, Int> = Settle(); s.offer('b', 3); s.pop(); s.offer('b', 1); s.pop() == None`
    /// example: `var s: Settle<Char, Int> = Settle(best: max); s.offer('a', 5); s.offer('b', 3); s.pop() == Some(('a', 5))`
    pub fn pop(mut self) -> ?(K, P):
        while q.len > 0:
            let (p, k) = q[0]
            q.swap(0, q.len - 1)
            let _ = q.pop()
            self.sift_down()
            if done.get(k) == None and cur.get(k) == Some(p):
                done[k] = true
                return Some((k, p))
        None
    /// Whether `k` has popped.
    ///
    /// cost: O(1) expected
    /// traps: never
    /// example: `var s: Settle<Char, Int> = Settle(); s.offer('a', 1); not s.settled('a') and s.pop() == Some(('a', 1)) and s.settled('a')`
    pub fn settled(self, k: K) -> Bool = done.get(k) != None
    /// The best priority offered for `k` so far (for a settled key, the one it popped with),
    /// or `None` if `k` was never offered.
    ///
    /// cost: O(1) expected
    /// traps: never
    /// example: `var s: Settle<Char, Int> = Settle(); s.offer('a', 4); s.get('a') == Some(4) and s.get('b') == None`
    pub fn get(self, k: K) -> ?P = cur.get(k)
