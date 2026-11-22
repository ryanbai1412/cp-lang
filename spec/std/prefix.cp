//! Prefix sums, prefix/suffix monoid folds, and offline difference arrays.
//!
//! Call `prefix(a)` on a `[D]T`, `[R, C]T`, or `List<T>` to build static prefix sums in O(n) or
//! O(r * c), then answer any `.sum(...)` or `.prefix(...)` query in O(1).
//! Use `Diff<D, T>()` or `Diff<R, C, T>()` for O(1) offline range/rectangle adds and O(n) /
//! O(r * c) `finish()`.
use std.algebra.{Monoid}
use std.seq.{ends, RingRange}

/// O(n) calls of `f`. The n + 1 exclusive prefix folds, indexed by `D`'s cuts: `pre[c]` folds
/// everything left of cut `c` (`from`, or `f`'s identity, at the first cut), and `pre[v]` for a
/// position folds through `v`. Suffix folds are `self.rev.scan(f)`, on the same cuts.
///
///     let pre = a.scan(+)
///     pre[r.r] - pre[r.l]          // sum over r; also pre.fold(r)
///     pre[v.l] + a[v] + suf[v.r]   // with suf = a.rev.scan(+)
///
/// The folds run left to right: the value at cut `c.next` is `f(pre[c], a[c.r])`, so `f` need not
/// be commutative. Without `from`, `f` must be an op with an identity (`+`, `*`, `min`, a
/// `Monoid`); a plain closure with neither is a compile error. The table's `prefix(b)` is
/// `pre[b]`; its `sum`, `get`, `suffix`, `cyclic_sum` and `first_*` subtract stored folds, so
/// they are range folds only when `f` is `+` (a `Group`).
///
/// cost: O(n) calls of `f`; O(n) space
/// empty: one cut, holding `from` or the identity
/// order: keyed by `D.cuts`, left to right
/// traps: never (beyond what `f` traps on)
/// example: `[3, 1, 4].scan(+).prefix(2) == 4`
/// example: `[3, 1, 4].scan(+).sum(1..=2) == 5`
/// example: `[3, 1, 4].scan(+, from: 10).prefix(0) == 10`
pub fn scan<D, T>(self: [D]T, f: (T, T) -> T, from: ?T = None) -> Prefix<D, T> = todo

/// Fill direction for an append-only `Prefix.filling(...)` table.
pub enum FillOrder = Forward | Reverse

/// Append-only 1-D prefix/suffix sum table returned by `Prefix<D, T>.filling(...)`.
pub struct FillPrefix<D, T>:
    p: [D.cuts]T
    rev: Bool
    next: Int

    /// Write the next cell in fill order (`D.first..=D.last` for `Forward`, reverse for `Reverse`).
    ///
    /// Each position is written exactly once, in fill order; the table tracks the next position
    /// it expects.
    ///
    /// cost: O(1)
    /// traps: when `i` is not the next position in fill order (out of order, repeated, or after
    /// every position is written)
    /// example: `domain D = 0..3; var f = Prefix<D, Int>.filling(); f.set(0, 5); f.set(1, 7); f.prefix(2) == 12`
    pub fn set(mut self, i: D, x: T) = todo

    /// The sum of everything left of boundary `b`; zero at the first cut.
    ///
    /// cost: O(1)
    /// traps: when a position left of `b` is not yet written (with `Reverse`, every position left
    /// of `b` is written last)
    /// example: `domain D = 0..3; var f = Prefix<D, Int>.filling(); f.set(0, 5); f.set(1, 7); f.prefix(1) == 5`
    /// example: `domain D = 0..3; var f = Prefix<D, Int>.filling(); f.prefix(0) == 0`
    pub fn prefix(self, b: Cut<D>) -> T = todo

    /// The sum of `a[i]` for `i` in `r`, or `0` when `r` is empty.
    ///
    /// cost: O(1)
    /// empty: zero, whatever has been written
    /// traps: when `r` is non-empty and contains a position not yet written
    /// example: `domain D = 0..3; var f = Prefix<D, Int>.filling(); f.set(0, 5); f.set(1, 7); f.sum(0..2) == 12`
    /// example: `domain D = 0..3; var f = Prefix<D, Int>.filling(order: FillOrder.Reverse); f.set(2, 4); f.set(1, 6); f.sum(1..) == 10`
    pub fn sum(self, r: Range<D>) -> T = todo

