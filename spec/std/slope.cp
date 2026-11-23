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

/// O(log n) expected per slope operation, for n slopes.
/// A concave function on the integers `lo..=hi`: its value at `lo` and its
/// slopes in falling order. Law: every operation keeps it concave.
pub struct Concave(start: Int, base: Int):
    slopes: LazyTreap<Int, Slide>

    /// The function defined on the single point `d`, with `f(d) = 0`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var t = Concave.point(3); t.at(3) == 0`
    pub fn point(d: Int) -> Concave = Concave(d, 0, falling())

    /// The function that is 0 on every point of `r`; its domain is `r`, so `lo` is the first point of
    /// `r` and `hi` the last.
    ///
    /// cost: O(|r| log |r|) expected
    /// empty: traps
    /// traps: when `r` is empty
    /// example: `let f = Concave.flat(2..5); f.lo == 2 and f.hi == 4`
    /// example: `var f = Concave.flat(0..=1); f.max() == 0`
    pub fn flat(r: Range<Int>) -> Concave:
        var f = Concave.point(r.first!)
        for _ in 0..r.len - 1: f.slopes.insert(0)
        f

    /// Make room for `n` slopes in total, so later operations do not regrow the storage. Changes
    /// no value.
    ///
    /// cost: O(n)
    /// traps: when `n < 0`
    /// example: `var f = Concave.flat(0..=3); f.reserve(100); f.at(2) == 0`
    pub fn reserve(mut self, n: Int) = start

    /// The first point of the domain.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var f = Concave.flat(0..=3); f.shift(-2); f.lo == -2`
    @get pub fn lo(self) -> Int = start

    /// The last point of the domain.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var f = Concave.flat(0..=3); f.max_convolve(Concave.flat(0..=2)); f.hi == 5`
    @get pub fn hi(self) -> Int = start + slopes.len

    /// Replace `f` with `c => max over a + b == c of f(a) + g(b)`. The domain becomes
    /// `lo + g.lo ..= hi + g.hi` and the two slope multisets merge, so the result stays concave.
    /// Pass the smaller function as `g`: the cost is in its size.
    ///
    /// cost: O(|g| log n) expected, for `|g|` slopes in `g`
    /// ties: only the value is kept; which point attains it does not matter
    /// traps: never
    /// example: `var f = Concave.point(0); f.add_const(3); f.max_convolve(Concave.flat(0..=2)); f.hi == 2 and f.at(1) == 3`
    /// example: `var f = Concave.flat(0..=2); f.add_abs(1); var g = Concave.flat(0..=1); g.add_linear(-5); f.max_convolve(g); f.at(3) == -6`
    pub fn max_convolve(mut self, g: Concave):
        start += g.start
        base += g.base
        for s in g.slopes.to_list(): slopes.insert(s)

    /// `f(c) += q(c)` for a quadratic `q` with leading coefficient at most 0, so `f` stays concave.
    /// `q` is called at most four times: three points fix it and a fourth checks that it is
    /// quadratic.
    ///
    /// cost: O(1)
    /// traps: when `q` is not a quadratic polynomial on the points read, or its leading coefficient is positive
    /// example: `var f = Concave.flat(0..=3); f.add_quadratic(x => -x * x + 1); f.at(2) == -3`
    pub fn add_quadratic(mut self, q: Int -> Int):
        let (q0, q1, q2) = (q(start), q(start + 1), q(start + 2))
        let step = q2 - 2 * q1 + q0
        assert step <= 0
        assert q(start + 3) == q0 + 3 * (q2 - q1)
        base += q0
        slopes.apply_all(Slide(q1 - q0, step))

    /// The value `f(c)`.
    ///
    /// cost: O(log n) expected (O(c - lo + log n) in the current implementation)
    /// traps: when `c` is outside `lo..=hi`
    /// example: `var f = Concave.flat(0..=4); f.add_abs(1); f.at(4) == -3`
    pub fn at(mut self, c: Int) -> Int:
        assert start <= c and c <= start + slopes.len
        base + slopes.take(c - start).sum()

    /// The largest value of `f` over its domain.
    ///
    /// cost: O(log n) expected (O(n) in the current implementation)
    /// ties: only the value is kept; which point attains it does not matter
    /// traps: never
    /// example: `var f = Concave.flat(0..=4); f.add_abs(1); f.add_const(5); f.max() == 5`
    pub fn max(mut self) -> Int:
        var best = base
        for s in slopes.to_list():
            if s <= 0: break
            best += s
        best

    /// Add `min(0, x - a)`: a hinge with slope 1 left of `a` and flat from `a` on.
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `var f = Concave.flat(0..=4); f.add_left(2); f.at(0) == -2 and f.at(3) == 0`
    pub fn add_left(mut self, a: Int):
        let k = if a <= start: 0 else: min(a - start, slopes.len)
        base += min(0, start - a)
        slopes.apply_range(0, k, Slide(1, 0))
    /// Add `min(0, a - x)`: flat up to `a` and slope -1 right of it.
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `var f = Concave.flat(0..=4); f.add_right(2); f.at(4) == -2 and f.at(1) == 0`
    pub fn add_right(mut self, a: Int):
        let k = if a <= start: 0 else: min(a - start, slopes.len)
        base += min(0, a - start)
        slopes.apply_range(k, slopes.len, Slide(-1, 0))
    /// Add `-|x - a|`; `add_left(a)` and `add_right(a)` together.
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `var f = Concave.flat(0..=4); f.add_abs(1); f.at(0) == -1 and f.at(4) == -3`
    pub fn add_abs(mut self, a: Int):
        self.add_left(a)
        self.add_right(a)
    /// Add the constant `c` everywhere.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var f = Concave.point(2); f.add_const(-4); f.at(2) == -4`
    pub fn add_const(mut self, c: Int):
        base += c
    /// Add `k * x`, applied lazily to every slope.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var f = Concave.flat(0..=4); f.add_linear(2); f.at(3) == 6 and f.at(0) == 0`
    pub fn add_linear(mut self, k: Int):
        base += k * start
        slopes.apply_all(Slide(k, 0))
    /// Replace `f(x)` with the largest `f(y)` over `y <= x` in the domain; the domain is unchanged
    /// and every falling slope becomes 0.
    ///
    /// cost: O(log n) expected (O(n log n) in the current implementation)
    /// ties: only the value is kept; which point attains it does not matter
    /// traps: never
    /// example: `var f = Concave.flat(0..=4); f.add_abs(2); f.prefix_max(); f.at(0) == -2 and f.at(4) == 0`
    pub fn prefix_max(mut self):
        let k = self.count_at_least(0)
        let m = slopes.len - k
        slopes.erase_range(k..slopes.len)
        for _ in 0..m: slopes.insert(0)
    /// Replace `f(x)` with the largest `f(y)` over `y >= x` in the domain; the domain is unchanged
    /// and every rising slope becomes 0.
    ///
    /// cost: O(log n) expected (O(n log n) in the current implementation)
    /// ties: only the value is kept; which point attains it does not matter
    /// traps: never
    /// example: `var f = Concave.flat(0..=4); f.add_abs(2); f.suffix_max(); f.at(0) == 0 and f.at(4) == -2`
    pub fn suffix_max(mut self):
        let k = self.count_at_least(1)
        base = self.max()
        slopes.erase_range(0..k)
        for _ in 0..k: slopes.insert(0)
    /// Move the function right by `d`: the new `f(x)` is the old `f(x - d)`, and `lo` and `hi` both
    /// grow by `d`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var f = Concave.point(0); f.add_const(7); f.shift(5); f.lo == 5 and f.at(5) == 7`
    pub fn shift(mut self, d: Int):
        start += d
    /// Replace `f(x)` with the largest old `f(y)` over `x - b <= y <= x - a`; the domain becomes
    /// `lo + a ..= hi + b`. The same as `max_convolve(Concave.flat(a..=b))`.
    ///
    /// cost: O(b - a + log n) expected
    /// ties: only the value is kept; which point attains it does not matter
    /// traps: when `a > b`
    /// example: `var f = Concave.point(0); f.add_const(1); f.slide_max(-1, 2); f.lo == -1 and f.hi == 2 and f.at(2) == 1`
    pub fn slide_max(mut self, a: Int, b: Int):
        assert a <= b, "`slide_max` needs `a <= b`"
        start += a
        for _ in 0..b - a: slopes.insert(0)
    /// The points where `f` reaches its maximum, as a half-open range; by concavity they are
    /// contiguous.
    ///
    /// cost: O(log n) expected (O(n) in the current implementation)
    /// order: ascending points
    /// ties: every maximizing point is included
    /// traps: never
    /// example: `var f = Concave.flat(0..=4); f.add_abs(2); f.argmax() == 2..3`
    /// example: `var t = Concave.flat(0..=4); t.argmax() == 0..5`
    pub fn argmax(mut self) -> Range<Int> =
        start + self.count_at_least(1)..start + self.count_at_least(0) + 1

    fn count_at_least(self, v: Int) -> Int:
        var k = 0
        for s in slopes.to_list():
            if s < v: break
            k += 1
        k

