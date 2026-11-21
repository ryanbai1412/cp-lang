//! Fenwick tree (binary indexed tree) over a domain: point add, range sum.
use std.algebra.{Group}
use std.seq.{ends}

/// Point add and range sum over anything with `+` and `-` (`Int`, `Mod<P>`).
/// `Fenwick<D, Int>()` starts at zero and `Fenwick<D, Int>(a)` from an array, in O(n).
/// `add`, `prefix` and `sum` are O(log n); memory is n + 1 values.
/// A range is one prefix minus another, so there is no "Fenwick on max": that is a `SegTree`.
pub struct Fenwick<D, T>:
    t: List<T>

    /// Create a Fenwick tree over `D`, optionally initialized from `a`. O(|D|).
    pub fn new(a: ?[D]T = none) -> Fenwick<D, T> = todo

    /// `a[i] += x`.
    pub fn add(mut self, i: D, x: T) = todo

    /// The sum of everything left of the boundary `b`: `prefix(v.r)` includes `v`, `prefix(v.l)` does not.
    pub fn prefix(self, b: Cut<D>) -> T = todo

    /// The sum of a range: `f.sum(l..=r)`, `f.sum(..i)`, `f.sum(i..)`. An empty range is zero.
    pub fn sum(self, r: Range<D>) -> T = todo

    /// The first boundary where the prefix sum reaches `target`, for non-negative values:
    /// the smallest `b` with `prefix(b) >= target`, or `none`. O(log n).
    pub fn first_ge(self, target: Int) -> ?Cut<D> = todo

    /// The `k`-th position, counting from 1, when each position stands for as many items as
    /// its value: the first `d` with `prefix` through `d` at least `k`, or `none` when there
    /// are fewer than `k` items. For non-negative values. O(log n).
    pub fn kth(self, k: Int) -> ?D = todo

    /// O(log |D|). Single-position value `a[i]`.
    pub fn get(self, i: D) -> T = todo

    /// O(log |D|). Assign `a[i] = x` (`IndexSet` write-through place).
    pub fn set(mut self, i: D, x: T) = todo

    /// O(log |D|). Total sum of the entire domain.
    pub @get fn all(self) -> T = todo

    /// O(log |D|). Sum of everything right of boundary `b`.
    pub fn suffix(self, b: Cut<D>) -> T = todo

    /// O(log |D|). Smallest boundary `b` with `prefix(b) > target`, for non-negative values, or `none`.
    pub fn first_gt(self, target: Int) -> ?Cut<D> = todo

    /// O(|D|). Reset all entries to zero without reallocating.
    pub fn clear(mut self) = todo

/// Point xor and range xor over anything with `^` (`Int`, `U64`): the Fenwick tree whose
/// `+` is `^`, for Zobrist hashes and nim sums. `XorFenwick<D, U64>()` starts at zero and
/// `XorFenwick<D, U64>(a)` from an array, in O(n). `flip`, `prefix` and `xor` are O(log n).
pub struct XorFenwick<D, T>:
    t: List<T>

    /// Create a bitwise-XOR Fenwick tree over `D`, optionally initialized from `a`. O(|D|).
    pub fn new(a: ?[D]T = none) -> XorFenwick<D, T> = todo

    /// `a[i] ^= x`.
    pub fn flip(mut self, i: D, x: T) = todo

    /// The xor of everything left of the boundary `b`: `prefix(v.r)` includes `v`, `prefix(v.l)` does not.
    pub fn prefix(self, b: Cut<D>) -> T = todo

    /// The xor of a range: `f.xor(l..=r)`, `f.xor(..i)`, `f.xor(i..)`. An empty range is zero.
    pub fn xor(self, r: Range<D>) -> T = todo

/// Point add and rectangle sum on a grid. `add` and `sum` are O(log R * log C);
/// memory is (R + 1) * (C + 1) values.
pub struct Fenwick2D<R, C, T>:
    t: List<T>

    /// Create an all-zero 2D Fenwick tree over `(R, C)`. O(|R| * |C|).
    pub fn new() -> Fenwick2D<R, C, T> = todo

    /// `a[r, c] += x`.
    pub fn add(mut self, r: R, c: C, x: T) = todo

    fn corner(self, r: Int, c: Int) -> T = todo

    /// The sum of the rectangle `rows` by `cols`.
    pub fn sum(self, rows: Range<R>, cols: Range<C>) -> T = todo

    /// O(log |R| * log |C|). Prefix rectangle sum above `r` and left of `c`.
    pub fn prefix(self, r: Cut<R>, c: Cut<C>) -> T = todo

    /// O(log |R| * log |C|). Single-cell value `a[r, c]`.
    pub fn get(self, r: R, c: C) -> T = todo

    /// O(|R| * |C|). Reset all entries to zero without reallocating.
    pub fn clear(mut self) = todo

