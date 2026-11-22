//! Binary heaps (round 5): `Heap<T>(by:)` pops the smallest key first and `Heap(by: desc)` the
//! largest. One type per shape: `Heap`, `ErasableHeap`, `IndexedHeap`, `MeldHeap`, `MinMaxHeap`.
use std.algebra.{Desc, desc}

/// A priority queue on `<`. `push` and `pop` are O(log n), `peek` is O(1); memory is one list.
/// For a mixed order put `desc(x)` in the entry: `Heap<(Int, Desc<V>)>()` pops the
/// smallest distance first and, among equals, the largest vertex.
pub struct Heap<T>:
    xs: List<T>

    /// Create an empty `Heap<T>`, optionally ordered by key extractor `by:` (W7). O(1).
    pub fn new<K>(by: ?(T -> K) = None) -> Heap<T> = todo

    /// Build a `Heap<T>` from `xs` via Floyd heapify. O(n).
    pub fn of(xs: List<T>) -> Heap<T> = todo

    /// Number of elements currently in the heap. O(1).
    pub @get fn len(self) -> Int = todo
    /// Whether the heap is empty. O(1).
    pub @get fn is_empty(self) -> Bool = todo

    /// The smallest element, or `None` if the heap is empty.
    pub @get fn peek(self) -> ?T = todo

    /// Remove all elements from the heap. O(1).
    pub fn clear(mut self) = todo

    /// Move the heap's contents out in O(1), leaving `self` empty.
    pub fn take(mut self) -> Heap<T> = todo

    // Moves a hole down instead of swapping: one write per level, as std::priority_queue.
    fn sift_down(mut self, start: Int) = todo

    /// Push `x` into the heap. O(log n).
    pub fn push(mut self, x: T) = todo

    /// Remove and return the smallest element, or `None` if the heap is empty. O(log n).
    pub fn pop(mut self) -> ?T = todo

    /// Reserve backing storage for at least `cap` elements. O(cap).
    pub fn reserve(mut self, cap: Int) = todo

    /// Push `x` and pop the minimum in one sift. O(log n).
    pub fn push_pop(mut self, x: T) -> T = todo

    /// Replace the minimum with `x` and return the old minimum (or `None` if empty). O(log n).
    pub fn replace_top(mut self, x: T) -> ?T = todo

    /// Push all elements of `items`. O(k log n).
    pub fn push_all(mut self, items: List<T>) = todo

    /// Pop and yield all elements in ascending order, leaving the heap empty. O(n log n).
    pub fn drain(mut self) -> yields T = todo

/// Min-heap supporting O(log n) arbitrary-value `remove(x)` via a lazy deletion heap.
/// `push` and `remove` are O(log n); `peek` and `pop` are O(log n) amortized; `len`, `is_empty`,
/// `clear`, and `take` are O(1).
pub struct ErasableHeap<T>:
    live: Heap<T>
    dead: Heap<T>

    /// Create an empty `ErasableHeap<T>`, optionally ordered by key extractor `by:` (W7). O(1).
    pub fn new<K>(by: ?(T -> K) = None) -> ErasableHeap<T> = todo

    /// Number of active elements in the heap. O(1).
    pub @get fn len(self) -> Int = todo

    /// Whether no active elements remain. O(1).
    pub @get fn is_empty(self) -> Bool = todo

    /// Remove all elements from the heap. O(1).
    pub fn clear(mut self) = todo

    /// Move the heap's contents out in O(1), leaving `self` empty.
    pub fn take(mut self) -> ErasableHeap<T> = todo

    fn prune(mut self) = todo

    /// Push `x` into the heap. O(log n).
    pub fn push(mut self, x: T) = todo

    /// Schedule one copy of `x` for lazy removal (`x` must be present). O(log n).
    pub fn remove(mut self, x: T) = todo

    /// Return the smallest active element, or `None` if empty. O(log n) amortized.
    pub fn peek(mut self) -> ?T = todo

    /// Remove and return the smallest active element, or `None` if empty. O(log n) amortized.
    pub fn pop(mut self) -> ?T = todo

/// Domain-keyed min-heap over `D` supporting O(log n) `set`, `decrease`, `increase`, `remove`,
/// and `pop`, plus O(1) `contains`, `get`, and `peek`.
pub struct IndexedHeap<D, K>:
    heap: List<D>
    pos: [D]Int
    keys: [D]?K

    /// Create an empty `IndexedHeap<D, K>`. O(D.K).
    pub fn new() -> IndexedHeap<D, K> = todo

    /// Number of keys currently in the heap. O(1).
    pub @get fn len(self) -> Int = todo

    /// Whether the heap is empty. O(1).
    pub @get fn is_empty(self) -> Bool = todo

    /// Whether `v` is currently in the heap. O(1).
    pub fn contains(self, v: D) -> Bool = todo

    /// Current priority of `v`, or `None` if absent. O(1).
    pub fn get(self, v: D) -> ?K = todo

    /// Pair `(v, key)` with the smallest key, or `None` if empty. O(1).
    pub @get fn peek(self) -> ?(D, K) = todo

    /// Remove all entries from the heap. O(len).
    pub fn clear(mut self) = todo

    fn swap_at(mut self, i: Int, j: Int) = todo

    fn sift_up(mut self, start: Int) = todo

    fn sift_down(mut self, start: Int) = todo

    /// Insert `v` with key `k`, or update `v`'s key to `k`. O(log n).
    pub fn set(mut self, v: D, k: K) = todo

    /// Insert `v` with `k`, or lower its key to `k` if `k` is smaller. Returns `true` if updated. O(log n).
    pub fn decrease(mut self, v: D, k: K) -> Bool = todo

    /// Insert `v` with `k`, or raise its key to `k` if `k` is larger. Returns `true` if updated. O(log n).
    pub fn increase(mut self, v: D, k: K) -> Bool = todo

    /// Remove `v` from the heap if present; returns its old key or `None`. O(log n).
    pub fn remove(mut self, v: D) -> ?K = todo

    /// Remove and return `(v, key)` with the smallest key, or `None` if empty. O(log n).
    pub fn pop(mut self) -> ?(D, K) = todo

