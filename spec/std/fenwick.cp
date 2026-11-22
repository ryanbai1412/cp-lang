//! Fenwick tree (binary indexed tree) over a domain: point add, range sum.
use std.algebra.{Group}
use std.seq.{ends}

/// Point add and range sum over anything with `+` and `-` (`Int`, `Mod<P>`).
/// `Fenwick<D, Int>()` starts at zero and `Fenwick<D, Int>(a)` from an array, in O(n).
/// `add`, `prefix` and `sum` are O(log n); memory is n + 1 values.
/// A range is one prefix minus another, so there is no "Fenwick on max": that is a `SegTree`.
pub struct Fenwick<D, T>:
    t: List<T>

    /// A Fenwick tree over `D`: all zeros, or holding the values of `a` when given. The values
    /// must form a group under `+` and `-` (`Int`, `Mod<P>`, `F64`), since a range sum is one
    /// prefix minus another.
    ///
    /// cost: O(|D|) time; memory |D| + 1 values
    /// empty: an empty `D` gives a tree whose every sum is zero
    /// order: positions keep the domain `D` (of `a`, when given)
    /// traps: never
    /// example: `Fenwick([1, 2, 3]).all == 6`
    /// example: `Fenwick<0..3, Int>().all == 0`
    pub fn new(a: ?[D]T = None) -> Fenwick<D, T> = todo

    /// Add `x` to the value at position `i`: `a[i] += x`.
    ///
    /// cost: O(log |D|)
    /// traps: when `i` is outside `D`
    /// example: `var f = Fenwick([1, 2, 3]); f.add(0, 5); f.fold(0..2) == 8`
    pub fn add(mut self, i: D, x: T) = todo

    /// The sum of every position left of the boundary `b`. For a position `v`, `prefix(v.r)`
    /// includes `v` and `prefix(v.l)` does not; the start cut gives zero.
    ///
    /// cost: O(log |D|)
    /// traps: never (`b` is a cut of `D` by type)
    /// example: `Fenwick([1, 2, 3]).prefix((0..2).r) == 3`
    /// example: `Fenwick([1, 2, 3]).prefix((0..0).r) == 0`
    pub fn prefix(self, b: Cut<D>) -> T = todo

    /// The sum over the positions of `r`: `prefix(r.r) - prefix(r.l)`. Accepts every range form
    /// (`l..r`, `l..=r`, `..i`, `i..`).
    ///
    /// cost: O(log |D|)
    /// empty: zero
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `Fenwick([1, 2, 3]).fold(1..3) == 5`
    /// example: `Fenwick([1, 2, 3]).fold(2..2) == 0`
    pub fn fold(self, r: Range<D>) -> T = todo

    /// The smallest boundary `b` with `prefix(b) >= target`, by binary lifting over the tree;
    /// `None` when even the whole sum is below `target`. A `target <= 0` gives the start cut.
    /// Correct only when every value is non-negative (prefix sums non-decreasing).
    ///
    /// cost: O(log |D|)
    /// ties: the leftmost qualifying cut
    /// traps: never (negative values give an unspecified cut, not a trap)
    /// example: `Fenwick([1, 2, 3]).first_ge(3) == Some((0..2).r)`
    /// example: `Fenwick([1, 2, 3]).first_ge(7) == None`
    /// example: `Fenwick([1, 2, 3]).first_ge(0) == Some((0..0).r)`
    pub fn first_ge(self, target: Int) -> ?Cut<D> = todo

    /// The `k`-th item counting from 1, when position `d` holds `a[d]` items: the first `d` with
    /// `prefix(d.r) >= k`, or `None` when fewer than `k` items exist in total. Positions with value
    /// zero are never returned. For non-negative values.
    ///
    /// cost: O(log |D|)
    /// ties: the leftmost qualifying position
    /// traps: when `k < 1` (negative values give an unspecified position, not a trap)
    /// example: `Fenwick([1, 0, 2]).kth(2) == Some(2)`
    /// example: `Fenwick([1, 0, 2]).kth(1) == Some(0)`
    /// example: `Fenwick([1, 0, 2]).kth(4) == None`
    pub fn kth(self, k: Int) -> ?D = todo

    /// The current value at position `i`: `sum(i..=i)`.
    ///
    /// cost: O(log |D|)
    /// traps: when `i` is outside `D`
    /// example: `Fenwick([1, 2, 3]).get(1) == 2`
    pub fn get(self, i: D) -> T = todo

    /// Assign `a[i] = x`, by adding the difference `x - get(i)`; also the write-through place for
    /// `f[i] = x`.
    ///
    /// cost: O(log |D|)
    /// traps: when `i` is outside `D`
    /// example: `var f = Fenwick([1, 2, 3]); f.set(1, 5); f.all == 9`
    pub fn set(mut self, i: D, x: T) = todo

    /// The sum of every position of `D`, kept current by `add`, `set` and `clear`.
    ///
    /// cost: O(1)
    /// empty: zero on an empty domain
    /// traps: never
    /// example: `Fenwick([1, 2, 3]).all == 6`
    @get pub fn all(self) -> T = todo

    /// The sum of every position right of the boundary `b`: `all - prefix(b)`. The end cut gives zero.
    ///
    /// cost: O(log |D|)
    /// traps: never
    /// example: `Fenwick([1, 2, 3]).suffix((0..1).r) == 5`
    /// example: `Fenwick([1, 2, 3]).suffix((0..3).r) == 0`
    pub fn suffix(self, b: Cut<D>) -> T = todo

    /// The smallest boundary `b` with `prefix(b) > target`, or `None` when the whole sum is at most
    /// `target`. For non-negative values.
    ///
    /// cost: O(log |D|)
    /// ties: the leftmost qualifying cut
    /// traps: never (negative values give an unspecified cut, not a trap)
    /// example: `Fenwick([1, 2, 3]).first_gt(3) == Some((0..3).r)`
    /// example: `Fenwick([1, 2, 3]).first_gt(6) == None`
    pub fn first_gt(self, target: Int) -> ?Cut<D> = todo

    /// Reset every value to zero, keeping the domain and the allocation.
    ///
    /// cost: O(|D|)
    /// traps: never
    /// example: `var f = Fenwick([1, 2, 3]); f.clear(); f.all == 0`
    pub fn clear(mut self) = todo

