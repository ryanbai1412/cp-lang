//! Binary heap ordered by an `Order` value.
use std.algebra.{Order, Less, Greater}

/// A priority queue: `top` is the first element under `order`.
/// `Heap<Int>(Less<Int>)` pops the smallest first, `Heap<Int>(Greater<Int>)` the largest.
/// `push` and `pop` are O(log n), `top` is O(1); memory is one list.
pub struct Heap<T>(order: Order<T>):
    xs: List<T>

    pub fn new(order: Order<T>) -> Heap<T>:
        Heap(order, [])

    pub @get fn len(self) -> Int = xs.len
    pub @get fn is_empty(self) -> Bool = xs.is_empty

    /// The first element, or `none` if the heap is empty.
    pub @get fn top(self) -> ?T = if xs.is_empty: none else: xs[0]

    pub fn push(mut self, x: T):
        xs.push(x)
        var i = xs.len - 1
        while i > 0:
            let p = (i - 1) / 2
            if not order.lt(xs[i], xs[p]): break
            xs.swap(i, p)
            i = p

    /// Remove and return the first element, or `none` if the heap is empty.
    pub fn pop(mut self) -> ?T:
        if xs.is_empty: return none
        xs.swap(0, xs.len - 1)
        let out = xs.pop()
        var i = 0
        while true:
            var best = i
            for c in [2 * i + 1, 2 * i + 2]:
                if c < xs.len and order.lt(xs[c], xs[best]): best = c
            if best == i: break
            xs.swap(i, best)
            i = best
        out

@test
fn pops_in_sorted_order():
    var h = Heap<Int>(Less<Int>)
    var xs = [rand(-50, 50) for _ in 0..40]
    for x in xs: h.push(x)
    assert h.len == 40
    sort(mut xs)
    for x in xs:
        assert h.top == x
        assert h.pop() == x
    assert h.pop() == none and h.is_empty

@test
fn max_heap_with_a_custom_order():
    var h = Heap<(Int, Str)>(Order<(Int, Str)>.by(p => -p.0))
    h.push((2, "b"))
    h.push((9, "z"))
    h.push((5, "m"))
    assert h.pop()!.1 == "z"
    assert h.pop()!.1 == "m"
    var g = Heap<Int>(Greater<Int>)
    for x in [3, 1, 4, 1, 5]: g.push(x)
    assert g.pop() == 5 and g.pop() == 4