/// Prefix sums over a domain `D`, a 2-D grid `(R, C)`, or a `List<T>`, of `Int`s,
/// `F64`s, `Mod`s, or anything else with `+`, `-` and a zero.
/// Construct statically with `prefix(a)` in O(|D|) / O(|R| * |C|) / O(a.len), or incrementally
/// with `Prefix<...>.filling(order: FillOrder.Forward | FillOrder.Reverse)` and O(1) `.set(...)`
/// in fill order; `.sum(...)` and `.prefix(...)` are O(1) over the already-filled prefix or suffix.
pub struct Prefix<D, T>:
    p: [D.cuts]T

    /// Build 1-D prefix sums over `a: [D]T`; `prefix(a)` calls it.
    ///
    /// cost: O(|D|) time and space
    /// empty: a table whose every query is zero
    /// traps: never
    /// example: `domain D = 0..3; let p = Prefix<D, Int>.new([D] v => 1 + Int(v)); p.all == 6`
    pub fn new(a: [D]T) -> Prefix<D, T> = todo

    /// Create an append-only 1-D prefix/suffix sum table filled via `.set(i, x)` in `order`.
    ///
    /// `Forward` writes `D.first` to `D.last` and answers queries over the written prefix;
    /// `Reverse` writes `D.last` to `D.first` and answers queries over the written suffix.
    ///
    /// cost: O(|D|) time and space
    /// traps: never
    /// example: `domain D = 0..3; var f = Prefix<D, Int>.filling(); f.set(0, 2); f.sum(0..1) == 2`
    pub fn filling(order: FillOrder = FillOrder.Forward) -> FillPrefix<D, T> = todo

    /// The sum of everything left of boundary `b`: zero at the first cut, `all` at the last.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain D = 0..3; let p = Prefix<D, Int>.new([D] v => 1 + Int(v)); p.prefix(2) == 3`
    /// example: `domain D = 0..3; let p = Prefix<D, Int>.new([D] v => 1 + Int(v)); p.prefix(0) == 0`
    pub fn prefix(self, b: Cut<D>) -> T = todo

    /// The sum of `a[i]` for `i` in `r`, or `0` when `r` is empty: `prefix(r.r) - prefix(r.l)`.
    ///
    /// cost: O(1)
    /// empty: zero
    /// traps: never
    /// example: `domain D = 0..3; let p = Prefix<D, Int>.new([D] v => 1 + Int(v)); p.sum(1..=2) == 5`
    pub fn sum(self, r: Range<D>) -> T = todo

    /// The sum over a wrapping run of positions, e.g. `a.ring.span(i, k)`: the part from `r.start`
    /// to `D.last` plus the part from `D.first` to `r.end` when it wraps.
    ///
    /// cost: O(1)
    /// empty: zero for an empty span
    /// traps: never
    /// example: `domain D = 0..3; let a: [D]Int = [D] v => 1 + Int(v); Prefix<D, Int>.new(a).cyclic_sum(a.ring.span(2, 2)) == 4`
    pub fn cyclic_sum(self, r: RingRange<D>) -> T = todo

    /// Recover the original element `a[i]`, as `prefix(i.r) - prefix(i.l)`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain D = 0..3; let p = Prefix<D, Int>.new([D] v => 1 + Int(v)); p.get(1) == 2`
    pub fn get(self, i: D) -> T = todo

    /// The sum of everything right of boundary `b`: `all - prefix(b)`, zero at the last cut.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain D = 0..3; let p = Prefix<D, Int>.new([D] v => 1 + Int(v)); p.suffix(1) == 5`
    pub fn suffix(self, b: Cut<D>) -> T = todo

    /// The sum of the entire domain: `prefix` at the last cut, zero for an empty domain.
    ///
    /// cost: O(1)
    /// empty: zero
    /// traps: never
    /// example: `domain D = 0..3; let p = Prefix<D, Int>.new([D] v => 1 + Int(v)); p.all == 6`
    pub @get fn all(self) -> T = todo

    /// The first boundary `b` with `prefix(b) >= target`, or `None` when even `all` is smaller.
    ///
    /// A binary search over the cuts, which needs `prefix` non-decreasing, so every element must be
    /// non-negative; the table records at build time whether that holds. A `target` at or below
    /// zero gives the first cut.
    ///
    /// cost: O(log |D|)
    /// empty: the first cut when `target <= 0`, else `None`
    /// ties: the first in cut order (zero elements make equal prefixes)
    /// traps: when some element is negative
    /// example: `Int([2, 0, 3].scan(+).first_ge(2)!) == 1`
    /// example: `[2, 0, 3].scan(+).first_ge(6) == None`
    pub fn first_ge(self, target: T) -> ?Cut<D> = todo

    /// The first boundary `b` with `prefix(b) > target`, or `None` when `all <= target`.
    ///
    /// The same search as `first_ge`, with the same non-negative requirement.
    ///
    /// cost: O(log |D|)
    /// empty: the first cut when `target < 0`, else `None`
    /// ties: the first in cut order
    /// traps: when some element is negative
    /// example: `Int([2, 0, 3].scan(+).first_gt(2)!) == 3`
    /// example: `[2, 0, 3].scan(+).first_gt(5) == None`
    pub fn first_gt(self, target: T) -> ?Cut<D> = todo

