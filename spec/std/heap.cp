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

    /// Create an empty `Heap<T>`, optionally ordered by key extractor `by:` (W7). O(1).
    ///
    /// With `by: f` the heap compares `f(x)`, and `f` is called on every comparison, so it
    /// should be cheap (a field, `desc`). Without `by:` it compares the elements themselves.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h: Heap<Int> = Heap.new(); h.push(3); h.push(1); h.pop() == Some(1)`
    /// example: `var h: Heap<Int> = Heap.new(by: desc); h.push(1); h.push(3); h.pop() == Some(3)`
    pub fn new<K>(by: ?(T -> K) = None) -> Heap<T> = todo

    /// Build a `Heap<T>` from `xs` via Floyd heapify. O(n).
    ///
    /// The heap orders the elements themselves (there is no `by:`); pops come out in the order
    /// of `xs.sorted()`.
    ///
    /// cost: O(n)
    /// empty: an empty heap
    /// traps: never
    /// example: `var h = Heap.of([3, 1, 2]); h.pop() == Some(1) and h.len == 2`
    pub fn of(xs: List<T>) -> Heap<T> = todo

    /// Number of elements currently in the heap. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Heap.of([3, 1]).len == 2`
    pub @get fn len(self) -> Int = todo
    /// Whether the heap is empty. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Heap.of([3]).is_empty == false`
    pub @get fn is_empty(self) -> Bool = todo

    /// The smallest element, or `None` if the heap is empty.
    ///
    /// With `by: desc` the smallest key is the largest element.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// traps: never
    /// example: `Heap.of([3, 1, 2]).peek == Some(1)`
    pub @get fn peek(self) -> ?T = todo

    /// Remove all elements from the heap. O(1).
    ///
    /// The order `by:` is kept.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h = Heap.of([1, 2]); h.clear(); h.pop() == None`
    pub fn clear(mut self) = todo

    /// Move the heap's contents out in O(1), leaving `self` empty.
    ///
    /// Both heaps keep the order `by:`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h = Heap.of([2, 1]); let t = h.take(); h.is_empty and t.peek == Some(1)`
    pub fn take(mut self) -> Heap<T> = todo

    // Moves a hole down instead of swapping: one write per level, as std::priority_queue.
    fn sift_down(mut self, start: Int) = todo

    /// Push `x` into the heap. O(log n).
    ///
    /// cost: O(log n), amortized over list growth
    /// traps: never
    /// example: `var h = Heap.of([5]); h.push(2); h.peek == Some(2)`
    pub fn push(mut self, x: T) = todo

    /// Remove and return the smallest element, or `None` if the heap is empty. O(log n).
    ///
    /// cost: O(log n)
    /// empty: `None`
    /// traps: never
    /// example: `var h = Heap.of([2, 1]); h.pop() == Some(1) and h.pop() == Some(2) and h.pop() == None`
    pub fn pop(mut self) -> ?T = todo

    /// Reserve backing storage for at least `cap` elements. O(cap).
    ///
    /// cost: O(cap) when the storage grows, O(1) otherwise
    /// traps: never
    /// example: `var h = Heap.of([2]); h.reserve(100); h.peek == Some(2)`
    pub fn reserve(mut self, cap: Int) = todo

    /// Push `x` and pop the minimum in one sift. O(log n).
    ///
    /// Returns `x` itself, leaving the heap unchanged, when the heap is empty or the key of
    /// `x` is no greater than the top's; otherwise returns the old top and keeps `x`.
    ///
    /// cost: O(log n); O(1) when `x` is returned
    /// empty: returns `x`; the heap stays empty
    /// traps: never
    /// example: `var h = Heap.of([2, 5]); h.push_pop(3) == 2 and h.peek == Some(3)`
    /// example: `var h = Heap.of([2]); h.push_pop(1) == 1 and h.peek == Some(2)`
    pub fn push_pop(mut self, x: T) -> T = todo

    /// Replace the minimum with `x` and return the old minimum (or `None` if empty). O(log n).
    ///
    /// The old top is returned even when `x` is smaller than it. On an empty heap `x` is
    /// pushed and the result is `None`.
    ///
    /// cost: O(log n)
    /// empty: pushes `x` and returns `None`
    /// traps: never
    /// example: `var h = Heap.of([2, 5]); h.replace_top(9) == Some(2) and h.peek == Some(5)`
    /// example: `var h: Heap<Int> = Heap.new(); h.replace_top(4) == None and h.peek == Some(4)`
    pub fn replace_top(mut self, x: T) -> ?T = todo

    /// Push all elements of `items`. O(k log n).
    ///
    /// cost: O(k log n) for `k` items
    /// empty: an empty `items` does nothing
    /// traps: never
    /// example: `var h = Heap.of([4]); h.push_all([3, 1]); h.peek == Some(1) and h.len == 3`
    pub fn push_all(mut self, items: List<T>) = todo

    /// Pop and yield all elements in ascending order, leaving the heap empty. O(n log n).
    ///
    /// Each step is one `pop`. A loop that stops early leaves the elements not yet yielded in
    /// the heap.
    ///
    /// cost: O(log n) per step
    /// empty: yields nothing
    /// order: ascending key order, the order `pop` would return them
    /// traps: never
    /// example: `var h = Heap.of([3, 1, 2]); h.drain().collect() == [1, 2, 3] and h.is_empty`
    pub fn drain(mut self) -> yields T = todo

