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
    ///
    /// cost: O(1); nothing is allocated until the first push
    /// traps: never
    /// example: `var q: Deque<Int> = Deque.new(); q.len == 0`
    pub fn new() -> Deque<T> = todo

    /// A deque holding `xs`, front to back: the queue of a search, `var q = Deque.of(sources)`,
    /// then `while let u = q.pop_front(): ...`. O(n).
    ///
    /// The first element of `xs` is the front, so `pop_front` returns the elements in the
    /// order `xs` lists them.
    ///
    /// cost: O(n)
    /// empty: an empty deque
    /// traps: never
    /// example: `Deque.of([1, 2, 3]).to_list() == [1, 2, 3]`
    /// example: `var q = Deque.of([1, 2]); q.pop_front() == Some(1)`
    pub fn of(xs: List<T>) -> Deque<T> = todo

    /// Reserve capacity for at least `cap` elements without reallocating. O(1) when empty, O(cap) otherwise.
    ///
    /// The contents and their order do not change; a `cap` no larger than the current
    /// capacity (including any `cap <= 0`) does nothing.
    ///
    /// cost: O(1) when empty, O(cap) otherwise
    /// traps: never
    /// example: `var q = Deque.of([1, 2]); q.reserve(100); q.to_list() == [1, 2]`
    pub fn reserve(mut self, cap: Int) = todo

    /// Number of elements currently in the deque. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Deque.of([4, 5]).len == 2`
    pub @get fn len(self) -> Int = todo
    /// Whether the deque is empty. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Deque.of([7]).is_empty == false`
    pub @get fn is_empty(self) -> Bool = todo

    /// Remove all elements from the deque. O(1).
    ///
    /// The capacity is kept, so refilling does not reallocate.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var q = Deque.of([1, 2]); q.clear(); q.len == 0`
    pub fn clear(mut self) = todo

    /// Move the deque's contents out in O(1), leaving `self` empty.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var q = Deque.of([1, 2]); let r = q.take(); q.len == 0 and r.to_list() == [1, 2]`
    pub fn take(mut self) -> Deque<T> = todo

    fn grow_to(mut self, cap: Int, fill: T) = todo

    /// Append `x` to the back of the deque. O(1) amortized.
    ///
    /// cost: O(1) amortized; a full ring doubles
    /// traps: never
    /// example: `var q = Deque.of([1]); q.push(2); q.to_list() == [1, 2]`
    pub fn push(mut self, x: T) = todo

    /// Prepend `x` to the front of the deque. O(1) amortized.
    ///
    /// Every existing element moves one place back: `q[0]` is now `x`.
    ///
    /// cost: O(1) amortized; a full ring doubles
    /// traps: never
    /// example: `var q = Deque.of([1]); q.push_front(0); q.to_list() == [0, 1]`
    pub fn push_front(mut self, x: T) = todo

    /// Remove and return the back, or `None` when empty.
    ///
    /// cost: O(1)
    /// empty: `None`, and the deque stays empty
    /// traps: never
    /// example: `var q = Deque.of([1, 2]); q.pop() == Some(2) and q.to_list() == [1]`
    pub fn pop(mut self) -> ?T = todo

    /// Remove and return the front, or `None` when empty.
    ///
    /// cost: O(1)
    /// empty: `None`, and the deque stays empty
    /// traps: never
    /// example: `var q = Deque.of([1, 2]); q.pop_front() == Some(1) and q.to_list() == [2]`
    pub fn pop_front(mut self) -> ?T = todo

    /// O(k) for `k` removed. Remove and return the longest front run on which `ok` holds,
    /// in order: a queue sorted by key releases everything below a rising threshold.
    ///
    /// `ok` is called front to back and the run stops at the first element where it is false;
    /// that element and everything behind it stay. `ok` is called at most `k + 1` times.
    ///
    /// cost: O(k) for `k` removed
    /// empty: `[]`
    /// order: front first, the order the elements sat in; a new `List`, keyed from 0
    /// traps: never
    /// example: `var q = Deque.of([1, 2, 5, 3]); q.pop_front_while(x => x < 4) == [1, 2] and q.to_list() == [5, 3]`
    pub fn pop_front_while(mut self, ok: T -> Bool) -> List<T> = todo

    /// Pop from the back while `ok` holds; the popped values, back first.
    /// O(popped).
    ///
    /// The mirror of `pop_front_while`: `ok` is called back to front and the run stops at the
    /// first element where it is false.
    ///
    /// cost: O(k) for `k` removed
    /// empty: `[]`
    /// order: back first, the order they were popped; a new `List`, keyed from 0
    /// traps: never
    /// example: `var q = Deque.of([5, 1, 2]); q.pop_while(x => x < 4) == [2, 1] and q.to_list() == [5]`
    pub fn pop_while(mut self, ok: T -> Bool) -> List<T> = todo

    /// The front, or `None` when empty.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// ties: none: there is one front
    /// traps: never
    /// example: `Deque.of([4, 5]).first == Some(4)`
    pub @get fn first(self) -> ?T = todo

    /// The back, or `None` when empty. `dq.last(k)` is the element `k`
    /// back from it, so `dq.last(1)` is the one before the back.
    ///
    /// `dq.last(k)` is `dq[dq.len - 1 - k]` when `k < dq.len`, and `None` otherwise.
    ///
    /// cost: O(1)
    /// empty: `None` for every `k`
    /// ties: none: each `k` names one place
    /// traps: when `k < 0`
    /// example: `Deque.of([4, 5, 6]).last == Some(6)`
    /// example: `Deque.of([4, 5, 6]).last(1) == Some(5)`
    /// example: `Deque.of([4]).last(1) == None`
    pub @get fn last(self, k: Int = 0) -> ?T = todo

    /// Each element with its place from the front: `for (i, x) in dq.items`. O(1) to make,
    /// O(n) to walk.
    ///
    /// cost: O(1) to make, O(1) per step
    /// empty: yields nothing
    /// order: front to back; the places are `0..len`
    /// traps: never
    /// example: `Deque.of([7, 8]).items.collect() == [(0, 7), (1, 8)]`
    pub @get fn items(self) -> yields (Int, T) = todo

    /// The `i`-th element from the front. O(1).
    ///
    /// `q[i]` calls this; place `0` is the front and `len - 1` the back.
    ///
    /// cost: O(1)
    /// traps: when `i` is not in `0..len`
    /// example: `Deque.of([7, 8]).get(1) == 8`
    /// example: `var q = Deque.of([1]); q.push_front(0); q[0] == 0`
    pub fn get(self, i: Int) -> T = todo

    /// Overwrite the `i`-th element from the front (W8 `IndexSet` backing `dq[i] = x`). O(1).
    ///
    /// cost: O(1)
    /// traps: when `i` is not in `0..len`
    /// example: `var q = Deque.of([7, 8]); q[0] = 9; q.to_list() == [9, 8]`
    pub fn set(mut self, i: Int, x: T) = todo

    /// Materialize the elements front to back as a `List<T>`. O(n).
    ///
    /// cost: O(n), one new `List`
    /// empty: `[]`
    /// order: front to back; a new `List`, keyed from 0
    /// traps: never
    /// example: `var q = Deque.of([2]); q.push_front(1); q.to_list() == [1, 2]`
    pub fn to_list(self) -> List<T> = todo

    /// Yield elements front to back without allocating. O(n).
    ///
    /// `for x in q` uses this.
    ///
    /// cost: O(1) per step
    /// empty: yields nothing
    /// order: front to back
    /// traps: never
    /// example: `Deque.of([1, 2]).each().collect() == [1, 2]`
    pub fn each(self) -> yields T = todo

    /// Yield elements back to front without allocating. O(n).
    ///
    /// cost: O(1) per step
    /// empty: yields nothing
    /// order: back to front
    /// traps: never
    /// example: `Deque.of([1, 2]).each_rev().collect() == [2, 1]`
    pub fn each_rev(self) -> yields T = todo

    /// Rotate elements left by `k` positions (`k` mod `len`). O(min(k, n - k)).
    ///
    /// The element at place `k` becomes the front. `k` is reduced with Euclidean `%`, so a
    /// negative `k` rotates right: `rotate_left(-1)` is `rotate_right(1)`.
    ///
    /// cost: O(min(k, n - k)) after reducing `k` mod `n`
    /// empty: does nothing, for every `k`
    /// traps: never
    /// example: `var q = Deque.of([1, 2, 3, 4]); q.rotate_left(1); q.to_list() == [2, 3, 4, 1]`
    /// example: `var q = Deque.of([1, 2, 3, 4]); q.rotate_left(-1); q.to_list() == [4, 1, 2, 3]`
    pub fn rotate_left(mut self, k: Int) = todo

    /// Rotate elements right by `k` positions (`k` mod `len`). O(min(k, n - k)).
    ///
    /// The back `k` elements move to the front, in order. `rotate_right(k)` is
    /// `rotate_left(-k)`.
    ///
    /// cost: O(min(k, n - k)) after reducing `k` mod `n`
    /// empty: does nothing, for every `k`
    /// traps: never
    /// example: `var q = Deque.of([1, 2, 3, 4]); q.rotate_right(1); q.to_list() == [4, 1, 2, 3]`
    /// example: `var q = Deque.of([1, 2, 3]); q.rotate_right(4); q.to_list() == [3, 1, 2]`
    pub fn rotate_right(mut self, k: Int) = todo