/// Internal append-only 2-D prefix/suffix sum table returned by `Prefix<R, C, T>.filling(...)`.
pub struct _FillPrefix2 < R, C, T >:
    p: [R.cuts, C.cuts]T
    rev: Bool
    next: Int

    /// Write the next cell in row-major fill order (top-left to bottom-right for `Forward`,
    /// bottom-right to top-left for `Reverse`).
    ///
    /// cost: O(1)
    /// traps: when `(r, c)` is not the next cell in fill order
    /// example: `domain R = 0..2; domain C = 0..2; var f = Prefix<R, C, Int>.filling(); f.set(0, 0, 1); f.set(0, 1, 2); f.prefix(1, 2) == 3`
    pub fn set(mut self, r: R, c: C, x: T) = todo

    /// The sum of everything above row boundary `r` and left of column boundary `c`; zero when
    /// either cut is the first.
    ///
    /// cost: O(1)
    /// traps: when that rectangle is non-empty and holds a cell not yet written
    /// example: `domain R = 0..2; domain C = 0..2; var f = Prefix<R, C, Int>.filling(); f.set(0, 0, 1); f.set(0, 1, 2); f.set(1, 0, 3); f.prefix(2, 1) == 4`
    pub fn prefix(self, r: Cut<R>, c: Cut<C>) -> T = todo

    /// The sum of `a[r, c]` over `rows` by `cols`, or `0` when either range is empty.
    ///
    /// With `Forward` the rectangle is readable once its bottom-right cell is written; with
    /// `Reverse`, once its top-left cell is.
    ///
    /// cost: O(1)
    /// empty: zero when `rows` or `cols` is empty
    /// traps: when the rectangle is non-empty and holds a cell not yet written
    /// example: `domain R = 0..2; domain C = 0..2; var f = Prefix<R, C, Int>.filling(); f.set(0, 0, 1); f.set(0, 1, 2); f.set(1, 0, 3); f.sum(0..=1, 0..=0) == 4`
    /// example: `domain R = 0..2; domain C = 0..2; var f = Prefix<R, C, Int>.filling(order: FillOrder.Reverse); f.set(1, 1, 5); f.set(1, 0, 6); f.sum(1..=1, 0..=1) == 11`
    pub fn sum(self, rows: Range<R>, cols: Range<C>) -> T = todo

