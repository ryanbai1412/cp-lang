//! Prefix sums, prefix/suffix monoid folds, and offline difference arrays.
//!
//! Call `prefix(a)` on a `[D]T`, `[R, C]T`, or `List<T>` to build static prefix sums in O(n) or
//! O(r * c), then answer any `.sum(...)` or `.prefix(...)` query in O(1).
//! Use `Diff<D, T>()` or `Diff<R, C, T>()` for O(1) offline range/rectangle adds and O(n) /
//! O(r * c) `finish()`.
use std.algebra.{Monoid}
use std.seq.{ends}

/// Fill direction for an append-only `Prefix.filling(...)` table.
pub enum FillOrder = Forward | Reverse

/// Append-only 1-D prefix/suffix sum table returned by `Prefix<D, T>.filling(...)`.
pub struct FillPrefix<D, T>:
    p: [D.cuts]T
    rev: Bool
    next: Int

    /// Write the next cell in fill order (`D.first..=D.last` for `Forward`, reverse for `Reverse`). O(1).
    pub fn set(mut self, i: D, x: T) = todo

    /// The sum of everything left of boundary `b`. O(1).
    pub fn prefix(self, b: Cut<D>) -> T = todo

    /// The sum of `a[i]` for `i` in `r`, or `0` when `r` is empty. O(1).
    pub fn sum(self, r: Range<D>) -> T = todo

/// Prefix sums over a domain `D`, a 2-D grid `(R, C)`, or a `List<T>`, of `Int`s,
/// `F64`s, `Mod`s, or anything else with `+`, `-` and a zero.
/// Construct statically with `prefix(a)` in O(|D|) / O(|R| * |C|) / O(a.len), or incrementally
/// with `Prefix<...>.filling(order: FillOrder.Forward | FillOrder.Reverse)` and O(1) `.set(...)`
/// in fill order; `.sum(...)` and `.prefix(...)` are O(1) over the already-filled prefix or suffix.
pub struct Prefix<D, T>:
    p: [D.cuts]T

    /// Build 1-D prefix sums over `a: [D]T`. O(|D|).
    pub fn new(a: [D]T) -> Prefix<D, T> = todo

    /// Create an append-only 1-D prefix/suffix sum table filled via `.set(i, x)` in `order`. O(|D|).
    pub fn filling(order: FillOrder = FillOrder.Forward) -> FillPrefix<D, T> = todo

    /// The sum of everything left of boundary `b`. O(1).
    pub fn prefix(self, b: Cut<D>) -> T = todo

    /// The sum of `a[i]` for `i` in `r`, or `0` when `r` is empty. O(1).
    pub fn sum(self, r: Range<D>) -> T = todo

    /// O(1). Sum of a wrapping range `r` on a cyclic domain `D`.
    pub fn cyclic_sum(self, r: Range<D>) -> T = todo

    /// O(1). Recover the original element `a[i]`.
    pub fn get(self, i: D) -> T = todo

    /// O(1). Sum of everything right of boundary `b`.
    pub fn suffix(self, b: Cut<D>) -> T = todo

    /// O(1). Total sum of the entire domain.
    pub @get fn all(self) -> T = todo

    /// O(log |D|). Smallest boundary `b` with `prefix(b) >= target`, for non-negative values, or `none`.
    pub fn first_ge(self, target: T) -> ?Cut<D> = todo

    /// O(log |D|). Smallest boundary `b` with `prefix(b) > target`, for non-negative values, or `none`.
    pub fn first_gt(self, target: T) -> ?Cut<D> = todo

/// Internal append-only 2-D prefix/suffix sum table returned by `Prefix<R, C, T>.filling(...)`.
pub struct _FillPrefix2 < R, C, T >:
    p: [R.cuts, C.cuts]T
    rev: Bool
    next: Int

    /// Write the next cell in row-major fill order (top-left to bottom-right for `Forward`, bottom-right to top-left for `Reverse`). O(1).
    pub fn set(mut self, r: R, c: C, x: T) = todo

    /// The sum of everything above row boundary `r` and left of column boundary `c`. O(1).
    pub fn prefix(self, r: Cut<R>, c: Cut<C>) -> T = todo

    /// The sum of `a[r, c]` over `rows` by `cols`, or `0` when either range is empty. O(1).
    pub fn sum(self, rows: Range<R>, cols: Range<C>) -> T = todo

/// Internal 2-D shape of `Prefix<R, C, T>`, constructed by `prefix(a)` or `Prefix<R, C, T>.filling(...)`.
pub struct _Prefix2 < R, C, T >:
    p: [R.cuts, C.cuts]T

    /// Build 2-D prefix sums over `a: [R, C]T`. O(|R| * |C|).
    pub fn new(a: [R, C]T) -> Prefix<R, C, T> = todo

    /// Create an append-only 2-D prefix/suffix sum table filled via `.set(r, c, x)` in row-major `order`. O(|R| * |C|).
    pub fn filling(order: FillOrder = FillOrder.Forward) -> FillPrefix<R, C, T> = todo

    /// The sum of everything above row boundary `r` and left of column boundary `c`. O(1).
    pub fn prefix(self, r: Cut<R>, c: Cut<C>) -> T = todo

    /// The sum of `a[r, c]` over `rows` by `cols`, or `0` when either range is empty. O(1).
    pub fn sum(self, rows: Range<R>, cols: Range<C>) -> T = todo