/// Point add and rectangle sum on a grid. `add` and `sum` are O(log R * log C);
/// memory is (R + 1) * (C + 1) values.
pub struct Fenwick2D<R, C, T>:
    t: List<T>

    /// An all-zero 2-D Fenwick tree over the grid `R` by `C`.
    ///
    /// cost: O(|R| |C|) time; memory (|R| + 1)(|C| + 1) values
    /// traps: never
    /// example: `Fenwick2D<0..2, 0..2, Int>().fold(0..2, 0..2) == 0`
    pub fn new() -> Fenwick2D<R, C, T> = todo

    /// Add `x` to the cell `(r, c)`: `a[r, c] += x`.
    ///
    /// cost: O(log |R| log |C|)
    /// traps: when `r` is outside `R` or `c` is outside `C`
    /// example: `var f = Fenwick2D<0..2, 0..2, Int>(); f.add(1, 0, 5); f.fold(1..2, 0..1) == 5`
    pub fn add(mut self, r: R, c: C, x: T) = todo

    fn corner(self, r: Int, c: Int) -> T = todo

    /// The sum over the rectangle `rows` by `cols`, by inclusion-exclusion over four prefixes.
    ///
    /// cost: O(log |R| log |C|)
    /// empty: zero when either range is empty
    /// traps: when a range of a non-empty rectangle reaches outside its domain
    /// example: `var f = Fenwick2D<0..2, 0..2, Int>(); f.add(0, 0, 1); f.add(1, 1, 2); f.fold(0..2, 1..2) == 2`
    /// example: `var f = Fenwick2D<0..2, 0..2, Int>(); f.add(0, 0, 1); f.fold(0..2, 1..1) == 0`
    pub fn fold(self, rows: Range<R>, cols: Range<C>) -> T = todo

    /// The sum of every cell above the row cut `r` and left of the column cut `c`.
    ///
    /// cost: O(log |R| log |C|)
    /// traps: never
    /// example: `var f = Fenwick2D<0..2, 0..2, Int>(); f.add(0, 0, 1); f.add(1, 1, 2); f.prefix((0..1).r, (0..2).r) == 1`
    pub fn prefix(self, r: Cut<R>, c: Cut<C>) -> T = todo

    /// The current value of the cell `(r, c)`.
    ///
    /// cost: O(log |R| log |C|)
    /// traps: when `r` is outside `R` or `c` is outside `C`
    /// example: `var f = Fenwick2D<0..2, 0..2, Int>(); f.add(1, 1, 2); f.get(1, 1) == 2`
    pub fn get(self, r: R, c: C) -> T = todo

    /// Reset every cell to zero, keeping the domains and the allocation.
    ///
    /// cost: O(|R| |C|)
    /// traps: never
    /// example: `var f = Fenwick2D<0..2, 0..2, Int>(); f.add(1, 1, 2); f.clear(); f.get(1, 1) == 0`
    pub fn clear(mut self) = todo