/// Min-heap supporting O(log n) arbitrary-value `remove(x)` via a lazy deletion heap.
/// `push` and `remove` are O(log n); `peek` and `pop` are O(log n) amortized; `len`, `is_empty`,
/// `clear`, and `take` are O(1).
pub struct ErasableHeap<T>:
    live: Heap<T>
    dead: Heap<T>

    /// Create an empty `ErasableHeap<T>`, optionally ordered by key extractor `by:` (W7). O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h: ErasableHeap<Int> = ErasableHeap.new(); h.push(2); h.push(1); h.remove(1); h.pop() == Some(2)`
    pub fn new<K>(by: ?(T -> K) = None) -> ErasableHeap<T> = todo

    /// Number of active elements in the heap. O(1).
    ///
    /// Pushed elements minus removed ones, counting copies.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h: ErasableHeap<Int> = ErasableHeap.new(); h.push(1); h.push(1); h.remove(1); h.len == 1`
    pub @get fn len(self) -> Int = todo

    /// Whether no active elements remain. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h: ErasableHeap<Int> = ErasableHeap.new(); h.push(1); h.remove(1); h.is_empty`
    pub @get fn is_empty(self) -> Bool = todo

    /// Remove all elements from the heap. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h: ErasableHeap<Int> = ErasableHeap.new(); h.push(1); h.clear(); h.pop() == None`
    pub fn clear(mut self) = todo

    /// Move the heap's contents out in O(1), leaving `self` empty.
    ///
    /// Pending removals move with the contents.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h: ErasableHeap<Int> = ErasableHeap.new(); h.push(1); let t = h.take(); h.is_empty and t.len == 1`
    pub fn take(mut self) -> ErasableHeap<T> = todo

    fn prune(mut self) = todo

    /// Push `x` into the heap. O(log n).
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `var h: ErasableHeap<Int> = ErasableHeap.new(); h.push(4); h.peek() == Some(4)`
    pub fn push(mut self, x: T) = todo

    /// Schedule one copy of `x` for lazy removal (`x` must be present). O(log n).
    ///
    /// One copy goes, even when several are present. The copy leaves at once as far as `len`,
    /// `peek` and `pop` can see; its storage is freed when it would reach the top. Removing a
    /// value that is not present corrupts the heap: debug builds trap when they find such a
    /// value surfacing.
    ///
    /// cost: O(log n)
    /// traps: never in release; debug traps when `x` is found not to be present
    /// example: `var h: ErasableHeap<Int> = ErasableHeap.new(); h.push(1); h.push(1); h.push(3); h.remove(1); h.pop() == Some(1) and h.pop() == Some(3)`
    pub fn remove(mut self, x: T) = todo

    /// Return the smallest active element, or `None` if empty. O(log n) amortized.
    ///
    /// Takes `mut self` because it first discards removed values sitting on top.
    ///
    /// cost: O(log n) amortized
    /// empty: `None`
    /// traps: never
    /// example: `var h: ErasableHeap<Int> = ErasableHeap.new(); h.push(1); h.push(2); h.remove(1); h.peek() == Some(2)`
    pub fn peek(mut self) -> ?T = todo

    /// Remove and return the smallest active element, or `None` if empty. O(log n) amortized.
    ///
    /// cost: O(log n) amortized
    /// empty: `None`
    /// traps: never
    /// example: `var h: ErasableHeap<Int> = ErasableHeap.new(); h.push(5); h.push(2); h.pop() == Some(2) and h.len == 1`
    pub fn pop(mut self) -> ?T = todo

