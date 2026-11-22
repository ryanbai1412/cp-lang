//! Prefix sums, prefix/suffix monoid folds, and offline difference arrays.
//!
//! Call `prefix(a)` on a `[D]T`, `[R, C]T`, or `List<T>` to build static prefix sums in O(n) or
//! O(r * c), then answer any `.sum(...)` or `.prefix(...)` query in O(1).
//! Use `Diff<D, T>()` or `Diff<R, C, T>()` for O(1) offline range/rectangle adds and O(n) /
//! O(r * c) `finish()`.
use std.algebra.{Monoid}
use std.seq.{ends, RingRange}

/// The `|D| + 1` exclusive prefix folds of `self` under `f`, indexed by `D`'s cuts: `pre[c]`
/// folds everything left of cut `c` (`from`, or `f`'s identity, at the first cut), and
/// `pre[v]` for a position folds through `v`. Suffix folds are `self.rev.scan(f)`, on the
/// same cuts.
///
///     let pre = a.scan(+)
///     pre[r.r] - pre[r.l]          // sum over r; also pre.fold(r)
///     pre[v.l] + a[v] + suf[v.r]   // with suf = a.rev.scan(+)
///
/// cost: O(n) calls of `f`
/// empty: an empty `D` gives one cut holding `from` (or the identity)
/// traps: never
/// example: `let a: [0..3]Int = [1, 2, 3]; let pre = a.scan(+); pre[(0..2).r] == 3 and pre[(0..0).r] == 0 and pre[1] == 3`
/// example: `let a: [0..3]Int = [1, 2, 3]; a.scan(+, from: 10)[(0..3).r] == 16`
pub fn scan<D, T>(self: [D]T, f: (T, T) -> T, from: ?T = None) -> Prefix<D, T> = todo

/// Fill direction for an append-only `Prefix.filling(...)` table.
pub enum FillOrder = Forward | Reverse

/// Append-only 1-D prefix/suffix sum table returned by `Prefix<D, T>.filling(...)`.
pub struct FillPrefix<D, T>:
    p: [D.cuts]T
    rev: Bool
    next: Int

    /// Writes the next cell in fill order: `D.first..=D.last` for `Forward`, the reverse for
    /// `Reverse`.
    ///
    /// cost: O(1)
    /// traps: when `i` is not the next cell in fill order
    /// example: `var f = Prefix<0..3, Int>.filling(); f.set(0, 1); f.set(1, 2); f.sum(0..2) == 3`
    pub fn set(mut self, i: D, x: T) = todo

    /// The sum of everything left of cut `b`; every cell left of `b` must be filled already.
    ///
    /// cost: O(1)
    /// traps: when a cell left of `b` is not filled yet
    /// example: `var f = Prefix<0..3, Int>.filling(); f.set(0, 1); f.set(1, 2); f.prefix((0..2).r) == 3`
    pub fn prefix(self, b: Cut<D>) -> T = todo

    /// The sum of `a[i]` for `i` in `r`, or `0` when `r` is empty; every cell of `r` must be
    /// filled already.
    ///
    /// cost: O(1)
    /// empty: an empty `r` gives `0`
    /// traps: when a cell of `r` is not filled yet
    /// example: `var f = Prefix<0..3, Int>.filling(order: FillOrder.Reverse); f.set(2, 4); f.set(1, 1); f.sum(1..3) == 5`
    pub fn sum(self, r: Range<D>) -> T = todo

