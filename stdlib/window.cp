//! Sliding-window folds in O(1) amortized time.
//!
//! `MonoDeque<T>` maintains the minimum or maximum of a sliding window;
//! `WindowFold<T>` (the two-stack trick) maintains the fold of any monoid without inverses
//! (such as `gcd`, bitwise `|`, or matrix products).
use std.algebra.{Monoid}
use std.deque.{Deque}
use std.math.{gcd}

/// Sliding-window min or max over positions pushed in monotone order.
/// `push`, `drop_before`, `drop_after` are O(1) amortized; `best`, `best_at`, `len`, `is_empty`, `clear` are O(1).
pub struct MonoDeque<T>(by: (T, T) -> T):
    q: Deque<(Int, T)>

    /// A window keeping the `min`, `max`, or any binary selector `by` of active elements. O(1).
    pub fn new(by: (T, T) -> T = min) -> MonoDeque<T> = MonoDeque(by, Deque())

    /// Number of elements currently in the monotonic deque. O(1).
    pub @get fn len(self) -> Int = q.len

    /// Whether the monotonic deque is empty. O(1).
    pub @get fn is_empty(self) -> Bool = q.is_empty

    /// Best value currently in the window, or `none` when empty. O(1).
    pub @get fn best(self) -> ?T = if let f = q.first: f.1 else: none

    /// Position of the best value currently in the window, or `none` when empty. O(1).
    pub @get fn best_at(self) -> ?Int = if let f = q.first: f.0 else: none

    /// Remove all elements from the monotonic deque. O(1).
    pub fn clear(mut self):
        q.clear()

    /// Push `(at, val)` at the right end of the window (`at` non-decreasing). O(1) amortized.
    pub fn push(mut self, at: Int, val: T):
        while not q.is_empty and by(q.last!.1, val) == val:
            let _ = q.pop()
        q.push((at, val))

    /// Drop every entry with position strictly less than `min_at` (for left-to-right sweeps). O(1) amortized.
    pub fn drop_before(mut self, min_at: Int):
        while not q.is_empty and q.first!.0 < min_at:
            let _ = q.pop_front()

    /// Drop every entry with position strictly greater than `max_at` (for right-to-left sweeps). O(1) amortized.
    pub fn drop_after(mut self, max_at: Int):
        while not q.is_empty and q.first!.0 > max_at:
            let _ = q.pop_front()

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
            let _ = cat.pop()
            assert got == (if plain.is_empty: none else: plain[0])
            if not plain.is_empty: plain = [plain[i] for i in 1..plain.len]
        assert q.len == plain.len and q.is_empty == plain.is_empty
        var g = 0
        var s = ""
        for x in plain:
            g = gcd(g, x)
            s += Str(x)
        assert q.fold == g and cat.fold == s