/// Range add and point get via a difference Fenwick tree.
/// `DualFenwick<D, Int>()` starts at zero and `DualFenwick<D, Int>(a)` from an array, in O(n).
/// `add` and `get` are O(log n); memory is n + 1 values.
pub struct DualFenwick<D, T>:
    t: List<T>

    /// A range-add, point-get Fenwick tree over `D`: all zeros, or holding the values of `a` when
    /// given. It stores differences, so `get(i)` is a prefix sum of them.
    ///
    /// cost: O(|D|) time; memory |D| + 1 values
    /// empty: an empty `D` gives a tree with no positions to read
    /// order: positions keep the domain `D` (of `a`, when given)
    /// traps: never
    /// example: `DualFenwick([1, 2, 3]).get(2) == 3`
    pub fn new(a: ?[D]T = None) -> DualFenwick<D, T> = todo

    fn add_at(mut self, p0: Int, x: T) = todo

    /// Add `x` to every position of `r`.
    ///
    /// cost: O(log |D|)
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var d = DualFenwick([1, 2, 3]); d.add(0..2, 10); d.get(1) == 12`
    /// example: `var d = DualFenwick([1, 2, 3]); d.add(0..2, 10); d.get(2) == 3`
    pub fn add(mut self, r: Range<D>, x: T) = todo

    /// The current value at position `i`: its initial value plus every `x` added over a range
    /// containing `i`.
    ///
    /// cost: O(log |D|)
    /// traps: when `i` is outside `D`
    /// example: `var d = DualFenwick([1, 2, 3]); d.add(1..3, 5); d.get(1) == 7`
    pub fn get(self, i: D) -> T = todo

    /// Reset every value to zero, keeping the domain and the allocation.
    ///
    /// cost: O(|D|)
    /// traps: never
    /// example: `var d = DualFenwick([1, 2, 3]); d.clear(); d.get(0) == 0`
    pub fn clear(mut self) = todo

/// O(n log n) space. Offline coordinate-compressed 2D Fenwick tree (KACTL `FT2`):
/// register all update coordinates with `add_point`, call `build()`, then run `add` and `sum`
/// in O(log^2 n).
pub struct SparseFenwick2D<T>:
    n: Int

    /// An empty offline 2-D Fenwick builder. Use it in two phases: register every point that will
    /// ever be updated with `add_point`, call `build` once, then use `add` and `sum`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var f = SparseFenwick2D<Int>(); f.build(); f.fold(0..10, 0..10) == 0`
    pub fn new() -> SparseFenwick2D<T> = todo

    /// Register the point `(x, y)` as a future `add` target. Registering a point twice is harmless.
    ///
    /// cost: O(1) amortized
    /// traps: after `build`
    /// example: `var f = SparseFenwick2D<Int>(); f.add_point(5, 7); f.add_point(5, 7); f.build(); f.add(5, 7, 3); f.fold(0..10, 0..10) == 3`
    pub fn add_point(mut self, x: Int, y: Int) = todo

    /// Compress the registered `x` coordinates, and per `x` bucket the `y` coordinates it can
    /// receive, and allocate the bucket trees at zero. Ends the registration phase.
    ///
    /// cost: O(n log n) for `n` registered points; memory O(n log n)
    /// traps: when called a second time
    /// example: `var f = SparseFenwick2D<Int>(); f.add_point(1, 2); f.build(); f.fold(1..2, 2..3) == 0`
    pub fn build(mut self) = todo

    /// Add `v` at the registered point `(x, y)`.
    ///
    /// cost: O(log^2 n)
    /// traps: before `build`, or when `(x, y)` was never registered
    /// example: `var f = SparseFenwick2D<Int>(); f.add_point(1, 2); f.add_point(3, 4); f.build(); f.add(1, 2, 5); f.add(3, 4, 7); f.fold(0..2, 0..10) == 5`
    pub fn add(mut self, x: Int, y: Int, v: T) = todo

    /// The total added at registered points with `x` in `xs` and `y` in `ys`. The ranges are over
    /// raw coordinates; points outside them, or never registered, contribute nothing.
    ///
    /// cost: O(log^2 n)
    /// empty: zero when either range is empty
    /// traps: before `build`
    /// example: `var f = SparseFenwick2D<Int>(); f.add_point(1, 2); f.add_point(3, 4); f.build(); f.add(1, 2, 5); f.add(3, 4, 7); f.fold(0..10, 3..5) == 7`
    /// example: `var f = SparseFenwick2D<Int>(); f.add_point(1, 2); f.build(); f.add(1, 2, 5); f.fold(2..2, 0..10) == 0`
    pub fn fold(self, xs: Range<Int>, ys: Range<Int>) -> T = todo

@test
fn sums_match_a_plain_array() = todo

@test
fn empty_and_half_open_ranges() = todo

@test
fn first_ge_finds_the_kth() = todo

@test
fn two_dimensions_match_a_plain_grid() = todo

@test
fn xor_matches_a_plain_array() = todo

@test
fn dual_and_range_fenwick_match_brute() = todo
