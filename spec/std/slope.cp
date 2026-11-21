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

    /// O(1). `f(d) = 0` on the single point `d`.
    pub fn point(d: Int) -> Concave = todo

    /// O(len log len) expected. `f(c) = 0` on `r`.
    pub fn flat(r: Range<Int>) -> Concave = todo

    /// O(n). Make room for `n` slopes in total without regrowing.
    pub fn reserve(mut self, n: Int) = todo

    /// O(1). The first point of the domain.
    pub @get fn lo(self) -> Int = todo

    /// O(1). The last point of the domain.
    pub @get fn hi(self) -> Int = todo

    /// O(|g| log n) expected; convolve the smaller into the larger.
    /// `f = max over a + b = c of f(a) + g(b)`: the domains add and the slope
    /// multisets merge.
    pub fn max_convolve(mut self, g: Concave) = todo

    /// O(1). `f(c) += q(c)` for a quadratic `q` with leading coefficient at
    /// most 0, read at three points (a fourth checks it is quadratic).
    pub fn add_quadratic(mut self, q: Int -> Int) = todo

    /// O(log n) expected with subtree sums; O(c - lo + log n) today.
    /// `f(c)` for `c` in `lo..=hi`.
    pub fn at(mut self, c: Int) -> Int = todo

    /// O(log n) expected with subtree sums; O(n) today. The largest value of `f`.
    pub fn max(mut self) -> Int = todo

    /// O(log n) expected. Adds `min(0, x - a)`: a hinge that is flat right of `a`.
    pub fn add_left(mut self, a: Int) = todo
    /// O(log n) expected. Adds `min(0, a - x)`: a hinge that is flat left of `a`.
    pub fn add_right(mut self, a: Int) = todo
    /// O(log n) expected. Adds `-|x - a|`.
    pub fn add_abs(mut self, a: Int) = todo
    /// O(1). Adds the constant `c`.
    pub fn add_const(mut self, c: Int) = todo
    /// O(1) lazy. Adds `k * x`.
    pub fn add_linear(mut self, k: Int) = todo
    /// O(log n) expected. `f(x) = max over y <= x of f(y)`.
    pub fn prefix_max(mut self) = todo
    /// O(log n) expected. `f(x) = max over y >= x of f(y)`.
    pub fn suffix_max(mut self) = todo
    /// O(1). `f(x) = f(x - d)`: the domain moves by `d`.
    pub fn shift(mut self, d: Int) = todo
    /// O(log n) expected. `f(x) = max over x - b <= y <= x - a of f(y)`.
    pub fn slide_max(mut self, a: Int, b: Int) = todo
    /// O(log n) expected. The points where `f` is largest.
    pub fn argmax(mut self) -> Range<Int> = todo

/// O(log n) expected per slope operation, for n slopes.
/// A convex function on the integers `lo..=hi`, the negation of a `Concave`:
/// the same operations with `min` for `max`.
pub struct Convex(neg: Concave):
    /// O(1). `f(d) = 0` on the single point `d`.
    pub fn point(d: Int) -> Convex = todo

    /// O(len log len) expected. `f(c) = 0` on `r`.
    pub fn flat(r: Range<Int>) -> Convex = todo

    /// O(1). The first point of the domain.
    pub @get fn lo(self) -> Int = todo

    /// O(1). The last point of the domain.
    pub @get fn hi(self) -> Int = todo

    /// O(|g| log n) expected. `f = min over a + b = c of f(a) + g(b)`.
    pub fn min_convolve(mut self, g: Convex) = todo

    /// O(1). `f(c) += q(c)` for a quadratic `q` with leading coefficient at least 0.
    pub fn add_quadratic(mut self, q: Int -> Int) = todo

    /// O(log n) expected with subtree sums; O(c - lo + log n) today.
    pub fn at(mut self, c: Int) -> Int = todo

    /// O(log n) expected with subtree sums; O(n) today. The smallest value of `f`.
    pub fn min(mut self) -> Int = todo

    /// O(log n) expected. Adds `max(0, a - x)`.
    pub fn add_left(mut self, a: Int) = todo
    /// O(log n) expected. Adds `max(0, x - a)`.
    pub fn add_right(mut self, a: Int) = todo
    /// O(log n) expected. Adds `|x - a|`.
    pub fn add_abs(mut self, a: Int) = todo
    /// O(1). Adds the constant `c`.
    pub fn add_const(mut self, c: Int) = todo
    /// O(1) lazy. Adds `k * x`.
    pub fn add_linear(mut self, k: Int) = todo
    /// O(log n) expected. `f(x) = min over y <= x of f(y)`.
    pub fn prefix_min(mut self) = todo
    /// O(log n) expected. `f(x) = min over y >= x of f(y)`.
    pub fn suffix_min(mut self) = todo
    /// O(1). `f(x) = f(x - d)`: the domain moves by `d`.
    pub fn shift(mut self, d: Int) = todo
    /// O(log n) expected. `f(x) = min over x - b <= y <= x - a of f(y)`.
    pub fn slide_min(mut self, a: Int, b: Int) = todo
    /// O(log n) expected. The points where `f` is smallest.
    pub fn argmin(mut self) -> Range<Int> = todo

@test
fn concave_matches_a_table_dp() = todo

@test
fn convex_mirrors_concave() = todo