/// Domain-keyed min-heap over `D` supporting O(log n) `set`, `decrease`, `increase`, `remove`,
/// and `pop`, plus O(1) `contains`, `get`, and `peek`.
///
/// Each `v: D` is in the heap at most once, with one key `K`. Ties between equal keys go to the
/// smaller `v` in `D`'s order, so the pop order is fully determined. Memory is O(|D|).
pub struct IndexedHeap<D, K>:
    heap: List<D>
    pos: [D]Int
    keys: [D]?K

    /// Create an empty `IndexedHeap<D, K>`. O(D.K).
    ///
    /// cost: O(|D|)
    /// traps: never
    /// example: `domain V = 0..4; var h: IndexedHeap<V, Int> = IndexedHeap.new(); h.len == 0`
    pub fn new() -> IndexedHeap<D, K> = todo

    /// Number of keys currently in the heap. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..4; var h: IndexedHeap<V, Int> = IndexedHeap.new(); h.set(V(1), 5); h.set(V(1), 6); h.len == 1`
    pub @get fn len(self) -> Int = todo

    /// Whether the heap is empty. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..4; var h: IndexedHeap<V, Int> = IndexedHeap.new(); h.is_empty`
    pub @get fn is_empty(self) -> Bool = todo

    /// Whether `v` is currently in the heap. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..4; var h: IndexedHeap<V, Int> = IndexedHeap.new(); h.set(V(2), 5); h.contains(V(2)) and not h.contains(V(3))`
    pub fn contains(self, v: D) -> Bool = todo

    /// Current priority of `v`, or `None` if absent. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain V = 0..4; var h: IndexedHeap<V, Int> = IndexedHeap.new(); h.set(V(2), 5); h.get(V(2)) == Some(5) and h.get(V(0)) == None`
    pub fn get(self, v: D) -> ?K = todo

    /// Pair `(v, key)` with the smallest key, or `None` if empty. O(1).
    ///
    /// cost: O(1)
    /// empty: `None`
    /// traps: never
    /// example: `domain V = 0..4; var h: IndexedHeap<V, Int> = IndexedHeap.new(); h.set(V(3), 1); h.set(V(1), 1); h.peek == Some((V(1), 1))`
    pub @get fn peek(self) -> ?(D, K) = todo

    /// Remove all entries from the heap. O(len).
    ///
    /// cost: O(len)
    /// traps: never
    /// example: `domain V = 0..4; var h: IndexedHeap<V, Int> = IndexedHeap.new(); h.set(V(1), 5); h.clear(); h.get(V(1)) == None`
    pub fn clear(mut self) = todo

    fn swap_at(mut self, i: Int, j: Int) = todo

    fn sift_up(mut self, start: Int) = todo

    fn sift_down(mut self, start: Int) = todo

    /// Insert `v` with key `k`, or update `v`'s key to `k`. O(log n).
    ///
    /// The new key may be larger or smaller than the old one.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `domain V = 0..4; var h: IndexedHeap<V, Int> = IndexedHeap.new(); h.set(V(1), 5); h.set(V(1), 9); h.get(V(1)) == Some(9)`
    pub fn set(mut self, v: D, k: K) = todo

    /// Insert `v` with `k`, or lower its key to `k` if `k` is smaller. Returns `true` if updated. O(log n).
    ///
    /// The relaxation step of Dijkstra. An equal key is not smaller: it returns `false`.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `domain V = 0..4; var h: IndexedHeap<V, Int> = IndexedHeap.new(); h.decrease(V(1), 5) and not h.decrease(V(1), 5) and h.decrease(V(1), 3) and h.get(V(1)) == Some(3)`
    pub fn decrease(mut self, v: D, k: K) -> Bool = todo

    /// Insert `v` with `k`, or raise its key to `k` if `k` is larger. Returns `true` if updated. O(log n).
    ///
    /// An equal key is not larger: it returns `false`.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `domain V = 0..4; var h: IndexedHeap<V, Int> = IndexedHeap.new(); h.increase(V(1), 5) and not h.increase(V(1), 2) and h.get(V(1)) == Some(5)`
    pub fn increase(mut self, v: D, k: K) -> Bool = todo

    /// Remove `v` from the heap if present; returns its old key or `None`. O(log n).
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `domain V = 0..4; var h: IndexedHeap<V, Int> = IndexedHeap.new(); h.set(V(1), 5); h.remove(V(1)) == Some(5) and h.remove(V(1)) == None`
    pub fn remove(mut self, v: D) -> ?K = todo

    /// Remove and return `(v, key)` with the smallest key, or `None` if empty. O(log n).
    ///
    /// cost: O(log n)
    /// empty: `None`
    /// traps: never
    /// example: `domain V = 0..4; var h: IndexedHeap<V, Int> = IndexedHeap.new(); h.set(V(2), 5); h.set(V(0), 7); h.pop() == Some((V(2), 5)) and h.len == 1`
    /// example: `domain V = 0..4; var h: IndexedHeap<V, Int> = IndexedHeap.new(); h.set(V(3), 1); h.set(V(1), 1); h.pop() == Some((V(1), 1))`
    pub fn pop(mut self) -> ?(D, K) = todo