/// Internal 2-D shape of `Prefix<R, C, T>`, constructed by `prefix(a)` or `Prefix<R, C, T>.filling(...)`.
pub struct _Prefix2 < R, C, T >:
    p: [R.cuts, C.cuts]T

    /// Build 2-D prefix sums over `a: [R, C]T`; `prefix(a)` on a grid calls it.
    ///
    /// cost: O(|R| * |C|) time and space
    /// empty: a table whose every query is zero
    /// traps: never
    /// example: `domain R = 0..2; domain C = 0..3; let p = Prefix<R, C, Int>.new([R, C] (r, c) => 1 + 3 * Int(r) + Int(c)); p.sum(0..=1, 0..=2) == 21`
    pub fn new(a: [R, C]T) -> Prefix<R, C, T> = todo

    /// Create an append-only 2-D prefix/suffix sum table filled via `.set(r, c, x)` in row-major
    /// `order`.
    ///
    /// cost: O(|R| * |C|) time and space
    /// traps: never
    /// example: `domain R = 0..2; domain C = 0..2; var f = Prefix<R, C, Int>.filling(); f.set(0, 0, 7); f.sum(0..=0, 0..=0) == 7`
    pub fn filling(order: FillOrder = FillOrder.Forward) -> FillPrefix<R, C, T> = todo

    /// The sum of everything above row boundary `r` and left of column boundary `c`; zero when
    /// either cut is the first.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `domain R = 0..2; domain C = 0..3; let p = Prefix<R, C, Int>.new([R, C] (r, c) => 1 + 3 * Int(r) + Int(c)); p.prefix(1, 2) == 3`
    /// example: `domain R = 0..2; domain C = 0..3; let p = Prefix<R, C, Int>.new([R, C] (r, c) => 1 + 3 * Int(r) + Int(c)); p.prefix(0, 3) == 0`
    pub fn prefix(self, r: Cut<R>, c: Cut<C>) -> T = todo

    /// The sum of `a[r, c]` over `rows` by `cols`, or `0` when either range is empty, by
    /// inclusion-exclusion over the four corner cuts.
    ///
    /// cost: O(1)
    /// empty: zero when `rows` or `cols` is empty
    /// traps: never
    /// example: `domain R = 0..2; domain C = 0..3; let p = Prefix<R, C, Int>.new([R, C] (r, c) => 1 + 3 * Int(r) + Int(c)); p.sum(0..=1, 1..=2) == 16`
    pub fn sum(self, rows: Range<R>, cols: Range<C>) -> T = todo

/// Internal append-only list prefix/suffix sum table returned by `Prefix<T>.filling(n, ...)`.
pub struct _FillPrefixList < T >:
    p: List<T>
    rev: Bool
    next: Int

    /// Write index `i` in fill order (`0..n` for `Forward`, `(0..n).rev` for `Reverse`).
    ///
    /// cost: O(1)
    /// traps: when `i` is not the next index in fill order (including any `i` outside `0..n`)
    /// example: `var f = Prefix<Int>.filling(3); f.set(0, 3); f.set(1, 1); f.prefix(2) == 4`
    pub fn set(mut self, i: Int, x: T) = todo

    /// The sum of the first `k` values; zero for `k == 0`.
    ///
    /// cost: O(1)
    /// traps: when `k` is not in `0..=n`, or one of the first `k` values is not yet written
    /// example: `var f = Prefix<Int>.filling(3); f.set(0, 3); f.set(1, 1); f.prefix(1) == 3`
    pub fn prefix(self, k: Int) -> T = todo

    /// The sum of `a[i]` for `i` in `r`, or zero when `r` is empty.
    ///
    /// cost: O(1)
    /// empty: zero, wherever `r` lies
    /// traps: when `r` is non-empty and reaches outside `0..n` or holds an index not yet written
    /// example: `var f = Prefix<Int>.filling(3, order: FillOrder.Reverse); f.set(2, 4); f.set(1, 1); f.sum(1..3) == 5`
    pub fn sum(self, r: Range<Int>) -> T = todo

