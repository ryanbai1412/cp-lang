//! Double-ended queue: `Deque<T>`.

use std.seq.{remove}

/// A queue with both ends open: `var q: Deque<Int> = []`. `push` / `pop` work at the back,
/// `push_front` / `pop_front` at the front, all amortized O(1); `q[i]` is the `i`-th element
/// from the front in O(1). Memory is a ring of at most twice the largest length.
pub struct Deque<T>:
    buf: List<T>
    head: Int
    n: Int
    mask: Int
    want: Int

    /// Create an empty `Deque<T>`. O(1).
    pub fn new() -> Deque<T> = todo

    /// A deque holding `xs`, front to back: the queue of a search, `var q = Deque.of(sources)`,
    /// then `while let u = q.pop_front(): ...`. O(n).
    pub fn of(xs: List<T>) -> Deque<T> = todo

    /// Reserve capacity for at least `cap` elements without reallocating. O(1) when empty, O(cap) otherwise.
    pub fn reserve(mut self, cap: Int) = todo

    /// Number of elements currently in the deque. O(1).
    pub @get fn len(self) -> Int = todo
    /// Whether the deque is empty. O(1).
    pub @get fn is_empty(self) -> Bool = todo

    /// Remove all elements from the deque. O(1).
    pub fn clear(mut self) = todo

    /// Move the deque's contents out in O(1), leaving `self` empty.
    pub fn take(mut self) -> Deque<T> = todo

    fn grow_to(mut self, cap: Int, fill: T) = todo

    /// Append `x` to the back of the deque. O(1) amortized.
    pub fn push(mut self, x: T) = todo

    /// Prepend `x` to the front of the deque. O(1) amortized.
    pub fn push_front(mut self, x: T) = todo

    /// Remove and return the back, or `none` when empty.
    pub fn pop(mut self) -> ?T = todo

    /// Remove and return the front, or `none` when empty.
    pub fn pop_front(mut self) -> ?T = todo

    /// O(k) for `k` removed. Remove and return the longest front run on which `ok` holds,
    /// in order: a queue sorted by key releases everything below a rising threshold.
    pub fn pop_front_while(mut self, ok: T -> Bool) -> List<T> = todo

    /// Pop from the back while `ok` holds; the popped values, back first.
    /// O(popped).
    pub fn pop_while(mut self, ok: T -> Bool) -> List<T> = todo

    /// The front, or `none` when empty.
    pub @get fn first(self) -> ?T = todo

    /// The back, or `none` when empty. `dq.last(k)` is the element `k`
    /// back from it, so `dq.last(1)` is the one before the back.
    pub @get fn last(self, k: Int = 0) -> ?T = todo

    /// Each element with its place from the front: `for (i, x) in dq.items`. O(1) to make,
    /// O(n) to walk.
    pub @get fn items(self) -> yields (Int, T) = todo

    /// The `i`-th element from the front. O(1).
    pub fn get(self, i: Int) -> T = todo

    /// Overwrite the `i`-th element from the front (W8 `IndexSet` backing `dq[i] = x`). O(1).
    pub fn set(mut self, i: Int, x: T) = todo

    /// Materialize the elements front to back as a `List<T>`. O(n).
    pub fn to_list(self) -> List<T> = todo

    /// Yield elements front to back without allocating. O(n).
    pub fn each(self) -> yields T = todo

    /// Yield elements back to front without allocating. O(n).
    pub fn each_rev(self) -> yields T = todo

    /// Rotate elements left by `k` positions (`k` mod `len`). O(min(k, n - k)).
    pub fn rotate_left(mut self, k: Int) = todo

    /// Rotate elements right by `k` positions (`k` mod `len`). O(min(k, n - k)).
    pub fn rotate_right(mut self, k: Int) = todo

/// Doubly-linked list supporting O(1) end pushes/pops and O(1) splicing (`splice_back`, `splice_front`).
pub struct LinkedList<T>:
    _len: Int

    /// Create an empty `LinkedList<T>`. O(1).
    pub fn new() -> LinkedList<T> = todo

    /// Create a singleton `LinkedList<T>` containing `x`. O(1).
    pub fn of(x: T) -> LinkedList<T> = todo

    /// Build a `LinkedList<T>` from `xs`, front to back. O(n).
    pub fn from_list(xs: List<T>) -> LinkedList<T> = todo

    /// Number of elements currently in the list. O(1).
    pub @get fn len(self) -> Int = todo

    /// Whether the list is empty. O(1).
    pub @get fn is_empty(self) -> Bool = todo

    /// The front element (traps if empty). O(1).
    pub @get fn first(self) -> T = todo

    /// The back element (traps if empty). O(1).
    pub @get fn last(self) -> T = todo

    /// Append `x` to the back. O(1).
    pub fn push(mut self, x: T) = todo

    /// Prepend `x` to the front. O(1).
    pub fn push_front(mut self, x: T) = todo

    /// Remove and return the back element, or `none` if empty. O(1).
    pub fn pop(mut self) -> ?T = todo

    /// Remove and return the back element (alias for `pop`). O(1).
    pub fn pop_back(mut self) -> ?T = todo

    /// Remove and return the front element, or `none` if empty. O(1).
    pub fn pop_front(mut self) -> ?T = todo

    /// Splice all elements of `other` onto the back of `self`, consuming `other`. O(1).
    pub fn splice_back(mut self, other: LinkedList<T>) = todo

    /// Splice all elements of `other` onto the front of `self`, consuming `other`. O(1).
    pub fn splice_front(mut self, other: LinkedList<T>) = todo

    /// Reverse the list in place. O(1) via reversible links.
    pub fn reverse(mut self) = todo

    /// Remove all elements. O(1).
    pub fn clear(mut self) = todo

    /// Yield elements front to back. O(n).
    pub fn each(self) -> yields T = todo

    /// Yield elements back to front. O(n).
    pub fn each_rev(self) -> yields T = todo

@test
fn a_deque_matches_a_list() = todo

@test
fn a_deque_as_a_bfs_queue() = todo

@test
fn a_deque_of_a_list_is_a_bfs_queue() = todo

@test
fn a_deque_reserve_preserves_order() = todo