/// Prefix sums over a domain `D`, a 2-D grid `(R, C)`, or a `List<T>`, of `Int`s,
/// `F64`s, `Mod`s, or anything else with `+`, `-` and a zero.
/// Construct statically with `prefix(a)` in O(|D|) / O(|R| * |C|) / O(a.len), or incrementally
/// with `Prefix<...>.filling(order: FillOrder.Forward | FillOrder.Reverse)` and O(1) `.set(...)`
/// in fill order; `.sum(...)` and `.prefix(...)` are O(1) over the already-filled prefix or suffix.
pub struct Prefix<D, T>:
    p: [D.cuts]T

    /// Prefix sums over `a`, as `prefix(a)`.
    ///
    /// cost: O(|D|)
    /// empty: an empty `D` gives a table whose every sum is `0`
    /// traps: never
    /// example: `let a: [0..3]Int = [1, 2, 3]; Prefix.new(a).sum(1..3) == 5`
    pub fn new(a: [D]T) -> Prefix<D, T> = todo

    /// An all-zero append-only table filled by `.set(i, x)` in `order`; queries read the part
    /// already filled.
    ///
    /// cost: O(|D|)
    /// traps: never
    /// example: `var f = Prefix<0..3, Int>.filling(); f.set(0, 7); f.sum(0..1) == 7`
    pub fn filling(order: FillOrder = FillOrder.Forward) -> FillPrefix<D, T> = todo

    /// The sum of everything left of cut `b`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `let a: [0..3]Int = [1, 2, 3]; prefix(a).prefix((0..2).r) == 3 and prefix(a).prefix((0..0).r) == 0`
    pub fn prefix(self, b: Cut<D>) -> T = todo

    /// The sum of `a[i]` for `i` in `r`: `prefix(r.r) - prefix(r.l)`.
    ///
    /// cost: O(1)
    /// empty: an empty `r` gives `0`
    /// traps: never
    /// example: `let a: [0..3]Int = [1, 2, 3]; prefix(a).sum(1..3) == 5 and prefix(a).sum(2..2) == 0`
    pub fn sum(self, r: Range<D>) -> T = todo

    /// The sum over a wrapping run of positions, such as `a.ring.span(i, k)`: the part up to
    /// `D.last` plus the part from `D.first`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `let a: [0..3]Int = [1, 2, 3]; prefix(a).cyclic_sum(a.ring.span(2, 2)) == 4`
    pub fn cyclic_sum(self, r: RingRange<D>) -> T = todo

    /// The original element `a[i]`, recovered as a difference of two prefixes.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `let a: [0..3]Int = [1, 2, 3]; prefix(a).get(1) == 2`
    pub fn get(self, i: D) -> T = todo

    /// The sum of everything right of cut `b`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `let a: [0..3]Int = [1, 2, 3]; prefix(a).suffix((0..1).r) == 5 and prefix(a).suffix((0..3).r) == 0`
    pub fn suffix(self, b: Cut<D>) -> T = todo

    /// The sum of the whole domain; `0` for an empty one.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `let a: [0..3]Int = [1, 2, 3]; prefix(a).all == 6`
    pub @get fn all(self) -> T = todo

    /// The smallest cut `b` with `prefix(b) >= target`, or `None`. Assumes non-negative values,
    /// so the prefixes never decrease; a binary search.
    ///
    /// cost: O(log |D|)
    /// ties: the smallest such cut; a `target <= 0` gives the first cut
    /// traps: never
    /// example: `let a: [0..3]Int = [1, 2, 3]; prefix(a).first_ge(3) == Some((0..2).r)`
    /// example: `let a: [0..3]Int = [1, 2, 3]; prefix(a).first_ge(7) == None`
    pub fn first_ge(self, target: T) -> ?Cut<D> = todo

    /// The smallest cut `b` with `prefix(b) > target`, or `None`. Assumes non-negative values.
    ///
    /// cost: O(log |D|)
    /// ties: the smallest such cut
    /// traps: never
    /// example: `let a: [0..3]Int = [1, 2, 3]; prefix(a).first_gt(3) == Some((0..3).r)`
    /// example: `let a: [0..3]Int = [1, 2, 3]; prefix(a).first_gt(6) == None`
    pub fn first_gt(self, target: T) -> ?Cut<D> = todo

/// Internal append-only 2-D prefix/suffix sum table returned by `Prefix<R, C, T>.filling(...)`.
pub struct _FillPrefix2 < R, C, T >:
    p: [R.cuts, C.cuts]T
    rev: Bool
    next: Int

    /// Writes the next cell in row-major fill order: top-left to bottom-right for `Forward`,
    /// bottom-right to top-left for `Reverse`.
    ///
    /// cost: O(1)
    /// traps: when `(r, c)` is not the next cell in fill order
    /// example: `var f = Prefix<0..2, 0..2, Int>.filling(); f.set(0, 0, 1); f.set(0, 1, 2); f.sum(0..1, 0..2) == 3`
    pub fn set(mut self, r: R, c: C, x: T) = todo

    /// The sum above row cut `r` and left of column cut `c`; those cells must be filled.
    ///
    /// cost: O(1)
    /// traps: when a cell it covers is not filled yet
    /// example: `var f = Prefix<0..2, 0..2, Int>.filling(); f.set(0, 0, 1); f.set(0, 1, 2); f.prefix((0..1).r, (0..2).r) == 3`
    pub fn prefix(self, r: Cut<R>, c: Cut<C>) -> T = todo

    /// The sum of `a[r, c]` over `rows` by `cols`; those cells must be filled.
    ///
    /// cost: O(1)
    /// empty: an empty `rows` or `cols` gives `0`
    /// traps: when a cell it covers is not filled yet
    /// example: `var f = Prefix<0..2, 0..2, Int>.filling(); f.set(0, 0, 1); f.set(0, 1, 2); f.sum(0..1, 1..2) == 2`
    pub fn sum(self, rows: Range<R>, cols: Range<C>) -> T = todo

