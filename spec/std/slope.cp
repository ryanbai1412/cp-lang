//! Integer-indexed concave and convex functions as sorted slopes (slope trick).
//!
//! `Concave` keeps `f` on `lo..=hi` as `f(lo)` plus the multiset of slopes
//! `f(c + 1) - f(c)`, largest first, in a lazy treap; `Convex` is its mirror.
use std.lazy.{LazyTreap}

struct Slide(base: Int, step: Int)

fn falling() -> LazyTreap<Int, Slide> = todo

/// O(log n) expected per slope operation, for n slopes.
/// A concave function on the integers `lo..=hi`: its value at `lo` and its
/// slopes in falling order. Law: every operation keeps it concave.
pub struct Concave(start: Int, base: Int):
    slopes: LazyTreap<Int, Slide>

    /// The function defined on the single point `d`, with `f(d) = 0`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Concave.point(3).at(3) == 0`
    pub fn point(d: Int) -> Concave = todo

    /// The function that is 0 on every point of `r`; its domain is `r`, so `lo` is the first point of
    /// `r` and `hi` the last.
    ///
    /// cost: O(|r| log |r|) expected
    /// empty: traps
    /// traps: when `r` is empty
    /// example: `let f = Concave.flat(2..5); f.lo == 2 and f.hi == 4`
    /// example: `var f = Concave.flat(0..=1); f.max() == 0`
    pub fn flat(r: Range<Int>) -> Concave = todo

    /// Make room for `n` slopes in total, so later operations do not regrow the storage. Changes
    /// no value.
    ///
    /// cost: O(n)
    /// traps: when `n < 0`
    /// example: `var f = Concave.flat(0..=3); f.reserve(100); f.at(2) == 0`
    pub fn reserve(mut self, n: Int) = todo

    /// The first point of the domain.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var f = Concave.flat(0..=3); f.shift(-2); f.lo == -2`
    @get pub fn lo(self) -> Int = todo

    /// The last point of the domain.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var f = Concave.flat(0..=3); f.max_convolve(Concave.flat(0..=2)); f.hi == 5`
    @get pub fn hi(self) -> Int = todo

    /// Replace `f` with `c => max over a + b == c of f(a) + g(b)`. The domain becomes
    /// `lo + g.lo ..= hi + g.hi` and the two slope multisets merge, so the result stays concave.
    /// Pass the smaller function as `g`: the cost is in its size.
    ///
    /// cost: O(|g| log n) expected, for `|g|` slopes in `g`
    /// ties: only the value is kept; which point attains it does not matter
    /// traps: never
    /// example: `var f = Concave.point(0); f.add_const(3); f.max_convolve(Concave.flat(0..=2)); f.hi == 2 and f.at(1) == 3`
    /// example: `var f = Concave.flat(0..=2); f.add_abs(1); var g = Concave.flat(0..=1); g.add_linear(-5); f.max_convolve(g); f.at(3) == -6`
    pub fn max_convolve(mut self, g: Concave) = todo

    /// `f(c) += q(c)` for a quadratic `q` with leading coefficient at most 0, so `f` stays concave.
    /// `q` is called at most four times: three points fix it and a fourth checks that it is
    /// quadratic.
    ///
    /// cost: O(1)
    /// traps: when `q` is not a quadratic polynomial on the points read, or its leading coefficient is positive
    /// example: `var f = Concave.flat(0..=3); f.add_quadratic(x => -x * x + 1); f.at(2) == -3`
    pub fn add_quadratic(mut self, q: Int -> Int) = todo

    /// The value `f(c)`.
    ///
    /// cost: O(log n) expected (O(c - lo + log n) in the current implementation)
    /// traps: when `c` is outside `lo..=hi`
    /// example: `var f = Concave.flat(0..=4); f.add_abs(1); f.at(4) == -3`
    pub fn at(mut self, c: Int) -> Int = todo

    /// The largest value of `f` over its domain.
    ///
    /// cost: O(log n) expected (O(n) in the current implementation)
    /// ties: only the value is kept; which point attains it does not matter
    /// traps: never
    /// example: `var f = Concave.flat(0..=4); f.add_abs(1); f.add_const(5); f.max() == 5`
    pub fn max(mut self) -> Int = todo

    /// Add `min(0, x - a)`: a hinge with slope 1 left of `a` and flat from `a` on.
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `var f = Concave.flat(0..=4); f.add_left(2); f.at(0) == -2 and f.at(3) == 0`
    pub fn add_left(mut self, a: Int) = todo
    /// Add `min(0, a - x)`: flat up to `a` and slope -1 right of it.
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `var f = Concave.flat(0..=4); f.add_right(2); f.at(4) == -2 and f.at(1) == 0`
    pub fn add_right(mut self, a: Int) = todo
    /// Add `-|x - a|`; `add_left(a)` and `add_right(a)` together.
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `var f = Concave.flat(0..=4); f.add_abs(1); f.at(0) == -1 and f.at(4) == -3`
    pub fn add_abs(mut self, a: Int) = todo
    /// Add the constant `c` everywhere.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var f = Concave.point(2); f.add_const(-4); f.at(2) == -4`
    pub fn add_const(mut self, c: Int) = todo
    /// Add `k * x`, applied lazily to every slope.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var f = Concave.flat(0..=4); f.add_linear(2); f.at(3) == 6 and f.at(0) == 0`
    pub fn add_linear(mut self, k: Int) = todo
    /// Replace `f(x)` with the largest `f(y)` over `y <= x` in the domain; the domain is unchanged
    /// and every falling slope becomes 0.
    ///
    /// cost: O(log n) expected
    /// ties: only the value is kept; which point attains it does not matter
    /// traps: never
    /// example: `var f = Concave.flat(0..=4); f.add_abs(2); f.prefix_max(); f.at(0) == -2 and f.at(4) == 0`
    pub fn prefix_max(mut self) = todo
    /// Replace `f(x)` with the largest `f(y)` over `y >= x` in the domain; the domain is unchanged
    /// and every rising slope becomes 0.
    ///
    /// cost: O(log n) expected
    /// ties: only the value is kept; which point attains it does not matter
    /// traps: never
    /// example: `var f = Concave.flat(0..=4); f.add_abs(2); f.suffix_max(); f.at(0) == 0 and f.at(4) == -2`
    pub fn suffix_max(mut self) = todo
    /// Move the function right by `d`: the new `f(x)` is the old `f(x - d)`, and `lo` and `hi` both
    /// grow by `d`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var f = Concave.point(0); f.add_const(7); f.shift(5); f.lo == 5 and f.at(5) == 7`
    pub fn shift(mut self, d: Int) = todo
    /// Replace `f(x)` with the largest old `f(y)` over `x - b <= y <= x - a`; the domain becomes
    /// `lo + a ..= hi + b`. The same as `max_convolve(Concave.flat(a..=b))`.
    ///
    /// cost: O(log n) expected
    /// ties: only the value is kept; which point attains it does not matter
    /// traps: when `a > b`
    /// example: `var f = Concave.point(0); f.add_const(1); f.slide_max(-1, 2); f.lo == -1 and f.hi == 2 and f.at(2) == 1`
    pub fn slide_max(mut self, a: Int, b: Int) = todo
    /// The points where `f` reaches its maximum, as a half-open range; by concavity they are
    /// contiguous.
    ///
    /// cost: O(log n) expected
    /// order: ascending points
    /// ties: every maximizing point is included
    /// traps: never
    /// example: `var f = Concave.flat(0..=4); f.add_abs(2); f.argmax() == 2..3`
    /// example: `Concave.flat(0..=4).argmax() == 0..5`
    pub fn argmax(mut self) -> Range<Int> = todo

