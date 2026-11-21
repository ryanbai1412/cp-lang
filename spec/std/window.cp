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
    pub fn new(by: (T, T) -> T = min) -> MonoDeque<T> = MonoDeque(by, [], 0)

    /// Number of elements currently in the monotonic deque. O(1).
    pub @get fn len(self) -> Int = q.len - head

    /// Whether the monotonic deque is empty. O(1).
    pub @get fn is_empty(self) -> Bool = head == q.len

    /// Best value currently in the window, or `none` when empty. O(1).
    pub @get fn best(self) -> ?T:
        if head == q.len: return none
        unchecked: return q[head].1

    /// Position of the best value currently in the window, or `none` when empty. O(1).
    pub @get fn best_at(self) -> ?Int:
        if head == q.len: return none
        unchecked: return q[head].0

    /// Remove all elements from the monotonic deque. O(1).
    pub fn clear(mut self):
        q.clear()
        head = 0

    fn compact(mut self):
        if head == q.len:
            q.clear()
            head = 0
        elif head >= 65536 and head * 2 >= q.len:
            let rem = q.len - head
            unchecked:
                for i in 0..rem:
                    q[i] = q[head + i]
            q.truncate(rem)
            head = 0

    /// Push `(at, val)` at the right end of the window (`at` non-decreasing). O(1) amortized.
    pub fn push(mut self, at: Int, val: T):
        var sz = q.len
        unchecked:
            while sz > head and by(q[sz - 1].1, val) == val:
                sz -= 1
        if sz < q.len: q.truncate(sz)
        q.push((at, val))

    /// Drop every entry with position strictly less than `min_at` (for left-to-right sweeps). O(1) amortized.
    pub fn drop_before(mut self, min_at: Int):
        unchecked:
            while head < q.len and q[head].0 < min_at:
                head += 1
        if head == q.len or (head >= 65536 and head * 2 >= q.len):
            self.compact()

    /// Drop every entry with position strictly greater than `max_at` (for right-to-left sweeps). O(1) amortized.
    pub fn drop_after(mut self, max_at: Int):
        unchecked:
            while head < q.len and q[head].0 > max_at:
                head += 1
        if head == q.len or (head >= 65536 and head * 2 >= q.len):
            self.compact()

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
    pub fn new<M>(op: M) -> WindowFold<T>:
        let m = Monoid<T>.of(op)
        WindowFold(m, [], m.id, [])

    /// Number of elements currently in the sliding-window fold. O(1).
    pub @get fn len(self) -> Int = back.len + front.len

    /// Whether the sliding-window fold is empty. O(1).
    pub @get fn is_empty(self) -> Bool = back.is_empty and front.is_empty

    /// Push `x` onto the back of the queue. O(1).
    pub fn push(mut self, x: T):
        back.push(x)
        back_fold = m.op(back_fold, x)

    /// Remove and return the oldest value, or `none` when empty. O(1) amortized.
    pub fn pop(mut self) -> ?T:
        if front.is_empty:
            while not back.is_empty:
                let x = back.pop()!
                let under = if front.is_empty: m.id else: front.last!.1
                front.push((x, m.op(x, under)))
            back_fold = m.id
        if front.is_empty: return none
        front.pop()!.0

    /// Fold of the whole queue, oldest to newest, or `m.id` when empty. O(1).
    pub @get fn fold(self) -> T:
        let head = if front.is_empty: m.id else: front.last!.1
        m.op(head, back_fold)

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
fn mono_deque_matches_sliding_window_brute():
    let a = [rand(-20, 20) for _ in 0..30]
    let k = 5
    var mn = MonoDeque<Int>(min)
    var mx = MonoDeque<Int>(by: max)
    assert mn.best == none and mn.best_at == none
    for i in 0..a.len:
        mn.push(i, a[i])
        mx.push(i, a[i])
        mn.drop_before(i - k + 1)
        mx.drop_before(i - k + 1)
        let lo = max(0, i - k + 1)
        let want_mn = min(a[j] for j in lo..=i)
        let want_mx = max(a[j] for j in lo..=i)
        assert mn.best == want_mn and a[mn.best_at!] == want_mn and mn.best_at! >= lo
        assert mx.best == want_mx and a[mx.best_at!] == want_mx and mx.best_at! >= lo
    var rtl = MonoDeque<Int>(by: min)
    for i in (0..a.len).rev:
        rtl.push(i, a[i])
        rtl.drop_after(i + k - 1)
        let hi = min(a.len - 1, i + k - 1)
        let want = min(a[j] for j in i..=hi)
        assert rtl.best == want and a[rtl.best_at!] == want and rtl.best_at! <= hi

@test
fn window_fold_matches_brute_for_gcd_and_concat():
    var q = WindowFold<Int>(gcd)
    var cat = WindowFold<Str>(Monoid<Str>("", (a, b) => a + b))
    var plain: List<Int> = []
    for _ in 0..60:
        if rand(0, 2) > 0:
            let x = rand(1, 60)
            q.push(x)
            cat.push(Str(x))
            plain.push(x)
        else:
            let got = q.pop()
            cat.pop()
            assert got == (if plain.is_empty: none else: plain[0])
            if not plain.is_empty: plain = [plain[i] for i in 1..plain.len]
        assert q.len == plain.len and q.is_empty == plain.is_empty
        var g = 0
        var s = ""
        for x in plain:
            g = gcd(g, x)
            s += Str(x)
        assert q.fold == g and cat.fold == s
