//! Integer-indexed concave and convex functions as sorted slopes (slope trick).
//!
//! `Concave` keeps `f` on `lo..=hi` as `f(lo)` plus the multiset of slopes
//! `f(c + 1) - f(c)`, largest first, in a lazy treap; `Convex` is its mirror.
use std.lazy.{LazyTreap}

struct Slide(base: Int, step: Int)

fn falling() -> LazyTreap<Int, Slide> = LazyTreap<Int, Slide>(
    apply: (v, left, f) => v + f.base + left * f.step,
    compose: (f, g) => Slide(f.base + g.base, f.step + g.step),
    split: (f, left) => (f, Slide(f.base + (left + 1) * f.step, f.step)),
    before: (x, y) => x >= y,
)

/// A concave function on the integers `lo..=hi`: its value at `lo` and its
/// slopes in falling order. Law: every operation keeps it concave.
pub struct Concave(start: Int, base: Int):
    slopes: LazyTreap<Int, Slide>

    /// `f(d) = 0` on the single point `d`. O(1).
    pub fn point(d: Int) -> Concave = Concave(d, 0, falling())

    /// `f(c) = 0` on `r`. O(len log len) expected.
    pub fn flat(r: Range<Int>) -> Concave:
        var f = Concave.point(r.first!)
        for _ in 0..r.len - 1: f.slopes.insert(0)
        f

    /// Make room for `n` slopes in total without regrowing. O(n).
    pub fn reserve(mut self, n: Int): slopes.reserve(n)

    /// The first point of the domain. O(1).
    pub @get fn lo(self) -> Int = start

    /// The last point of the domain. O(1).
    pub @get fn hi(self) -> Int = start + slopes.len

    /// `f = max over a + b = c of f(a) + g(b)`: the domains add and the slope
    /// multisets merge. O(|g| log n) expected; convolve the smaller into the larger.
    pub fn max_convolve(mut self, g: Concave):
        start += g.start
        base += g.base
        for s in g.slopes.to_list(): slopes.insert(s)

    /// `f(c) += q(c)` for a quadratic `q` with leading coefficient at most 0,
    /// read at three points (a fourth checks it is quadratic). O(1).
    pub fn add_quadratic(mut self, q: Int -> Int):
        let (q0, q1, q2) = (q(start), q(start + 1), q(start + 2))
        let step = q2 - 2 * q1 + q0
        assert step <= 0
        assert q(start + 3) == q0 + 3 * (q2 - q1)
        base += q0
        slopes.apply_all(Slide(q1 - q0, step))

    /// `f(c)` for `c` in `lo..=hi`. O(c - lo + log n) expected.
    pub fn at(mut self, c: Int) -> Int:
        assert start <= c and c <= start + slopes.len
        base + slopes.take(c - start).sum()

    /// The largest value of `f`. O(n).
    pub fn max(mut self) -> Int:
        var best = base
        for s in slopes.to_list():
            if s <= 0: break
            best += s
        best

/// A convex function on the integers `lo..=hi`, the negation of a `Concave`:
/// the same operations with `min` for `max`.
pub struct Convex(neg: Concave):
    /// `f(d) = 0` on the single point `d`. O(1).
    pub fn point(d: Int) -> Convex = Convex(Concave.point(d))

    /// `f(c) = 0` on `r`. O(len log len) expected.
    pub fn flat(r: Range<Int>) -> Convex = Convex(Concave.flat(r))

    /// The first point of the domain. O(1).
    pub @get fn lo(self) -> Int = neg.lo

    /// The last point of the domain. O(1).
    pub @get fn hi(self) -> Int = neg.hi

    /// `f = min over a + b = c of f(a) + g(b)`. O(|g| log n) expected.
    pub fn min_convolve(mut self, g: Convex): neg.max_convolve(g.neg)

    /// `f(c) += q(c)` for a quadratic `q` with leading coefficient at least 0. O(1).
    pub fn add_quadratic(mut self, q: Int -> Int): neg.add_quadratic(c => -q(c))

    /// `f(c)` for `c` in `lo..=hi`. O(c - lo + log n) expected.
    pub fn at(mut self, c: Int) -> Int = -neg.at(c)

    /// The smallest value of `f`. O(n).
    pub fn min(mut self) -> Int = -neg.max()

@test
fn concave_matches_a_table_dp():
    for _ in 0..200:
        var f = Concave.point(rand(-3, 3))
        var lo = f.lo
        var t = [0]
        for _ in 0..rand(1, 12):
            let pick = rand(0, 2)
            if pick == 0:
                let (a, b) = (rand(-3, 0), rand(-20, 20))
                let c0 = rand(-30, 30)
                fn q(c: Int) -> Int = a * c * c + b * c + c0
                f.add_quadratic(q)
                t = [t[i] + q(lo + i) for i in 0..t.len]
            else:
                let (d, w) = (rand(-2, 2), rand(0, 3))
                f.max_convolve(Concave.flat(d..=d + w))
                lo += d
                var u = [Int.MIN for _ in 0..t.len + w]
                for i in 0..t.len:
                    for j in 0..=w: u[i + j] = max(u[i + j], t[i])
                t = u
        assert f.lo == lo and f.hi == lo + t.len - 1
        for i in 0..t.len: assert f.at(lo + i) == t[i]
        assert f.max() == t.max()!

@test
fn convex_mirrors_concave():
    for _ in 0..100:
        var f = Convex.point(0)
        var g = Concave.point(0)
        for _ in 0..rand(1, 8):
            let (a, b, w) = (rand(0, 3), rand(-9, 9), rand(0, 2))
            f.add_quadratic(c => a * c * c + b * c)
            g.add_quadratic(c => -(a * c * c + b * c))
            f.min_convolve(Convex.flat(0..=w))
            g.max_convolve(Concave.flat(0..=w))
        for c in f.lo..=f.hi: assert f.at(c) == -g.at(c)
        assert f.min() == -g.max()