/// Doubly-linked list supporting O(1) end pushes/pops and O(1) splicing (`splice_back`, `splice_front`).
pub struct LinkedList<T>:
    _len: Int

    /// Create an empty `LinkedList<T>`. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var l: LinkedList<Int> = LinkedList.new(); l.len == 0`
    pub fn new() -> LinkedList<T> = todo

    /// Create a singleton `LinkedList<T>` containing `x`. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `LinkedList.of(5).each().collect() == [5]`
    pub fn of(x: T) -> LinkedList<T> = todo

    /// Build a `LinkedList<T>` from `xs`, front to back. O(n).
    ///
    /// cost: O(n)
    /// empty: an empty list
    /// traps: never
    /// example: `LinkedList.from_list([1, 2]).each().collect() == [1, 2]`
    pub fn from_list(xs: List<T>) -> LinkedList<T> = todo

    /// Number of elements currently in the list. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `LinkedList.from_list([1, 2, 3]).len == 3`
    pub @get fn len(self) -> Int = todo

    /// Whether the list is empty. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `LinkedList.of(1).is_empty == false`
    pub @get fn is_empty(self) -> Bool = todo

    /// The front element (traps if empty). O(1).
    ///
    /// cost: O(1)
    /// ties: none: there is one front
    /// traps: when the list is empty
    /// example: `LinkedList.from_list([1, 2]).first == 1`
    pub @get fn first(self) -> T = todo

    /// The back element (traps if empty). O(1).
    ///
    /// cost: O(1)
    /// ties: none: there is one back
    /// traps: when the list is empty
    /// example: `LinkedList.from_list([1, 2]).last == 2`
    pub @get fn last(self) -> T = todo

    /// Append `x` to the back. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var l = LinkedList.of(1); l.push(2); l.each().collect() == [1, 2]`
    pub fn push(mut self, x: T) = todo

    /// Prepend `x` to the front. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var l = LinkedList.of(1); l.push_front(0); l.each().collect() == [0, 1]`
    pub fn push_front(mut self, x: T) = todo

    /// Remove and return the back element, or `None` if empty. O(1).
    ///
    /// cost: O(1)
    /// empty: `None`
    /// traps: never
    /// example: `var l = LinkedList.from_list([1, 2]); l.pop() == Some(2) and l.len == 1`
    pub fn pop(mut self) -> ?T = todo

    /// Remove and return the back element (alias for `pop`). O(1).
    ///
    /// cost: O(1)
    /// empty: `None`
    /// traps: never
    /// see: `pop`
    /// example: `var l = LinkedList.from_list([1, 2]); l.pop_back() == Some(2)`
    pub fn pop_back(mut self) -> ?T = todo

    /// Remove and return the front element, or `None` if empty. O(1).
    ///
    /// cost: O(1)
    /// empty: `None`
    /// traps: never
    /// example: `var l = LinkedList.from_list([1, 2]); l.pop_front() == Some(1) and l.first == 2`
    pub fn pop_front(mut self) -> ?T = todo

    /// Splice all elements of `other` onto the back of `self`, consuming `other`. O(1).
    ///
    /// The elements keep their order: `self`'s first, then `other`'s. No element is copied.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var a = LinkedList.from_list([1, 2]); a.splice_back(LinkedList.from_list([3, 4])); a.each().collect() == [1, 2, 3, 4]`
    pub fn splice_back(mut self, other: LinkedList<T>) = todo

    /// Splice all elements of `other` onto the front of `self`, consuming `other`. O(1).
    ///
    /// The elements keep their order: `other`'s first, then `self`'s.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var a = LinkedList.from_list([1, 2]); a.splice_front(LinkedList.from_list([3, 4])); a.each().collect() == [3, 4, 1, 2]`
    pub fn splice_front(mut self, other: LinkedList<T>) = todo

    /// Reverse the list in place. O(1) via reversible links.
    ///
    /// cost: O(1)
    /// empty: does nothing
    /// traps: never
    /// example: `var a = LinkedList.from_list([1, 2, 3]); a.reverse(); a.each().collect() == [3, 2, 1]`
    pub fn reverse(mut self) = todo

    /// Remove all elements. O(1).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var a = LinkedList.from_list([1, 2]); a.clear(); a.is_empty`
    pub fn clear(mut self) = todo

    /// Yield elements front to back. O(n).
    ///
    /// cost: O(1) per step
    /// empty: yields nothing
    /// order: front to back
    /// traps: never
    /// example: `LinkedList.from_list([1, 2]).each().collect() == [1, 2]`
    pub fn each(self) -> yields T = todo

    /// Yield elements back to front. O(n).
    ///
    /// cost: O(1) per step
    /// empty: yields nothing
    /// order: back to front
    /// traps: never
    /// example: `LinkedList.from_list([1, 2]).each_rev().collect() == [2, 1]`
    pub fn each_rev(self) -> yields T = todo

@test
fn a_deque_matches_a_list() = todo

@test
fn a_deque_as_a_bfs_queue() = todo

@test
fn a_deque_of_a_list_is_a_bfs_queue() = todo

@test
fn a_deque_reserve_preserves_order() = todo