/// Double-ended priority queue (min-max heap).
/// `first` and `last` are O(1); `push`, `pop_first`, and `pop_last` are O(log n).
pub struct MinMaxHeap<T>:
    _xs: List<T>

    /// Create an empty `MinMaxHeap<T>`, optionally ordered by key extractor `by:` (W7). O(1).
    pub fn new<K>(by: ?(T -> K) = None) -> MinMaxHeap<T> = todo

    /// Build a `MinMaxHeap<T>` from `xs` in linear time. O(n).
    pub fn of(xs: List<T>) -> MinMaxHeap<T> = todo

    /// Reserve backing storage for at least `cap` elements. O(cap).
    pub fn reserve(mut self, cap: Int) = todo

    /// Number of elements currently in the heap. O(1).
    pub @get fn len(self) -> Int = todo

    /// Whether the heap is empty. O(1).
    pub @get fn is_empty(self) -> Bool = todo

    /// The smallest element, or `None` if empty. O(1).
    pub @get fn first(self) -> ?T = todo

    /// The largest element, or `None` if empty. O(1).
    pub @get fn last(self) -> ?T = todo

    /// Insert `x`. O(log n).
    pub fn push(mut self, x: T) = todo

    /// Remove and return the smallest element, or `None` if empty. O(log n).
    pub fn pop_first(mut self) -> ?T = todo

    /// Remove and return the largest element, or `None` if empty. O(log n).
    pub fn pop_last(mut self) -> ?T = todo

    /// Remove all elements. O(1).
    pub fn clear(mut self) = todo

    /// Move contents out in O(1), leaving `self` empty.
    pub fn take(mut self) -> MinMaxHeap<T> = todo

/// Meldable min-heap (skew / leftist heap) with O(log n) `meld` and O(1) lazy offset `add_all`.
/// `peek` / `top` is O(1); `push`, `pop`, and `meld` are O(log n).
pub struct MeldHeap<T>:
    _len: Int

    /// Create an empty `MeldHeap<T>`, optionally ordered by key extractor `by:` (W7). O(1).
    pub fn new<K>(by: ?(T -> K) = None) -> MeldHeap<T> = todo

    /// Build a `MeldHeap<T>` from `xs`. O(n).
    pub fn of(xs: List<T>) -> MeldHeap<T> = todo

    /// Number of elements currently in the heap. O(1).
    pub @get fn len(self) -> Int = todo

    /// Whether the heap is empty. O(1).
    pub @get fn is_empty(self) -> Bool = todo

    /// The smallest element, or `None` if empty. O(1).
    pub @get fn peek(self) -> ?T = todo

    /// The smallest element, or `None` if empty (alias for `peek`). O(1).
    pub @get fn top(self) -> ?T = todo

    /// Push `x` into the heap. O(log n).
    pub fn push(mut self, x: T) = todo

    /// Push all elements of `items`. O(k log n).
    pub fn push_all(mut self, items: List<T>) = todo

    /// Remove and return the smallest element, or `None` if empty. O(log n).
    pub fn pop(mut self) -> ?T = todo

    /// Meld `other` into `self` (consuming `other`). O(log n).
    pub fn meld(mut self, other: MeldHeap<T>) = todo

    /// Meld `other` into `self`, leaving `other` empty. O(log n).
    pub fn absorb(mut self, mut other: MeldHeap<T>) = todo

    /// Add `delta` lazily to every element (for additive types). O(1).
    pub fn add_all(mut self, delta: T) = todo

    /// Remove all elements. O(1).
    pub fn clear(mut self) = todo

    /// Move contents out in O(1), leaving `self` empty.
    pub fn take(mut self) -> MeldHeap<T> = todo

    /// Pop and return all elements in ascending order, leaving `self` empty. O(n log n).
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
pub struct Settle<K, P>(best: (P, P) -> P = min):
    pub fn offer(mut self, k: K, p: P) = todo
    pub fn pop(mut self) -> ?(K, P) = todo
    pub fn settled(self, k: K) -> Bool = todo
    pub fn get(self, k: K) -> ?P = todo