/// O(log n) expected per slope operation, for n slopes.
/// A convex function on the integers `lo..=hi`, the negation of a `Concave`:
/// the same operations with `min` for `max`.
pub struct Convex(neg: Concave):
    /// The function defined on the single point `d`, with `f(d) = 0`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Convex.point(3).at(3) == 0`
    pub fn point(d: Int) -> Convex = todo

    /// The function that is 0 on every point of `r`; its domain is `r`.
    ///
    /// cost: O(|r| log |r|) expected
    /// empty: traps
    /// traps: when `r` is empty
    /// example: `let f = Convex.flat(2..5); f.lo == 2 and f.hi == 4`
    pub fn flat(r: Range<Int>) -> Convex = todo

    /// The first point of the domain.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var f = Convex.flat(0..=3); f.shift(-2); f.lo == -2`
    @get pub fn lo(self) -> Int = todo

    /// The last point of the domain.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var f = Convex.flat(0..=3); f.min_convolve(Convex.flat(0..=2)); f.hi == 5`
    @get pub fn hi(self) -> Int = todo

    /// Replace `f` with `c => min over a + b == c of f(a) + g(b)`. The domain becomes
    /// `lo + g.lo ..= hi + g.hi` and the slope multisets merge. Pass the smaller function as `g`.
    ///
    /// cost: O(|g| log n) expected, for `|g|` slopes in `g`
    /// ties: only the value is kept; which point attains it does not matter
    /// traps: never
    /// example: `var f = Convex.point(0); f.add_const(3); f.min_convolve(Convex.flat(0..=2)); f.hi == 2 and f.at(1) == 3`
    pub fn min_convolve(mut self, g: Convex) = todo

    /// `f(c) += q(c)` for a quadratic `q` with leading coefficient at least 0, so `f` stays convex;
    /// `q` is called at most four times, as in `Concave.add_quadratic`.
    ///
    /// cost: O(1)
    /// traps: when `q` is not a quadratic polynomial on the points read, or its leading coefficient is negative
    /// example: `var f = Convex.flat(0..=3); f.add_quadratic(x => x * x); f.at(3) == 9`
    pub fn add_quadratic(mut self, q: Int -> Int) = todo

    /// The value `f(c)`.
    ///
    /// cost: O(log n) expected (O(c - lo + log n) in the current implementation)
    /// traps: when `c` is outside `lo..=hi`
    /// example: `var f = Convex.flat(0..=4); f.add_abs(1); f.at(4) == 3`
    pub fn at(mut self, c: Int) -> Int = todo

    /// The smallest value of `f` over its domain.
    ///
    /// cost: O(log n) expected (O(n) in the current implementation)
    /// ties: only the value is kept; which point attains it does not matter
    /// traps: never
    /// example: `var f = Convex.flat(0..=4); f.add_abs(1); f.add_const(5); f.min() == 5`
    pub fn min(mut self) -> Int = todo

    /// Add `max(0, a - x)`: slope -1 left of `a` and flat from `a` on.
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `var f = Convex.flat(0..=4); f.add_left(2); f.at(0) == 2 and f.at(3) == 0`
    pub fn add_left(mut self, a: Int) = todo
    /// Add `max(0, x - a)`: flat up to `a` and slope 1 right of it.
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `var f = Convex.flat(0..=4); f.add_right(2); f.at(4) == 2 and f.at(1) == 0`
    pub fn add_right(mut self, a: Int) = todo
    /// Add `|x - a|`; `add_left(a)` and `add_right(a)` together.
    ///
    /// cost: O(log n) expected
    /// traps: never
    /// example: `var f = Convex.flat(0..=4); f.add_abs(1); f.at(0) == 1 and f.at(4) == 3`
    pub fn add_abs(mut self, a: Int) = todo
    /// Add the constant `c` everywhere.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var f = Convex.point(2); f.add_const(-4); f.at(2) == -4`
    pub fn add_const(mut self, c: Int) = todo
    /// Add `k * x`, applied lazily to every slope.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var f = Convex.flat(0..=4); f.add_linear(-2); f.at(3) == -6`
    pub fn add_linear(mut self, k: Int) = todo
    /// Replace `f(x)` with the smallest `f(y)` over `y <= x` in the domain; the domain is unchanged.
    ///
    /// cost: O(log n) expected
    /// ties: only the value is kept; which point attains it does not matter
    /// traps: never
    /// example: `var f = Convex.flat(0..=4); f.add_abs(2); f.prefix_min(); f.at(0) == 2 and f.at(4) == 0`
    pub fn prefix_min(mut self) = todo
    /// Replace `f(x)` with the smallest `f(y)` over `y >= x` in the domain; the domain is unchanged.
    ///
    /// cost: O(log n) expected
    /// ties: only the value is kept; which point attains it does not matter
    /// traps: never
    /// example: `var f = Convex.flat(0..=4); f.add_abs(2); f.suffix_min(); f.at(0) == 0 and f.at(4) == 2`
    pub fn suffix_min(mut self) = todo
    /// Move the function right by `d`: the new `f(x)` is the old `f(x - d)`, and `lo` and `hi` both
    /// grow by `d`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var f = Convex.point(0); f.add_const(7); f.shift(5); f.lo == 5 and f.at(5) == 7`
    pub fn shift(mut self, d: Int) = todo
    /// Replace `f(x)` with the smallest old `f(y)` over `x - b <= y <= x - a`; the domain becomes
    /// `lo + a ..= hi + b`. The same as `min_convolve(Convex.flat(a..=b))`.
    ///
    /// cost: O(log n) expected
    /// ties: only the value is kept; which point attains it does not matter
    /// traps: when `a > b`
    /// example: `var f = Convex.point(0); f.add_const(1); f.slide_min(-1, 2); f.lo == -1 and f.hi == 2 and f.at(-1) == 1`
    pub fn slide_min(mut self, a: Int, b: Int) = todo
    /// The points where `f` reaches its minimum, as a half-open range; by convexity they are
    /// contiguous.
    ///
    /// cost: O(log n) expected
    /// order: ascending points
    /// ties: every minimizing point is included
    /// traps: never
    /// example: `var f = Convex.flat(0..=4); f.add_abs(2); f.argmin() == 2..3`
    pub fn argmin(mut self) -> Range<Int> = todo

@test
fn concave_matches_a_table_dp() = todo

@test
fn convex_mirrors_concave() = todo