/// Internal append-only list prefix/suffix sum table returned by `Prefix<T>.filling(n, ...)`.
pub struct _FillPrefixList < T >:
    p: List<T>
    rev: Bool
    next: Int

    /// Write index `i` in fill order (`0..n` for `Forward`, `(0..n).rev` for `Reverse`). O(1).
    pub fn set(mut self, i: Int, x: T) = todo

    /// The sum of the first `k` values. O(1).
    pub fn prefix(self, k: Int) -> T = todo

    /// The sum of `a[i]` for `i` in `r`, or zero when `r` is empty. O(1).
    pub fn sum(self, r: Range<Int>) -> T = todo

/// Internal list shape of `Prefix<T>`, constructed by `prefix(a)` or `Prefix<T>.filling(n, ...)`.
pub struct _PrefixList < T >:
    p: List<T>

    /// Build 1-D prefix sums over `a: List<T>`. O(a.len).
    pub fn new(a: List<T>) -> Prefix<T> = todo

    /// Create an append-only 1-D prefix/suffix sum table of length `n`, filled via `.set(i, x)` in `order`. O(n).
    pub fn filling(n: Int, order: FillOrder = FillOrder.Forward) -> FillPrefix<T> = todo

    /// The sum of the first `k` values. O(1).
    pub fn prefix(self, k: Int) -> T = todo

    /// The sum of `a[i]` for `i` in `r`, or zero when `r` is empty. O(1).
    pub fn sum(
        self,
        r: Range<Int>,
    ) -> T = todo

/// Prefix monoid folds of `a`, indexed by `Cut<D>`: `p[b]` is the fold of `..b`,
/// with `p[D.cuts.nth(0)] == m.id`. O(n).
pub fn prefix_fold<D, T, M>(op: M, a: [D]T) -> [D.cuts]T = todo

/// Suffix monoid folds of `a`, indexed by `Cut<D>`: `s[b]` is the fold of `b..`,
/// with `s[D.cuts.nth(D.K)] == m.id`. With `prefix_fold`, the fold outside `r` is
/// `m.op(pref[r.start], suff[r.end])`. O(n).
pub fn suffix_fold<D, T, M>(op: M, a: [D]T) -> [D.cuts]T = todo

/// Offline difference array: `Diff<D, T>()` for 1-D ranges or `Diff<R, C, T>()` for 2-D grids.
/// O(1) range/rectangle `add`, O(|D|) or O(|R| * |C|) `finish`.
pub struct Diff<D, T>:
    d: [D.cuts]T

    /// Create an all-zero 1-D difference array over `D`. O(|D|).
    pub fn new() -> Diff<D, T> = todo

    /// Add `x` to every position in `r`. O(1).
    pub fn add(mut self, r: Range<D>, x: T) = todo

    /// Materialize the accumulated values across `D`. O(|D|).
    pub fn finish(self) -> [D]T = todo

/// Internal 2-D shape of `Diff<R, C, T>`: O(1) rectangle `add`, O(|R| * |C|) `finish`.
pub struct _Diff2 < R, C, T >:
    d: [R.cuts, C.cuts]T

    /// Create an all-zero 2-D difference array over `(R, C)`. O(|R| * |C|).
    pub fn new() -> Diff<R, C, T> = todo

    /// Add `x` to every cell in `rows` by `cols`. O(1).
    pub fn add(mut self, rows: Range<R>, cols: Range<C>, x: T) = todo

    /// Materialize the accumulated grid across `(R, C)`. O(|R| * |C|).
    pub fn finish(self) -> [R, C]T = todo

/// O(|D|) space. Online sweep-line difference accumulator: schedule future range adds in O(1)
/// and advance a cursor through `D` in `order` in O(1) per position.
pub struct SweepAdd<D, T>:
    d: [D.cuts]T

    /// O(|D|). Create an all-zero online sweep accumulator in `order` (`Forward` or `Reverse`).
    pub fn new(order: FillOrder = FillOrder.Forward) -> SweepAdd<D, T> = todo

    /// O(1). Schedule `+x` across `r` (must start at or ahead of the current sweep cursor).
    pub fn add(mut self, r: Range<D>, x: T) = todo

    /// O(1). Advance the cursor to `i` (next position in fill order) and return the accumulated value.
    pub fn advance(mut self, i: D) -> T = todo

    /// O(1). Accumulated value at the most recently advanced position.
    pub @get fn current(self) -> T = todo

/// O(|D|) space. Second-order offline difference array for arithmetic-progression range adds:
/// O(1) `add_arith`, O(|D|) `finish`.
pub struct ArithDiff<D, T>:
    d: [D.cuts]T

    /// O(|D|). Create an all-zero second-order difference array over `D`.
    pub fn new() -> ArithDiff<D, T> = todo

    /// O(1). Add `start + step * (i.ord - r.start.ord)` to every position `i` in `r`.
    pub fn add_arith(mut self, r: Range<D>, start: T, step: T) = todo

    /// O(1). Add a constant `x` to every position in `r`.
    pub fn add(mut self, r: Range<D>, x: T) = todo

    /// O(|D|). Materialize the accumulated values across `D`.
    pub fn finish(self) -> [D]T = todo

/// O(|D|). Exclusive monoid fold: `out[i]` is the fold of every element in `a` except `i`,
/// without requiring inverses.
pub fn prefix_excl<D, T, M>(op: M, a: [D]T) -> [D]T = todo

@test
fn one_d_sums_match_brute() = todo

@test
fn two_d_sums_match_brute() = todo

@test
fn a_plain_range_and_list_take_plain_ints() = todo

@test
fn prefix_and_suffix_folds_cover_outside_ranges() = todo

@test
fn diff_1d_and_2d_match_brute() = todo

@test
fn filling_1d_forward_and_reverse_match_brute() = todo

@test
fn filling_2d_forward_and_reverse_match_brute() = todo

@test
fn filling_list_forward_and_reverse_match_brute() = todo