/// Internal list shape of `Prefix<T>`, constructed by `prefix(a)` or `Prefix<T>.filling(n, ...)`.
pub struct _PrefixList < T >:
    p: List<T>

    /// Build 1-D prefix sums over `a: List<T>`; `prefix(a)` on a list calls it.
    ///
    /// cost: O(a.len) time and space
    /// empty: a table whose every query is zero
    /// traps: never
    /// example: `Prefix<Int>.new([3, 1, 4]).prefix(3) == 8`
    pub fn new(a: List<T>) -> Prefix<T> = todo

    /// Create an append-only 1-D prefix/suffix sum table of length `n`, filled via `.set(i, x)`
    /// in `order`.
    ///
    /// cost: O(n) time and space
    /// traps: when `n < 0`
    /// example: `var f = Prefix<Int>.filling(2); f.set(0, 9); f.sum(0..1) == 9`
    pub fn filling(n: Int, order: FillOrder = FillOrder.Forward) -> FillPrefix<T> = todo

    /// The sum of the first `k` values; zero for `k == 0`.
    ///
    /// cost: O(1)
    /// traps: when `k` is not in `0..=n`
    /// example: `Prefix<Int>.new([3, 1, 4]).prefix(2) == 4`
    /// example: `Prefix<Int>.new([3, 1, 4]).prefix(0) == 0`
    pub fn prefix(self, k: Int) -> T = todo

    /// The sum of `a[i]` for `i` in `r`, or zero when `r` is empty.
    ///
    /// cost: O(1)
    /// empty: zero, wherever `r` lies
    /// traps: when `r` is non-empty and reaches outside `0..n`
    /// example: `Prefix<Int>.new([3, 1, 4]).sum(1..3) == 5`
    /// example: `Prefix<Int>.new([3, 1, 4]).sum(2..2) == 0`
    pub fn sum(
        self,
        r: Range<Int>,
    ) -> T = todo

/// Prefix monoid folds of `a`, indexed by `Cut<D>`: `p[b]` is the fold of `..b`,
/// with `p[D.cuts.nth(0)] == m.id`. O(n).
///
/// The fold runs left to right, `p[c.next] == op(p[c], a[c.r])`, so `op` need not be
/// commutative or invertible.
///
/// cost: O(n) calls of `op`; O(n) space
/// empty: one cut, holding the identity
/// order: keyed by `D.cuts`, left to right
/// traps: never (beyond what `op` traps on)
/// example: `prefix_fold(*, [2, 3, 4])[2] == 6`
/// example: `prefix_fold(*, [2, 3, 4])[0] == 1`
pub fn prefix_fold<D, T, M>(op: M, a: [D]T) -> [D.cuts]T = todo

/// Suffix monoid folds of `a`, indexed by `Cut<D>`: `s[b]` is the fold of `b..`,
/// with `s[D.cuts.nth(D.K)] == m.id`. With `prefix_fold`, the fold outside `r` is
/// `m.op(pref[r.start], suff[r.end])`. O(n).
///
/// Each suffix keeps the elements' left-to-right order, `s[c] == op(a[c.r], s[c.next])`, so
/// `op` need not be commutative.
///
/// cost: O(n) calls of `op`; O(n) space
/// empty: one cut, holding the identity
/// order: keyed by `D.cuts`; each entry folds left to right
/// traps: never (beyond what `op` traps on)
/// example: `suffix_fold(*, [2, 3, 4])[1] == 12`
/// example: `suffix_fold(*, [2, 3, 4])[3] == 1`
pub fn suffix_fold<D, T, M>(op: M, a: [D]T) -> [D.cuts]T = todo