/// Double-ended priority queue (min-max heap).
/// `first` and `last` are O(1); `push`, `pop_first`, and `pop_last` are O(log n).
pub struct MinMaxHeap<T>:
    _xs: List<T>

    /// Create an empty `MinMaxHeap<T>`, optionally ordered by key extractor `by:` (W7). O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h: MinMaxHeap<Int> = MinMaxHeap.new(); h.push(2); h.push(7); h.first == Some(2) and h.last == Some(7)`
    pub fn new<K>(by: ?(T -> K) = None) -> MinMaxHeap<T> = todo

    /// Build a `MinMaxHeap<T>` from `xs` in linear time. O(n).
    ///
    /// The heap orders the elements themselves (there is no `by:`).
    ///
    /// cost: O(n)
    /// empty: an empty heap
    /// traps: never
    /// example: `var h = MinMaxHeap.of([3, 1, 4, 1, 5]); h.pop_last() == Some(5) and h.pop_first() == Some(1) and h.first == Some(1)`
    pub fn of(xs: List<T>) -> MinMaxHeap<T> = todo

    /// Reserve backing storage for at least `cap` elements. O(cap).
    ///
    /// cost: O(cap) when the storage grows, O(1) otherwise
    /// traps: never
    /// example: `var h = MinMaxHeap.of([2]); h.reserve(100); h.first == Some(2)`
    pub fn reserve(mut self, cap: Int) = todo

    /// Number of elements currently in the heap. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `MinMaxHeap.of([1, 1]).len == 2`
    pub @get fn len(self) -> Int = todo

    /// Whether the heap is empty. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `MinMaxHeap.of([1]).is_empty == false`
    pub @get fn is_empty(self) -> Bool = todo

    /// The smallest element, or `None` if empty. O(1).
    ///
    /// With one element, `first` and `last` are the same element.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// ties: unspecified among equal keys under `by:`; `pop_first` returns the one shown
    /// traps: never
    /// example: `MinMaxHeap.of([3, 1, 2]).first == Some(1)`
    /// example: `MinMaxHeap.of([7]).first == Some(7)`
    pub @get fn first(self) -> ?T = todo

    /// The largest element, or `None` if empty. O(1).
    ///
    /// cost: O(1)
    /// empty: `None`
    /// ties: unspecified among equal keys under `by:`; `pop_last` returns the one shown
    /// traps: never
    /// example: `MinMaxHeap.of([3, 1, 2]).last == Some(3)`
    /// example: `MinMaxHeap.of([7]).last == Some(7)`
    pub @get fn last(self) -> ?T = todo

    /// Insert `x`. O(log n).
    ///
    /// cost: O(log n), amortized over list growth
    /// traps: never
    /// example: `var h = MinMaxHeap.of([2, 5]); h.push(9); h.last == Some(9) and h.first == Some(2)`
    pub fn push(mut self, x: T) = todo

    /// Remove and return the smallest element, or `None` if empty. O(log n).
    ///
    /// cost: O(log n)
    /// empty: `None`
    /// traps: never
    /// example: `var h = MinMaxHeap.of([2, 5, 3]); h.pop_first() == Some(2) and h.first == Some(3)`
    pub fn pop_first(mut self) -> ?T = todo

    /// Remove and return the largest element, or `None` if empty. O(log n).
    ///
    /// cost: O(log n)
    /// empty: `None`
    /// traps: never
    /// example: `var h = MinMaxHeap.of([2, 5, 3]); h.pop_last() == Some(5) and h.last == Some(3)`
    pub fn pop_last(mut self) -> ?T = todo

    /// Remove all elements. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h = MinMaxHeap.of([2]); h.clear(); h.first == None`
    pub fn clear(mut self) = todo

    /// Move contents out in O(1), leaving `self` empty.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h = MinMaxHeap.of([2]); let t = h.take(); h.is_empty and t.first == Some(2)`
    pub fn take(mut self) -> MinMaxHeap<T> = todo