/// Internal 2-D shape of `Prefix<R, C, T>`, constructed by `prefix(a)` or `Prefix<R, C, T>.filling(...)`.
pub struct _Prefix2 < R, C, T >:
    p: [R.cuts, C.cuts]T

    /// 2-D prefix sums over `a`, as `prefix(a)`.
    ///
    /// cost: O(|R| |C|)
    /// empty: an empty `R` or `C` gives a table whose every sum is `0`
    /// traps: never
    /// example: `let g: [0..2, 0..2]Int = [[1, 2], [3, 4]]; Prefix.new(g).sum(0..2, 0..2) == 10`
    pub fn new(a: [R, C]T) -> Prefix<R, C, T> = todo

    /// An all-zero append-only 2-D table filled by `.set(r, c, x)` in row-major `order`.
    ///
    /// cost: O(|R| |C|)
    /// traps: never
    /// example: `var f = Prefix<0..2, 0..2, Int>.filling(); f.set(0, 0, 5); f.sum(0..1, 0..1) == 5`
    pub fn filling(order: FillOrder = FillOrder.Forward) -> FillPrefix<R, C, T> = todo

    /// The sum above row cut `r` and left of column cut `c`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `let g: [0..2, 0..2]Int = [[1, 2], [3, 4]]; prefix(g).prefix((0..1).r, (0..2).r) == 3`
    pub fn prefix(self, r: Cut<R>, c: Cut<C>) -> T = todo

    /// The sum of `a[r, c]` over `rows` by `cols`, by inclusion-exclusion of four prefixes.
    ///
    /// cost: O(1)
    /// empty: an empty `rows` or `cols` gives `0`
    /// traps: never
    /// example: `let g: [0..2, 0..2]Int = [[1, 2], [3, 4]]; prefix(g).sum(0..2, 1..2) == 6 and prefix(g).sum(1..1, 0..2) == 0`
    pub fn sum(self, rows: Range<R>, cols: Range<C>) -> T = todo

/// Internal append-only list prefix/suffix sum table returned by `Prefix<T>.filling(n, ...)`.
pub struct _FillPrefixList < T >:
    p: List<T>
    rev: Bool
    next: Int

    /// Writes index `i` in fill order: `0..n` for `Forward`, `(0..n).rev` for `Reverse`.
    ///
    /// cost: O(1)
    /// traps: when `i` is not the next index in fill order
    /// example: `var f = Prefix<Int>.filling(3); f.set(0, 5); f.set(1, 1); f.prefix(2) == 6`
    pub fn set(mut self, i: Int, x: T) = todo

    /// The sum of the first `k` values; they must be filled.
    ///
    /// cost: O(1)
    /// traps: when `k` is outside `0..=n` or a value below `k` is not filled yet
    /// example: `var f = Prefix<Int>.filling(3); f.set(0, 5); f.prefix(1) == 5 and f.prefix(0) == 0`
    pub fn prefix(self, k: Int) -> T = todo

    /// The sum of `a[i]` for `i` in `r`; those values must be filled.
    ///
    /// cost: O(1)
    /// empty: an empty `r` gives `0`
    /// traps: when `r` leaves `0..n` or covers a value not filled yet
    /// example: `var f = Prefix<Int>.filling(3, order: FillOrder.Reverse); f.set(2, 4); f.set(1, 1); f.sum(1..3) == 5`
    pub fn sum(self, r: Range<Int>) -> T = todo