/// Offline difference array: `Diff<D, T>()` for 1-D ranges or `Diff<R, C, T>()` for 2-D grids.
/// O(1) range/rectangle `add`, O(|D|) or O(|R| * |C|) `finish`.
pub struct Diff<D, T>:
    d: [D.cuts]T

    /// Create an all-zero 1-D difference array over `D`.
    ///
    /// cost: O(|D|) time and space
    /// traps: never
    /// example: `domain D = 0..3; Diff<D, Int>.new().finish()[D(1)] == 0`
    pub fn new() -> Diff<D, T> = todo

    /// Add `x` to every position in `r`; an empty `r` changes nothing.
    ///
    /// cost: O(1)
    /// empty: no effect
    /// traps: never
    /// example: `domain D = 0..4; var d = Diff<D, Int>.new(); d.add(1..3, 5); d.add(2..=3, 1); [d.finish()[v] for v in D] == [0, 5, 6, 1]`
    pub fn add(mut self, r: Range<D>, x: T) = todo

    /// The accumulated values across `D`: each position's total of the `add`s covering it. The
    /// structure is unchanged and can take more `add`s.
    ///
    /// cost: O(|D|) time and space
    /// empty: an empty array
    /// order: keyed by `D`
    /// traps: never
    /// example: `domain D = 0..3; var d = Diff<D, Int>.new(); d.add(0..=1, 2); d.finish()[D(1)] == 2 and d.finish()[D(2)] == 0`
    pub fn finish(self) -> [D]T = todo

/// Internal 2-D shape of `Diff<R, C, T>`: O(1) rectangle `add`, O(|R| * |C|) `finish`.
pub struct _Diff2 < R, C, T >:
    d: [R.cuts, C.cuts]T

    /// Create an all-zero 2-D difference array over `(R, C)`.
    ///
    /// cost: O(|R| * |C|) time and space
    /// traps: never
    /// example: `domain R = 0..2; domain C = 0..2; Diff<R, C, Int>.new().finish()[R(1), C(1)] == 0`
    pub fn new() -> Diff<R, C, T> = todo

    /// Add `x` to every cell in `rows` by `cols`; an empty side changes nothing.
    ///
    /// cost: O(1)
    /// empty: no effect when `rows` or `cols` is empty
    /// traps: never
    /// example: `domain R = 0..2; domain C = 0..3; var d = Diff<R, C, Int>.new(); d.add(0..=1, 1..=2, 3); d.finish()[R(1), C(2)] == 3 and d.finish()[R(0), C(0)] == 0`
    pub fn add(mut self, rows: Range<R>, cols: Range<C>, x: T) = todo

    /// The accumulated grid across `(R, C)`: each cell's total of the `add`s covering it. The
    /// structure is unchanged and can take more `add`s.
    ///
    /// cost: O(|R| * |C|) time and space
    /// empty: an empty grid
    /// order: keyed by `(R, C)`, row-major
    /// traps: never
    /// example: `domain R = 0..2; domain C = 0..2; var d = Diff<R, C, Int>.new(); d.add(1..=1, 0..=1, 4); d.add(0..=1, 1..=1, 1); d.finish()[R(1), C(1)] == 5`
    pub fn finish(self) -> [R, C]T = todo