/// O(log n) expected per slope operation, for n slopes.
/// A convex function on the integers `lo..=hi`, the negation of a `Concave`:
/// the same operations with `min` for `max`.
pub struct Convex(neg: Concave):
    /// The function defined on the single point `d`, with `f(d) = 0`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var t = Convex.point(3); t.at(3) == 0`
    pub fn point(d: Int) -> Convex = Convex(Concave.point(d))

    /// The function that is 0 on every point of `r`; its domain is `r`.
    ///
    /// cost: O(|r| log |r|) expected
    /// empty: traps
    /// traps: when `r` is empty
    /// example: `let f = Convex.flat(2..5); f.lo == 2 and f.hi == 4`
    pub fn flat(r: Range<Int>) -> Convex = Convex(Concave.flat(r))

    /// The first point of the domain.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var f = Convex.flat(0..=3); f.shift(-2); f.lo == -2`
    @get pub fn lo(self) -> Int = neg.lo

    /// The last point of the domain.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var f = Convex.flat(0..=3); f.min_convolve(Convex.flat(0..=2)); f.hi == 5`
    @get pub fn hi(self) -> Int = neg.hi

    /// Replace `f` with `c => min over a + b == c of f(a) + g(b)`. The domain becomes
    /// `lo + g.lo ..= hi + g.hi` and the slope multisets merge. Pass the smaller function as `g`.
    ///
    /// cost: O(|g| log n) expected, for `|g|` slopes in `g`
    /// ties: only the value is kept; which point attains it does not matter
    /// traps: never
    /// example: `var f = Convex.point(0); f.add_const(3); f.min_convolve(Convex.flat(0..=2)); f.hi == 2 and f.at(1) == 3`
    pub fn min_convolve(mut self, g: Convex) = neg.max_convolve(g.neg)

    /// `f(c) += q(c)` for a quadratic `q` with leading coefficient at least 0, so `f` stays convex;
    /// `q` is called at most four times, as in `Concave.add_quadratic`.
    ///
    /// cost: O(1)
    /// traps: when `q` is not a quadratic polynomial on the points read, or its leading coefficient is negative
    /// example: `var f = Convex.flat(0..=3); f.add_quadratic(x => x * x); f.at(3) == 9`
    pub fn add_quadratic(mut self, q: Int -> Int) = neg.add_quadratic(c => -q(c))

    /// The value `f(c)`.
    ///
    /// cost: O(log n) expected (O(c - lo + log n) in the current implementation)
    /// traps: when `c` is outside `lo..=hi`
    /// example: `var f = Convex.flat(0..=4); f.add_abs(1); f.at(4) == 3`
    pub fn at(mut self, c: Int) -> Int = -neg.at(c)

    /// The smallest value of `f` over its domain.
    ///
    /// cost: O(log n) expected (O(n) in the current implementation)
    /// ties: only the value is kept; which point attains it does not matter
    /// traps: never
    /// example: `var f = Convex.flat(0..=4); f.add_abs(1); f.add_const(5); f.min() == 5`
    pub fn min(mut self) -> Int = -neg.max()

    /// Add `max(0, a - x)`: slope -1 left of `a` and flat from `a` on.
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `var f = Convex.flat(0..=4); f.add_left(2); f.at(0) == 2 and f.at(3) == 0`
    pub fn add_left(mut self, a: Int) = neg.add_left(a)
    /// Add `max(0, x - a)`: flat up to `a` and slope 1 right of it.
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `var f = Convex.flat(0..=4); f.add_right(2); f.at(4) == 2 and f.at(1) == 0`
    pub fn add_right(mut self, a: Int) = neg.add_right(a)
    /// Add `|x - a|`; `add_left(a)` and `add_right(a)` together.
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `var f = Convex.flat(0..=4); f.add_abs(1); f.at(0) == 1 and f.at(4) == 3`
    pub fn add_abs(mut self, a: Int) = neg.add_abs(a)
    /// Add the constant `c` everywhere.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var f = Convex.point(2); f.add_const(-4); f.at(2) == -4`
    pub fn add_const(mut self, c: Int) = neg.add_const(-c)
    /// Add `k * x`, applied lazily to every slope.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var f = Convex.flat(0..=4); f.add_linear(-2); f.at(3) == -6`
    pub fn add_linear(mut self, k: Int) = neg.add_linear(-k)
    /// Replace `f(x)` with the smallest `f(y)` over `y <= x` in the domain; the domain is unchanged.
    ///
    /// cost: O(log n) expected (O(n log n) in the current implementation)
    /// ties: only the value is kept; which point attains it does not matter
    /// traps: never
    /// example: `var f = Convex.flat(0..=4); f.add_abs(2); f.prefix_min(); f.at(0) == 2 and f.at(4) == 0`
    pub fn prefix_min(mut self) = neg.prefix_max()
    /// Replace `f(x)` with the smallest `f(y)` over `y >= x` in the domain; the domain is unchanged.
    ///
    /// cost: O(log n) expected (O(n log n) in the current implementation)
    /// ties: only the value is kept; which point attains it does not matter
    /// traps: never
    /// example: `var f = Convex.flat(0..=4); f.add_abs(2); f.suffix_min(); f.at(0) == 0 and f.at(4) == 2`
    pub fn suffix_min(mut self) = neg.suffix_max()
    /// Move the function right by `d`: the new `f(x)` is the old `f(x - d)`, and `lo` and `hi` both
    /// grow by `d`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var f = Convex.point(0); f.add_const(7); f.shift(5); f.lo == 5 and f.at(5) == 7`
    pub fn shift(mut self, d: Int) = neg.shift(d)
    /// Replace `f(x)` with the smallest old `f(y)` over `x - b <= y <= x - a`; the domain becomes
    /// `lo + a ..= hi + b`. The same as `min_convolve(Convex.flat(a..=b))`.
    ///
    /// cost: O(b - a + log n) expected
    /// ties: only the value is kept; which point attains it does not matter
    /// traps: when `a > b`
    /// example: `var f = Convex.point(0); f.add_const(1); f.slide_min(-1, 2); f.lo == -1 and f.hi == 2 and f.at(-1) == 1`
    pub fn slide_min(mut self, a: Int, b: Int) = neg.slide_max(a, b)
    /// The points where `f` reaches its minimum, as a half-open range; by convexity they are
    /// contiguous.
    ///
    /// cost: O(log n) expected (O(n) in the current implementation)
    /// order: ascending points
    /// ties: every minimizing point is included
    /// traps: never
    /// example: `var f = Convex.flat(0..=4); f.add_abs(2); f.argmin() == 2..3`
    pub fn argmin(mut self) -> Range<Int> = neg.argmax()

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