/// Internal list shape of `Prefix<T>`, constructed by `prefix(a)` or `Prefix<T>.filling(n, ...)`.
pub struct _PrefixList < T >:
    p: List<T>

    /// Prefix sums over a list, as `prefix(a)`; indices are plain `Int`s.
    ///
    /// cost: O(a.len)
    /// empty: `[]` gives a table whose every sum is `0`
    /// traps: never
    /// example: `Prefix.new([1, 2, 3]).sum(0..3) == 6`
    pub fn new(a: List<T>) -> Prefix<T> = todo

    /// An all-zero append-only table of length `n`, filled by `.set(i, x)` in `order`.
    ///
    /// cost: O(n)
    /// traps: when `n < 0`
    /// example: `var f = Prefix<Int>.filling(2); f.set(0, 3); f.set(1, 4); f.sum(0..2) == 7`
    pub fn filling(n: Int, order: FillOrder = FillOrder.Forward) -> FillPrefix<T> = todo

    /// The sum of the first `k` values.
    ///
    /// cost: O(1)
    /// traps: when `k` is outside `0..=a.len`
    /// example: `prefix([1, 2, 3]).prefix(2) == 3 and prefix([1, 2, 3]).prefix(0) == 0`
    pub fn prefix(self, k: Int) -> T = todo

    /// The sum of `a[i]` for `i` in `r`.
    ///
    /// cost: O(1)
    /// empty: an empty `r` gives `0`
    /// traps: when `r` leaves `0..a.len`
    /// example: `prefix([1, 2, 3]).sum(1..3) == 5 and prefix([1, 2, 3]).sum(2..2) == 0`
    pub fn sum(
        self,
        r: Range<Int>,
    ) -> T = todo

/// The prefix folds of `a` under the monoid `op`, indexed by cuts: `p[b]` folds `..b`, and
/// the first cut holds the identity. Needs no inverse, unlike `Prefix`.
///
/// cost: O(n)
/// empty: an empty `D` gives one cut holding the identity
/// order: keyed by `D.cuts`, left to right
/// traps: never
/// example: `let a: [0..3]Int = [1, 3, 2]; prefix_fold(+, a)[(0..2).r] == 4 and prefix_fold(max, a)[(0..3).r] == 3`
pub fn prefix_fold<D, T, M>(op: M, a: [D]T) -> [D.cuts]T = todo

/// The suffix folds of `a` under the monoid `op`, indexed by cuts: `s[b]` folds `b..`, and
/// the last cut holds the identity. With `prefix_fold`, the fold outside `r` is
/// `op(pref[r.l], suff[r.r])`.
///
/// cost: O(n)
/// empty: an empty `D` gives one cut holding the identity
/// order: keyed by `D.cuts`, left to right
/// traps: never
/// example: `let a: [0..3]Int = [1, 3, 2]; suffix_fold(+, a)[(0..1).r] == 5 and suffix_fold(+, a)[(0..3).r] == 0`
pub fn suffix_fold<D, T, M>(op: M, a: [D]T) -> [D.cuts]T = todo

/// Offline difference array: `Diff<D, T>()` for 1-D ranges or `Diff<R, C, T>()` for 2-D grids.
/// O(1) range/rectangle `add`, O(|D|) or O(|R| * |C|) `finish`.
pub struct Diff<D, T>:
    d: [D.cuts]T

    /// An all-zero 1-D difference array over `D`, also written `Diff<D, T>()`.
    ///
    /// cost: O(|D|)
    /// empty: an empty `D` finishes to the empty table
    /// traps: never
    /// example: `Diff<0..2, Int>().finish() == [0, 0]`
    pub fn new() -> Diff<D, T> = todo

    /// Adds `x` to every position in `r`, recorded at its two cuts.
    ///
    /// cost: O(1)
    /// empty: an empty `r` changes nothing
    /// traps: never
    /// example: `var d = Diff<0..4, Int>(); d.add(1..3, 5); d.add(2..4, 1); d.finish() == [0, 5, 6, 1]`
    pub fn add(mut self, r: Range<D>, x: T) = todo

    /// The accumulated value at every position, by one prefix pass.
    ///
    /// cost: O(|D|)
    /// order: keyed by `D`
    /// traps: never
    /// example: `var d = Diff<0..3, Int>(); d.add(0..3, 2); d.finish() == [2, 2, 2]`
    pub fn finish(self) -> [D]T = todo

/// Internal 2-D shape of `Diff<R, C, T>`: O(1) rectangle `add`, O(|R| * |C|) `finish`.
pub struct _Diff2 < R, C, T >:
    d: [R.cuts, C.cuts]T

    /// An all-zero 2-D difference array over `(R, C)`, also written `Diff<R, C, T>()`.
    ///
    /// cost: O(|R| |C|)
    /// traps: never
    /// example: `Diff<0..1, 0..2, Int>().finish() == [[0, 0]]`
    pub fn new() -> Diff<R, C, T> = todo

    /// Adds `x` to every cell in `rows` by `cols`, recorded at four corners.
    ///
    /// cost: O(1)
    /// empty: an empty `rows` or `cols` changes nothing
    /// traps: never
    /// example: `var d = Diff<0..2, 0..2, Int>(); d.add(0..2, 1..2, 3); d.finish() == [[0, 3], [0, 3]]`
    pub fn add(mut self, rows: Range<R>, cols: Range<C>, x: T) = todo

    /// The accumulated value at every cell, by a 2-D prefix pass.
    ///
    /// cost: O(|R| |C|)
    /// order: keyed by `(R, C)`
    /// traps: never
    /// example: `var d = Diff<0..2, 0..2, Int>(); d.add(0..1, 0..2, 1); d.add(0..2, 0..1, 1); d.finish() == [[2, 1], [1, 0]]`
    pub fn finish(self) -> [R, C]T = todo