/// O(|D|) space. Online sweep-line difference accumulator: schedule future range adds in O(1)
/// and advance a cursor through `D` in `order` in O(1) per position.
pub struct SweepAdd<D, T>:
    d: [D.cuts]T

    /// Create an all-zero online sweep accumulator in `order` (`Forward` or `Reverse`), with no
    /// position advanced yet.
    ///
    /// cost: O(|D|) time and space
    /// traps: never
    /// example: `domain D = 0..3; var s = SweepAdd<D, Int>.new(); s.advance(0) == 0`
    pub fn new(order: FillOrder = FillOrder.Forward) -> SweepAdd<D, T> = todo

    /// Schedule `+x` across `r`, which must start at or ahead of the current sweep cursor: no
    /// position of `r` may have been advanced past already. An empty `r` changes nothing.
    ///
    /// cost: O(1)
    /// empty: no effect
    /// traps: when `r` is non-empty and holds a position already advanced to
    /// example: `domain D = 0..4; var s = SweepAdd<D, Int>.new(); s.add(1..=2, 5); let a = s.advance(0); let b = s.advance(1); a == 0 and b == 5`
    pub fn add(mut self, r: Range<D>, x: T) = todo

    /// Advance the cursor to `i`, the next position in fill order, and return the accumulated
    /// value there: the total of every scheduled `add` covering `i`.
    ///
    /// cost: O(1)
    /// traps: when `i` is not the next position in fill order
    /// example: `domain D = 0..3; var s = SweepAdd<D, Int>.new(order: FillOrder.Reverse); s.add(1..=2, 4); let a = s.advance(2); s.add(0..=1, 1); let b = s.advance(1); a == 4 and b == 5`
    pub fn advance(mut self, i: D) -> T = todo

    /// The accumulated value at the most recently advanced position, as `advance` returned it.
    ///
    /// cost: O(1)
    /// traps: when no position has been advanced yet
    /// example: `domain D = 0..3; var s = SweepAdd<D, Int>.new(); s.add(0..=0, 7); s.advance(0); s.current == 7`
    pub @get fn current(self) -> T = todo

/// O(|D|) space. Second-order offline difference array for arithmetic-progression range adds:
/// O(1) `add_arith`, O(|D|) `finish`.
pub struct ArithDiff<D, T>:
    d: [D.cuts]T

    /// Create an all-zero second-order difference array over `D`.
    ///
    /// cost: O(|D|) time and space
    /// traps: never
    /// example: `domain D = 0..3; ArithDiff<D, Int>.new().finish()[D(2)] == 0`
    pub fn new() -> ArithDiff<D, T> = todo

    /// Add `start + step * (i.ord - r.start.ord)` to every position `i` in `r`: `start` at the
    /// first position of `r`, growing by `step` per position. An empty `r` changes nothing.
    ///
    /// cost: O(1)
    /// empty: no effect
    /// traps: never
    /// example: `domain D = 0..5; var d = ArithDiff<D, Int>.new(); d.add_arith(1..=3, 10, 2); [d.finish()[v] for v in D] == [0, 10, 12, 14, 0]`
    pub fn add_arith(mut self, r: Range<D>, start: T, step: T) = todo

    /// Add a constant `x` to every position in `r`: `add_arith(r, x, 0)`.
    ///
    /// cost: O(1)
    /// empty: no effect
    /// traps: never
    /// example: `domain D = 0..5; var d = ArithDiff<D, Int>.new(); d.add_arith(1..=3, 10, 2); d.add(0..2, 1); [d.finish()[v] for v in D] == [1, 11, 12, 14, 0]`
    pub fn add(mut self, r: Range<D>, x: T) = todo

    /// The accumulated values across `D`. The structure is unchanged and can take more adds.
    ///
    /// cost: O(|D|) time and space
    /// empty: an empty array
    /// order: keyed by `D`
    /// traps: never
    /// example: `domain D = 0..3; var d = ArithDiff<D, Int>.new(); d.add_arith(0..=2, 1, 1); [d.finish()[v] for v in D] == [1, 2, 3]`
    pub fn finish(self) -> [D]T = todo

/// O(|D|). Exclusive monoid fold: `out[i]` is the fold of every element in `a` except `i`,
/// without requiring inverses.
///
/// `out[i] == op(prefix_fold(op, a)[i.l], suffix_fold(op, a)[i.r])`: the elements before `i`
/// then those after, in order, so `op` need not be commutative. A one-element array gives the
/// identity.
///
/// cost: O(|D|) calls of `op`; O(|D|) space
/// empty: an empty array
/// order: keyed by `D`
/// traps: never (beyond what `op` traps on)
/// example: `prefix_excl(*, [2, 3, 4])[1] == 8`
/// example: `prefix_excl(+, [5])[0] == 0`
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