/// Range add and point get via a difference Fenwick tree.
/// `DualFenwick<D, Int>()` starts at zero and `DualFenwick<D, Int>(a)` from an array, in O(n).
/// `add` and `get` are O(log n); memory is n + 1 values.
pub struct DualFenwick<D, T>:
    t: List<T>

    /// Create a range-add / point-query Fenwick tree over `D`. O(|D|).
    pub fn new(a: ?[D]T = none) -> DualFenwick<D, T> = todo

    fn add_at(mut self, p0: Int, x: T) = todo

    /// Add `x` to every position in `r`. O(log n).
    pub fn add(mut self, r: Range<D>, x: T) = todo

    /// Current value at position `i`. O(log n).
    pub fn get(self, i: D) -> T = todo

    /// O(|D|). Reset all entries to zero without reallocating.
    pub fn clear(mut self) = todo

/// Range add and range sum over `Int` using two Fenwick arrays.
/// `RangeFenwick<D>()` starts at zero and `RangeFenwick<D>(a)` from an array, in O(n).
/// `add`, `prefix` and `sum` are O(log n); memory is 2 * (n + 1) ints.
pub struct RangeFenwick<D>:
    b1: List<Int>
    b2: List<Int>

    /// Create a range-add / range-sum Fenwick tree over `D`. O(|D|).
    pub fn new(a: ?[D]Int = none) -> RangeFenwick<D> = todo

    fn add_at(mut self, p0: Int, v1: Int, v2: Int) = todo

    /// Add `x` to every position in `r`. O(log n).
    pub fn add(mut self, r: Range<D>, x: Int) = todo

    /// The sum of everything left of boundary `b`. O(log n).
    pub fn prefix(self, b: Cut<D>) -> Int = todo

    /// The sum of a range `r`. O(log n).
    pub fn sum(self, r: Range<D>) -> Int = todo

    /// O(log |D|). Single-position value `a[i]`.
    pub fn get(self, i: D) -> Int = todo

    /// O(log |D|). Total sum of the entire domain.
    pub @get fn all(self) -> Int = todo

    /// O(|D|). Reset all entries to zero without reallocating.
    pub fn clear(mut self) = todo

/// O(|D|) space. Fenwick tree parameterized by any abelian `Group<T>` (point update and range fold
/// in O(log |D|); generalizes `Fenwick` and `XorFenwick`).
pub struct GroupFenwick<D, T>(g: Group<T>):
    t: List<T>

    /// O(|D|). Create a group Fenwick tree over `D`, optionally initialized from `a`.
    pub fn new<G>(op: G, a: ?[D]T = none) -> GroupFenwick<D, T> = todo

    /// O(log |D|). Combine `x` into position `i`: `a[i] = g.op(a[i], x)`.
    pub fn update(mut self, i: D, x: T) = todo

    /// O(log |D|). Fold of everything left of boundary `b`.
    pub fn prefix(self, b: Cut<D>) -> T = todo

    /// O(log |D|). Fold of range `r`, using `g.inv` on the left prefix.
    pub fn fold(self, r: Range<D>) -> T = todo

    /// O(|D|). Reset all entries to `g.id`.
    pub fn clear(mut self) = todo

/// O(|D|) space. Prefix-only monotone Fenwick tree (e.g. prefix `max`/`min` for 1D DP / LIS):
/// O(log |D|) point combine and prefix fold with half the memory of `SegTree`.
pub struct PrefixFenwick<D, T>(op: (T, T) -> T, id: T):
    t: List<T>

    /// O(|D|). Create a prefix-only Fenwick tree initialized to `id` (or `a`).
    pub fn new(op: (T, T) -> T, id: T, a: ?[D]T = none) -> PrefixFenwick<D, T> = todo

    /// O(log |D|). Monotone update `a[i] = op(a[i], x)`.
    pub fn update(mut self, i: D, x: T) = todo

    /// O(log |D|). Fold of everything left of boundary `b`.
    pub fn prefix(self, b: Cut<D>) -> T = todo

    /// O(|D|). Reset all entries to `id`.
    pub fn clear(mut self) = todo

/// O(n log n) space. Offline coordinate-compressed 2D Fenwick tree (KACTL `FT2`):
/// register all update coordinates with `add_point`, call `build()`, then run `add` and `sum`
/// in O(log^2 n).
pub struct SparseFenwick2D<T>:
    n: Int

    /// O(1). Create an empty offline 2D Fenwick builder.
    pub fn new() -> SparseFenwick2D<T> = todo

    /// O(1) amortized. Register a point `(x, y)` that will later be updated.
    pub fn add_point(mut self, x: Int, y: Int) = todo

    /// O(n log n). Compress all registered coordinates and allocate per-bucket Fenwick arrays.
    pub fn build(mut self) = todo

    /// O(log^2 n). Add `v` at previously registered coordinate `(x, y)`.
    pub fn add(mut self, x: Int, y: Int, v: T) = todo

    /// O(log^2 n). Sum of all registered points with `x` in `xs` and `y` in `ys`.
    pub fn sum(self, xs: Range<Int>, ys: Range<Int>) -> T = todo

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