/// O(|D|) space. Online sweep-line difference accumulator: schedule future range adds in O(1)
/// and advance a cursor through `D` in `order` in O(1) per position.
pub struct SweepAdd<D, T>:
    d: [D.cuts]T

    /// An all-zero online sweep accumulator whose cursor walks `D` in `order` (`Forward` or
    /// `Reverse`).
    ///
    /// cost: O(|D|)
    /// traps: never
    /// example: `var s = SweepAdd<0..2, Int>(); s.advance(0) == 0`
    pub fn new(order: FillOrder = FillOrder.Forward) -> SweepAdd<D, T> = todo

    /// Schedules `+x` across `r`. `r` must not start before the next position the cursor will
    /// advance to (in fill order); positions already passed are not revisited.
    ///
    /// cost: O(1)
    /// empty: an empty `r` changes nothing
    /// traps: when `r` starts before the next position to advance to
    /// example: `var s = SweepAdd<0..3, Int>(); s.add(0..2, 4); let a = s.advance(0); s.add(1..3, 1); let b = s.advance(1); let c = s.advance(2); a == 4 and b == 5 and c == 1`
    pub fn add(mut self, r: Range<D>, x: T) = todo

    /// Moves the cursor to `i`, which must be the next position in fill order, and returns the
    /// value accumulated there.
    ///
    /// cost: O(1)
    /// traps: when `i` is not the next position in fill order
    /// example: `var s = SweepAdd<0..2, Int>(order: FillOrder.Reverse); s.add(0..2, 3); s.advance(1) == 3`
    pub fn advance(mut self, i: D) -> T = todo

    /// The value at the most recently advanced position; `0` before the first `advance`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var s = SweepAdd<0..2, Int>(); s.add(0..1, 4); s.advance(0); s.current == 4`
    pub @get fn current(self) -> T = todo

/// O(|D|) space. Second-order offline difference array for arithmetic-progression range adds:
/// O(1) `add_arith`, O(|D|) `finish`.
pub struct ArithDiff<D, T>:
    d: [D.cuts]T

    /// An all-zero second-order difference array over `D`.
    ///
    /// cost: O(|D|)
    /// traps: never
    /// example: `ArithDiff<0..2, Int>().finish() == [0, 0]`
    pub fn new() -> ArithDiff<D, T> = todo

    /// Adds `start + step * k` to the `k`-th position of `r` (counting from 0), an arithmetic
    /// progression.
    ///
    /// cost: O(1)
    /// empty: an empty `r` changes nothing
    /// traps: never
    /// example: `var d = ArithDiff<0..5, Int>(); d.add_arith(1..4, 10, 2); d.finish() == [0, 10, 12, 14, 0]`
    pub fn add_arith(mut self, r: Range<D>, start: T, step: T) = todo

    /// Adds the constant `x` to every position in `r` (`add_arith(r, x, 0)`).
    ///
    /// cost: O(1)
    /// empty: an empty `r` changes nothing
    /// traps: never
    /// example: `var d = ArithDiff<0..3, Int>(); d.add(0..2, 1); d.add_arith(0..3, 0, 1); d.finish() == [1, 2, 2]`
    pub fn add(mut self, r: Range<D>, x: T) = todo

    /// The accumulated value at every position, by two prefix passes.
    ///
    /// cost: O(|D|)
    /// order: keyed by `D`
    /// traps: never
    /// example: `var d = ArithDiff<0..3, Int>(); d.add_arith(0..3, 5, -1); d.finish() == [5, 4, 3]`
    pub fn finish(self) -> [D]T = todo

/// The fold of every element except one, for each position: `out[i]` folds `a` without
/// `a[i]`, keeping order, with no inverse needed (prefix and suffix folds).
///
/// cost: O(|D|)
/// empty: an empty `D` gives an empty table; a single position gets the identity
/// order: keyed by `D`
/// traps: never
/// example: `let a: [0..3]Int = [1, 2, 3]; prefix_excl(+, a) == [5, 4, 3] and prefix_excl(max, a) == [3, 3, 2]`
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
