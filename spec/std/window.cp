//! Sliding-window folds and stateful Mo/D&C windows.
//!
//! `MonoDeque<T>` maintains the minimum or maximum of a sliding window;
//! `WindowFold<T>` (the two-stack trick) maintains the fold of any monoid without inverses
//! (such as `gcd`, bitwise `|`, or matrix products);
//! `Window<D>` maintains a stateful sliding interval over index domain `D` via `enter`/`leave` callbacks.
use std.algebra.{Monoid}
use std.math.{gcd}

/// Sliding-window min or max over positions pushed in monotone order.
/// `push`, `drop_before`, `drop_after` are O(1) amortized; `best`, `best_at`, `len`, `is_empty`, `clear` are O(1).
pub struct MonoDeque<T>(by: (T, T) -> T):
    q: List<(Int, T)>
    head: Int

    /// A window keeping the `min`, `max`, or any binary selector `by` of active elements. O(1).
    pub fn new(by: (T, T) -> T = min) -> MonoDeque<T> = todo

    /// Number of elements currently in the monotonic deque. O(1).
    pub @get fn len(self) -> Int = todo

    /// Whether the monotonic deque is empty. O(1).
    pub @get fn is_empty(self) -> Bool = todo

    /// Best value currently in the window, or `none` when empty. O(1).
    pub @get fn best(self) -> ?T = todo

    /// Position of the best value currently in the window, or `none` when empty. O(1).
    pub @get fn best_at(self) -> ?Int = todo

    /// Remove all elements from the monotonic deque. O(1).
    pub fn clear(mut self) = todo

    fn compact(mut self) = todo

    /// Push `(at, val)` at the right end of the window (`at` non-decreasing). O(1) amortized.
    pub fn push(mut self, at: Int, val: T) = todo

    /// Drop every entry with position strictly less than `min_at` (for left-to-right sweeps). O(1) amortized.
    pub fn drop_before(mut self, min_at: Int) = todo

    /// Drop every entry with position strictly greater than `max_at` (for right-to-left sweeps). O(1) amortized.
    pub fn drop_after(mut self, max_at: Int) = todo

    /// O(1). Drop the front entry if its position equals `at`.
    pub fn drop(mut self, at: Int) = todo

    /// O(1). Remove and return the front `(at, val)` pair, or `none` when empty.
    pub fn pop_front(mut self) -> ?(at: Int, val: T) = todo

    /// O(1). Peek the front `(at, val)` pair without removing it, or `none` when empty.
    pub @get fn peek_front(self) -> ?(at: Int, val: T) = todo

/// FIFO queue supporting O(1) monoid fold of all elements without inverses (two-stack trick).
/// `push` and `fold` are O(1); `pop` is O(1) amortized. Keeps queue order for non-commutative monoids.
pub struct WindowFold<T>(m: Monoid<T>):
    back: List<T>
    back_fold: T
    front: List<(T, T)>

    /// An empty queue for a bare operator (`gcd`, `|`, `+`) or a `Monoid`. O(1).
    pub fn new<M>(op: M) -> WindowFold<T> = todo

    /// Number of elements currently in the sliding-window fold. O(1).
    pub @get fn len(self) -> Int = todo

    /// Whether the sliding-window fold is empty. O(1).
    pub @get fn is_empty(self) -> Bool = todo

    /// Push `x` onto the back of the queue. O(1).
    pub fn push(mut self, x: T) = todo

    /// Remove and return the oldest value, or `none` when empty. O(1) amortized.
    pub fn pop(mut self) -> ?T = todo

    /// Fold of the whole queue, oldest to newest, or `m.id` when empty. O(1).
    pub @get fn fold(self) -> T = todo

    /// O(1). Remove all elements from the queue.
    pub fn clear(mut self) = todo

    /// O(1) amortized. Peek the oldest element without removing it, or `none` when empty.
    pub @get fn peek_front(self) -> ?T = todo

/// O(n) space. Double-ended sliding-window monoid fold (SWAG / two-stack deque) without inverses:
/// O(1) amortized `push_front`, `push_back`, `pop_front`, `pop_back`; O(1) `fold`.
pub struct DequeFold<T>(m: Monoid<T>):
    n: Int

    /// O(1). Create an empty double-ended fold deque for a bare operator or `Monoid<T>`.
    pub fn new<M>(op: M) -> DequeFold<T> = todo

    /// O(1). Number of elements currently in the deque.
    pub @get fn len(self) -> Int = todo

    /// O(1). Whether the deque is empty.
    pub @get fn is_empty(self) -> Bool = todo

    /// O(1). Push `x` onto the front of the deque.
    pub fn push_front(mut self, x: T) = todo

    /// O(1). Push `x` onto the back of the deque.
    pub fn push_back(mut self, x: T) = todo

    /// O(1) amortized. Remove and return the front element, or `none` when empty.
    pub fn pop_front(mut self) -> ?T = todo

    /// O(1) amortized. Remove and return the back element, or `none` when empty.
    pub fn pop_back(mut self) -> ?T = todo

    /// O(1). Fold of the whole deque, front to back, or `m.id` when empty.
    pub @get fn fold(self) -> T = todo

    /// O(1). Remove all elements from the deque.
    pub fn clear(mut self) = todo

/// O(|D|). Yield the monoid fold of every contiguous length-`k` window of `a` in order.
pub fn sliding_fold<D, T, M>(a: [D]T, k: Int, op: M) -> yields T = todo

/// O(1) space. Stateful sliding window over index domain `D` backed by `enter`/`leave` callbacks
/// (for Mo's algorithm and divide-and-conquer DP). Always expands before shrinking so the active
/// interval is never inverted.
pub struct Window<D>(enter: D -> (), leave: D -> ()):
    lo: Int
    hi: Int

    /// O(1). Create a window initialized to `start` (or the empty window at `D.first.l..D.first.l`).
    pub fn new(enter: D -> (), leave: D -> (), start: ?Range<D> = none) -> Window<D> = todo

    /// O(1). Current active half-open window.
    pub @get fn range(self) -> Range<D> = todo

    /// O(1). Number of positions currently inside the window.
    pub @get fn len(self) -> Int = todo

    /// O(1). Whether the active window is empty.
    pub @get fn is_empty(self) -> Bool = todo

    /// O(|dl| + |dr|) `enter`/`leave` calls. Slide the active window to `to`, expanding before shrinking.
    pub fn move(mut self, to: Range<D>) = todo

    /// O(len) `leave` calls. Empty the active window back to `D.first.l..D.first.l`.
    pub fn reset(mut self) = todo

@test
fn mono_deque_matches_sliding_window_brute() = todo

@test
fn window_fold_matches_brute_for_gcd_and_concat() = todo
