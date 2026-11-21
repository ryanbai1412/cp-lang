//! Binary heaps: `MinHeap<T>()` pops the smallest first, `MaxHeap<T>()` the largest.
use std.algebra.{Desc, desc}

/// A priority queue on `<`. `push` and `pop` are O(log n), `peek` is O(1); memory is one list.
/// For a mixed order put `desc(x)` in the entry: `MinHeap<(Int, Desc<V>)>()` pops the
/// smallest distance first and, among equals, the largest vertex.
pub struct MinHeap<T>:
    xs: List<T>

    /// Create an empty `MinHeap<T>`. O(1).
    pub fn new() -> MinHeap<T> = MinHeap([])

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
        var i = 0
        while true:
            var best = i
            for c in [2 * i + 1, 2 * i + 2]:
                if c < xs.len and xs[c] < xs[best]: best = c
            if best == i: break
            xs.swap(i, best)
            i = best
        out

/// A priority queue that pops the largest element first. Same costs as `MinHeap`.
pub struct MaxHeap<T>:
    h: MinHeap<Desc<T>>

    /// Create an empty `MaxHeap<T>`. O(1).
    pub fn new() -> MaxHeap<T> = MaxHeap(MinHeap<Desc<T>>())

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
