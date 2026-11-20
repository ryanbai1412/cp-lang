//! A queue that knows the fold of everything in it, for any monoid: the "two stacks" trick.
//!
//! This is the sliding window for operations with no inverse (gcd, bitwise or, matrix
//! products). `MonoDeque` (`ds/mono_deque.cp`) covers min and max; a `Fenwick` covers sums.
use std.algebra.{Monoid}
use std.math.{gcd}

/// A FIFO queue with `fold` in O(1) and `push` / `pop` in O(1) amortized.
/// The fold is in queue order, so the operation need not be commutative.
pub struct TwoStackFold<T>(m: Monoid<T>):
    /// The back half, oldest first, and its fold.
    back: List<T>
    back_fold: T
    /// The front half, newest first: `front[i]` is (value, fold of it and everything behind it in the front half).
    front: List<(T, T)>

    /// An empty queue for a bare operator (`gcd`, `|`, `+`) or a `Monoid`.
    pub fn new<M>(op: M) -> TwoStackFold<T>:
        let m = Monoid<T>.of(op)
        TwoStackFold(m, [], m.id, [])

    pub @get fn len(self) -> Int = back.len + front.len

    pub fn push(mut self, x: T):
        back.push(x)
        back_fold = m.op(back_fold, x)

    /// Remove and return the oldest value, or `none` when empty.
    pub fn pop(mut self) -> ?T:
        if front.is_empty:
            while not back.is_empty:
                let x = back.pop()!
                let under = if front.is_empty: m.id else: front[front.len - 1].1
                front.push((x, m.op(x, under)))
            back_fold = m.id
        if front.is_empty: return none
        front.pop()!.0

    /// The fold of the whole queue, oldest to newest; the identity when empty.
    pub @get fn fold(self) -> T:
        let head = if front.is_empty: m.id else: front[front.len - 1].1
        m.op(head, back_fold)

@test
fn matches_folding_the_window():
    for _ in 0..30:
        var q = TwoStackFold<Int>(gcd)
        var cat = TwoStackFold<Str>(Monoid<Str>("", (a, b) => a + b))
        var plain: List<Int> = []
        for _ in 0..40:
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
            assert q.len == plain.len
            var g = 0
            var s = ""
            for x in plain:
                g = gcd(g, x)
                s = s + Str(x)
            assert q.fold == g and cat.fold == s

@test
fn shortest_window_with_gcd_one():
    let a = [6, 10, 15, 4, 9, 12]
    var q = TwoStackFold<Int>(gcd)
    var best: ?Int = none
    for x in a:
        q.push(x)
        while q.len > 1 and q.fold == 1:
            best min= q.len
            q.pop()
    assert best == 2            // [15, 4]