/// Meldable min-heap (skew / leftist heap) with O(log n) `meld` and O(1) lazy offset `add_all`.
/// `peek` / `top` is O(1); `push`, `pop`, and `meld` are O(log n).
pub struct MeldHeap<T>:
    _len: Int

    /// Create an empty `MeldHeap<T>`, optionally ordered by key extractor `by:` (W7). O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h: MeldHeap<Int> = MeldHeap.new(); h.push(3); h.push(1); h.pop() == Some(1)`
    pub fn new<K>(by: ?(T -> K) = None) -> MeldHeap<T> = todo

    /// Build a `MeldHeap<T>` from `xs`. O(n).
    ///
    /// The heap orders the elements themselves (there is no `by:`).
    ///
    /// cost: O(n)
    /// empty: an empty heap
    /// traps: never
    /// example: `MeldHeap.of([4, 2, 3]).peek == Some(2)`
    pub fn of(xs: List<T>) -> MeldHeap<T> = todo

    /// Number of elements currently in the heap. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `MeldHeap.of([4, 2]).len == 2`
    pub @get fn len(self) -> Int = todo

    /// Whether the heap is empty. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `MeldHeap.of([4]).is_empty == false`
    pub @get fn is_empty(self) -> Bool = todo

    /// The smallest element, or `None` if empty. O(1).
    ///
    /// The value includes every `add_all` applied since it was pushed.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// traps: never
    /// example: `var h = MeldHeap.of([4, 2]); h.add_all(1); h.peek == Some(3)`
    pub @get fn peek(self) -> ?T = todo

    /// The smallest element, or `None` if empty (alias for `peek`). O(1).
    ///
    /// cost: O(1)
    /// empty: `None`
    /// ties: as `peek`: unspecified among equal keys under `by:`
    /// traps: never
    /// see: `peek`
    /// example: `MeldHeap.of([4, 2]).top == Some(2)`
    pub @get fn top(self) -> ?T = todo

    /// Push `x` into the heap. O(log n).
    ///
    /// `x` is stored as given: earlier `add_all` offsets do not apply to it.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `var h = MeldHeap.of([1, 5]); h.add_all(10); h.push(3); h.drain() == [3, 11, 15]`
    pub fn push(mut self, x: T) = todo

    /// Push all elements of `items`. O(k log n).
    ///
    /// cost: O(k log n) for `k` items
    /// empty: an empty `items` does nothing
    /// traps: never
    /// example: `var h = MeldHeap.of([4]); h.push_all([3, 1]); h.drain() == [1, 3, 4]`
    pub fn push_all(mut self, items: List<T>) = todo

    /// Remove and return the smallest element, or `None` if empty. O(log n).
    ///
    /// cost: O(log n)
    /// empty: `None`
    /// traps: never
    /// example: `var h = MeldHeap.of([4, 2]); h.pop() == Some(2) and h.pop() == Some(4) and h.pop() == None`
    pub fn pop(mut self) -> ?T = todo

    /// Meld `other` into `self` (consuming `other`). O(log n).
    ///
    /// Every element keeps the value its own heap reported: pending `add_all` offsets apply only
    /// to the elements they were added to.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `var a = MeldHeap.of([4, 2]); a.meld(MeldHeap.of([3, 1])); a.drain() == [1, 2, 3, 4]`
    pub fn meld(mut self, other: MeldHeap<T>) = todo

    /// Meld `other` into `self`, leaving `other` empty. O(log n).
    ///
    /// `meld` for an `other` you keep using: small-to-large merging over a tree.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `var a = MeldHeap.of([4]); var b = MeldHeap.of([1]); a.absorb(mut b); a.peek == Some(1) and b.is_empty`
    pub fn absorb(mut self, mut other: MeldHeap<T>) = todo

    /// Add `delta` lazily to every element (for additive types). O(1).
    ///
    /// Only the elements in the heap now change; later pushes do not. Adding one value to
    /// every element keeps their order; with `by:`, the key must keep that order too, or later
    /// pops are unspecified.
    ///
    /// cost: O(1)
    /// empty: does nothing
    /// traps: never (overflow wraps in release, traps in debug)
    /// example: `var h = MeldHeap.of([1, 5]); h.add_all(10); h.pop() == Some(11)`
    pub fn add_all(mut self, delta: T) = todo

    /// Remove all elements. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h = MeldHeap.of([1]); h.clear(); h.pop() == None`
    pub fn clear(mut self) = todo

    /// Move contents out in O(1), leaving `self` empty.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var h = MeldHeap.of([1]); let t = h.take(); h.is_empty and t.peek == Some(1)`
    pub fn take(mut self) -> MeldHeap<T> = todo

    /// Pop and return all elements in ascending order, leaving `self` empty. O(n log n).
    ///
    /// cost: O(n log n)
    /// empty: `[]`
    /// order: ascending key order, the order `pop` would return them; a new `List`, keyed from 0
    /// traps: never
    /// example: `var h = MeldHeap.of([3, 1, 2]); h.drain() == [1, 2, 3] and h.is_empty`
    pub fn drain(mut self) -> List<T> = todo

@test
fn pops_in_sorted_order() = todo

@test
fn desc_gives_a_mixed_order() = todo

@test
fn clear_empties_the_heap() = todo

@test
fn take_and_small_into_large_swap() = todo

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
    /// Offer priority `p` for key `k`: an unsettled key keeps `best(old, p)` (or `p` when it
    /// had none); a settled key ignores the offer.
    ///
    /// cost: O(log n) amortized, `n` the offers so far
    /// traps: never
    /// example: `var s: Settle<Char, Int> = Settle(); s.offer('a', 5); s.offer('a', 2); s.offer('a', 4); s.get('a') == Some(2)`
    pub fn offer(mut self, k: K, p: P) = todo
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
    pub fn pop(mut self) -> ?(K, P) = todo
    /// Whether `k` has popped.
    ///
    /// cost: O(1) expected
    /// traps: never
    /// example: `var s: Settle<Char, Int> = Settle(); s.offer('a', 1); not s.settled('a') and s.pop() == Some(('a', 1)) and s.settled('a')`
    pub fn settled(self, k: K) -> Bool = todo
    /// The best priority offered for `k` so far (for a settled key, the one it popped with),
    /// or `None` if `k` was never offered.
    ///
    /// cost: O(1) expected
    /// traps: never
    /// example: `var s: Settle<Char, Int> = Settle(); s.offer('a', 4); s.get('a') == Some(4) and s.get('b') == None`
    pub fn get(self, k: K) -> ?P = todo
