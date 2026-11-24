//! Prefix sums, prefix/suffix monoid folds, and offline difference arrays.
//!
//! Call `Prefix(a)` on a `[D]T`, `[R, C]T`, or `List<T>` to build static prefix sums in \(O(n)\) or
//! \(O(r \cdot c)\), then answer any `.fold(...)`, `.cut(k)` or `.prefix(...)` query in \(O(1)\).
//! Use `Diff<D, T>()` or `Diff<R, C, T>()` for \(O(1)\) offline range/rectangle adds and \(O(n)\) /
//! \(O(r \cdot c)\) `finish()`.
use std.algebra.{Monoid}
use std.range.{ends, RingRange, first_true}

/// \(O(n)\) calls of `f`.
///
/// The n + 1 exclusive prefix folds, indexed by `D`'s cuts: `pre[c]` folds everything left of cut
/// `c` (`from`, or `f`'s identity, at the first cut), and `pre[v]` for a position folds through
/// `v`. Suffix folds are `self.rev.scan(f)`, on the same cuts.
///
///     let pre = a.scan(+)
///     pre[r.r] - pre[r.l]          // sum over r; also pre.fold(r)
///     pre[v.l] + a[v] + suf[v.r]   // with suf = a.rev.scan(+)
///
/// The folds run left to right: the value at cut `c.next` is `f(pre[c], a[c.r])`, so `f` need not
/// be commutative. Without `from`, `f` must be an op with an identity (`+`, `*`, `min`, a
/// `Monoid`); a plain closure with neither is a compile error. The table's `prefix(b)`, `all`
/// and `first_*` read stored folds and work for any `f`. Its `fold`, `suffix` and `cyclic_sum`
/// subtract them, so they need `f` to be `+` and trap otherwise: for a range `min`, use a
/// `SparseTable` or `SegTree`.
///
/// cost: \(O(n)\) calls of `f`; \(O(n)\) space
/// empty: one cut, holding `from` or the identity
/// order: keyed by `D.cuts`, left to right
/// traps: never (beyond what `f` traps on); see `fold` for range reads
/// example: `[3, 1, 4].scan(+).prefix(2) == 4`
/// example: `[3, 1, 4].scan(+).fold(1..=2) == 5`
/// example: `[3, 1, 4].scan(+, from: 10).prefix(0) == 10`
@lint(prefix_totals_list, from: "var p = [0]\nfor x in self:\n    p.push(p.last! + x)", to: "let p = self.scan(+)")
pub fn scan<D, T, M>(self: [D]T, f: M, from: ?T = None) -> Prefix<D, T> =
    Prefix<D, T>.of_scan(self, f, from)


fn same<A, B>(a: A, b: B) -> Bool = a == b

/// Fill direction for an append-only `Prefix.filling(...)` table.
pub enum FillOrder = Forward | Reverse

/// Append-only 1-D prefix/suffix sum table returned by `Prefix<D, T>.filling(...)`.
pub struct FillPrefix<D, T: Add + Sub>:
    p: [D.cuts]T
    pub rev: Bool
    pub next: Int

    /// Write the next cell in fill order (`D.first..=D.last` for `Forward`, reverse for `Reverse`).
    ///
    /// Each position is written exactly once, in fill order; the table tracks the next position
    /// it expects.
    ///
    /// cost: \(O(1)\)
    /// traps: when `i` is not the next position in fill order (out of order, repeated, or after
    /// every position is written)
    /// example: `domain D = 0..3; var f = Prefix<D, Int>.filling(); f.set(0, 5); f.set(1, 7); f.prefix(2) == 12`
    pub fn set(mut self, i: D, x: T):
        let in_order = i.ord == next
        assert in_order
        if rev:
            p[i.l] = x + p[i.r]
            next -= 1
        else:
            p[i.r] = p[i.l] + x
            next += 1

    /// The sum of everything left of boundary `b`; zero at the first cut.
    ///
    /// cost: \(O(1)\)
    /// traps: when a position left of `b` is not yet written (with `Reverse`, every position left
    /// of `b` is written last)
    /// example: `domain D = 0..3; var f = Prefix<D, Int>.filling(); f.set(0, 5); f.set(1, 7); f.prefix(1) == 5`
    /// example: `domain D = 0..3; var f = Prefix<D, Int>.filling(); f.prefix(0) == 0`
    pub fn prefix(self, b: Cut<D>) -> T:
        if b.ord == 0: return T.zero
        let in_filled = if rev: next < 0 else: b.ord <= next
        assert in_filled
        return if rev: self.p[D.cuts.nth(0)] - self.p[b] else: self.p[b]

    /// `f[c]`: the same as `prefix(c)`, as on a built `Prefix`; at a position `v`, `f[v]` is the
    /// sum through `v`, the same as `f[v.r]` (R07.1.6).
    ///
    /// cost: \(O(1)\)
    /// traps: when a position left of `c` is not yet written
    /// example: `domain D = 0..3; var f = Prefix<D, Int>.filling(); f.set(0, 5); f.set(1, 7); f[D(1)] == 12`
    pub fn at(self, c: Cut<D>) -> T = self.prefix(c)

    /// The sum of the first `k` positions, whatever `D`'s base: `prefix` at the cut `k` positions
    /// in. An `Int` index `f[k]` is an error (E0320); this is how a count reads the table.
    ///
    /// cost: \(O(1)\)
    /// traps: when `k` is not in `0..=D.count`, or a position among the first `k` is not yet written
    /// example: `domain D = 1..=3; var f = Prefix<D, Int>.filling(); f.set(1, 5); f.set(2, 7); f.cut(2) == 12`
    pub fn cut(self, k: Int) -> T = self.prefix(D.cuts.nth(k))

    /// The sum of `a[i]` for `i` in `r`, or `0` when `r` is empty.
    ///
    /// cost: \(O(1)\)
    /// empty: zero, whatever has been written
    /// traps: when `r` is non-empty and contains a position not yet written
    /// example: `domain D = 0..3; var f = Prefix<D, Int>.filling(); f.set(0, 5); f.set(1, 7); f.fold(0..2) == 12`
    /// example: `domain D = 0..3; var f = Prefix<D, Int>.filling(order: FillOrder.Reverse); f.set(2, 4); f.set(1, 6); f.fold(1..) == 10`
    pub fn fold(self, r: Range<D>) -> T:
        if r.is_empty: return T.zero
        if rev:
            let in_filled = r.l.ord > next
            assert in_filled
            return self.p[r.l] - self.p[r.r]
        self.prefix(r.r) - self.prefix(r.l)

/// Prefix sums over a domain `D`, a 2-D grid `(R, C)`, or a `List<T>`, of `Int`s,
/// `F64`s, `Mod`s, or anything else with `+`, `-` and a zero.
/// Construct statically with `Prefix(a)` in \(O(\lvert D \rvert)\) / \(O(\lvert R \rvert \cdot \lvert C \rvert)\) / \(O(a.\mathit{len})\), or incrementally
/// with `Prefix<...>.filling(order: FillOrder.Forward | FillOrder.Reverse)` and \(O(1)\) `.set(...)`
/// in fill order; `.fold(...)` and `.prefix(...)` are \(O(1)\) over the already-filled prefix or suffix.
/// On `a: [R, C]T`, `Prefix(a)` is the 2-D `Prefix<R, C, T>`, folding rectangles with
/// `fold(rows, cols)`; it is never a 1-D table over the cells.
pub struct Prefix<D, T: Add + Sub>:
    p: [D.cuts]T
    sums: Bool
    rises: Bool
    back: Bool

    /// Build 1-D prefix sums over `a: [D]T`; `Prefix(a)` calls it.
    ///
    /// cost: \(O(\lvert D \rvert)\) time and space
    /// empty: a table whose every query is zero
    /// traps: never
    /// example: `domain D = 0..3; let p = Prefix<D, Int>([D] v => 1 + Int(v)); p.all == 6`
    pub fn new(a: [D]T) -> Prefix<D, T>:
        var p: [D.cuts]T = T.zero
        var rises = true
        for i in D:
            p[i.r] = p[i.l] + a[i]
            if T: Ord:
                if a[i] < T.zero: rises = false
        Prefix(p, true, rises, false)

    /// Create an append-only 1-D prefix/suffix sum table filled via `.set(i, x)` in `order`.
    ///
    /// `Forward` writes `D.first` to `D.last` and answers queries over the written prefix;
    /// `Reverse` writes `D.last` to `D.first` and answers queries over the written suffix.
    ///
    /// cost: \(O(\lvert D \rvert)\) time and space
    /// traps: never
    /// example: `domain D = 0..3; var f = Prefix<D, Int>.filling(); f.set(0, 2); f.fold(0..1) == 2`
    pub fn filling(order: FillOrder = FillOrder.Forward) -> FillPrefix<D, T>:
        let rev = order == FillOrder.Reverse
        let p: [D.cuts]T = T.zero
        FillPrefix(p, rev, if rev: D.count - 1 else: 0)

    /// The sum of everything left of boundary `b`: zero at the first cut, `all` at the last.
    ///
    /// cost: \(O(1)\)
    /// traps: on a suffix table (`a.rev.scan(f)`) with `f` other than `+`, which subtracts
    /// example: `domain D = 0..3; let p = Prefix<D, Int>([D] v => 1 + Int(v)); p.prefix(2) == 3`
    /// example: `domain D = 0..3; let p = Prefix<D, Int>([D] v => 1 + Int(v)); p.prefix(0) == 0`
    pub fn prefix(self, b: Cut<D>) -> T:
        if back:
            assert sums, "a range read subtracts prefix folds, which is only right for a `scan(+)` table"
            if T: Sub:
                return self.all - self.p[b]
        self.p[b]

    // Builds the table in place, so scan returns it without a copy.
    fn of_scan<M>(a: [D]T, f: M, from: ?T) -> Prefix<D, T>:
        let m = Monoid<T>.of(f)
        var p: [D.cuts]T = from ?? m.id
        var acc = from ?? m.id
        var rises = true
        var forward = true
        for (i, x) in a.items:
            let before = acc
            acc = m.op(acc, x)
            if T: Ord:
                if acc < before: rises = false
            let c = a.cut_after(i)
            if c != i.r: forward = false
            p[c] = acc
        Prefix(p, forward and same(f, +), forward and rises, not forward)

    pub fn of_rev_scan<M>(a: [D]T, f: M, from: ?T) -> Prefix<D, T>:
        let m = Monoid<T>.of(f)
        var p: [D.cuts]T = from ?? m.id
        var acc = from ?? m.id
        var forward = true
        for i in a.keys.rev:
            acc = m.op(a[i], acc)
            let c = a.cut_before(i)
            if c != i.l: forward = false
            p[c] = acc
        Prefix(p, forward and same(f, +), false, true)

    /// The sum of `a[i]` for `i` in `r`, or `0` when `r` is empty: `prefix(r.r) - prefix(r.l)`.
    ///
    /// cost: \(O(1)\)
    /// empty: zero
    /// traps: on a table from `scan(f)` with `f` other than `+`
    /// example: `domain D = 0..3; let p = Prefix<D, Int>([D] v => 1 + Int(v)); p.fold(1..=2) == 5`
    /// trap: `[3, 1, 4].scan(min).fold(1..3)`
    pub fn fold(self, r: Range<D>) -> T:
        assert sums, "a range read subtracts prefix folds, which is only right for a `scan(+)` table"
        if r.is_empty: return T.zero
        self.between(r.l, r.r)

    fn between(self, l: Cut<D>, r: Cut<D>) -> T = if back: self.p[l] - self.p[r] else: self.p[r] - self.p[l]

    /// The sum over a wrapping run of positions, e.g. `a.ring.span(i, k)`: the part from `r.start`
    /// to `D.last` plus the part from `D.first` to `r.end` when it wraps.
    ///
    /// cost: \(O(1)\)
    /// empty: zero for an empty span
    /// traps: on a table from `scan(f)` with `f` other than `+`
    /// example: `domain D = 0..3; let a: [D]Int = [D] v => 1 + Int(v); Prefix<D, Int>(a).cyclic_sum(a.ring.span(2, 2)) == 4`
    pub fn cyclic_sum(self, r: RingRange<D>) -> T:
        assert sums, "a range read subtracts prefix folds, which is only right for a `scan(+)` table"
        if r.len == 0: return T.zero
        let (s, e) = (r.start, r.end)
        if s.ord <= e.ord: return self.between(s.l, e.r)
        self.between(s.l, D.cuts.nth(D.count)) + self.between(D.cuts.nth(0), e.r)

    /// `p[c]`: the sum of everything left of cut `c`, the same as `prefix(c)`; at a position `v`,
    /// `p[v]` is the sum through `v`, the same as `p[v.r]` (R07.1.6). On `Int` keys an index is the
    /// cut (decision 349). On a suffix table (`a.rev.scan(f)`), `p[c]` folds everything right of `c`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `domain D = 0..3; let p = Prefix<D, Int>([D] v => 1 + Int(v)); p[D(1)] == 3`
    pub fn at(self, c: Cut<D>) -> T = self.p[c]

    /// The fold of the first `k` positions, whatever `D`'s base: the value at the cut `k`
    /// positions in, the identity at `0` and `all` at `D.count`. An `Int` index `pre[k]` is an
    /// error (E0320, decision 507): a count reads the table here, a typed cut or position by `[]`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `k` is not in `0..=D.count`
    /// example: `[5, -3, -2].scan(+).cut(1) == 5`
    /// example: `[5, -3, -2].scan(+).cut(3) == 0`
    /// example: `domain D = 1..=3; let p: [D]Int = [4, 1, 2]; p.scan(max).cut(2) == 4`
    pub fn cut(self, k: Int) -> T = self.p[D.cuts.nth(k)]

    /// The sum of everything right of boundary `b`: `all - prefix(b)`, zero at the last cut.
    ///
    /// cost: \(O(1)\)
    /// traps: on a table from `scan(f)` with `f` other than `+`
    /// example: `domain D = 0..3; let p = Prefix<D, Int>([D] v => 1 + Int(v)); p.suffix(1) == 5`
    /// trap: `[3, 1, 4].scan(max).suffix(1)`
    pub fn suffix(self, b: Cut<D>) -> T:
        if back: return self.p[b]
        assert sums, "a range read subtracts prefix folds, which is only right for a `scan(+)` table"
        self.all - self.p[b]

    /// The sum of the entire domain: `prefix` at the last cut (a suffix table's first), zero for an
    /// empty domain.
    ///
    /// cost: \(O(1)\)
    /// empty: zero
    /// traps: never
    /// example: `domain D = 0..3; let p = Prefix<D, Int>([D] v => 1 + Int(v)); p.all == 6`
    @get pub fn all(self) -> T:
        self.p[D.cuts.nth(if back: 0 else: D.count)]

    /// The first boundary `b` with `prefix(b) >= target`, or `None` when even `all` is smaller.
    ///
    /// A binary search over the cuts, which needs `prefix` non-decreasing, so every element must be
    /// non-negative; the table records at build time whether that holds. A `target` at or below
    /// zero gives the first cut.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// empty: the first cut when `target <= 0`, else `None`
    /// ties: the first in cut order (zero elements make equal prefixes)
    /// traps: when some element is negative
    /// example: `Int([2, 0, 3].scan(+).first_ge(2)!) == 1`
    /// example: `[2, 0, 3].scan(+).first_ge(6) == None`
    pub fn first_ge(self, target: T) -> ?Cut<D>:
        assert not back, "a suffix table falls left to right; search a forward `scan(+)` instead"
        assert rises, "this binary search needs the prefix never to fall, so no element may be negative"
        if self.all < target: return None
        let lo = (0..D.count).first_true(mid => self.p[D.cuts.nth(mid)] >= target) ?? D.count
        Some(D.cuts.nth(lo))

    /// The first boundary `b` with `prefix(b) > target`, or `None` when `all <= target`.
    ///
    /// The same search as `first_ge`, with the same non-negative requirement.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// empty: the first cut when `target < 0`, else `None`
    /// ties: the first in cut order
    /// traps: when some element is negative
    /// example: `Int([2, 0, 3].scan(+).first_gt(2)!) == 3`
    /// example: `[2, 0, 3].scan(+).first_gt(5) == None`
    pub fn first_gt(self, target: T) -> ?Cut<D>:
        assert not back, "a suffix table falls left to right; search a forward `scan(+)` instead"
        assert rises, "this binary search needs the prefix never to fall, so no element may be negative"
        if self.all <= target: return None
        let lo = (0..D.count).first_true(mid => self.p[D.cuts.nth(mid)] > target) ?? D.count
        Some(D.cuts.nth(lo))

/// Internal append-only 2-D prefix/suffix sum table returned by `Prefix<R, C, T>.filling(...)`.
pub struct _FillPrefix2 <R, C, T: Add + Sub>:
    p: [R.cuts, C.cuts]T
    pub rev: Bool
    pub next: Int

    /// Write the next cell in row-major fill order (top-left to bottom-right for `Forward`,
    /// bottom-right to top-left for `Reverse`).
    ///
    /// cost: \(O(1)\)
    /// traps: when `(r, c)` is not the next cell in fill order
    /// example: `domain R = 0..2; domain C = 0..2; var f = Prefix<R, C, Int>.filling(); f.set(0, 0, 1); f.set(0, 1, 2); f.prefix(1, 2) == 3`
    pub fn set(mut self, r: R, c: C, x: T):
        let in_order = r.ord * C.count + c.ord == next
        assert in_order
        if rev:
            p[r.l, c.l] = x + p[r.r, c.l] + p[r.l, c.r] - p[r.r, c.r]
            next -= 1
        else:
            p[r.r, c.r] = p[r.l, c.r] + p[r.r, c.l] - p[r.l, c.l] + x
            next += 1

    /// The sum of everything above row boundary `r` and left of column boundary `c`; zero when
    /// either cut is the first.
    ///
    /// cost: \(O(1)\)
    /// traps: when that rectangle is non-empty and holds a cell not yet written
    /// example: `domain R = 0..2; domain C = 0..2; var f = Prefix<R, C, Int>.filling(); f.set(0, 0, 1); f.set(0, 1, 2); f.set(1, 0, 3); f.prefix(2, 1) == 4`
    pub fn prefix(self, r: Cut<R>, c: Cut<C>) -> T:
        if r.ord == 0 or c.ord == 0: return T.zero
        let in_filled = if rev: next < 0 else: (r.ord - 1) * C.count + (c.ord - 1) < next
        assert in_filled
        let r0 = R.cuts.nth(0)
        let c0 = C.cuts.nth(0)
        return if rev:
            self.p[r0, c0] - self.p[r, c0] - self.p[r0, c] + self.p[r, c]
        else:
            self.p[r, c]

    /// The sum of `a[r, c]` over `rows` by `cols`, or `0` when either range is empty.
    ///
    /// With `Forward` the rectangle is readable once its bottom-right cell is written; with
    /// `Reverse`, once its top-left cell is.
    ///
    /// cost: \(O(1)\)
    /// empty: zero when `rows` or `cols` is empty
    /// traps: when the rectangle is non-empty and holds a cell not yet written
    /// example: `domain R = 0..2; domain C = 0..2; var f = Prefix<R, C, Int>.filling(); f.set(0, 0, 1); f.set(0, 1, 2); f.set(1, 0, 3); f.fold(0..=1, 0..=0) == 4`
    /// example: `domain R = 0..2; domain C = 0..2; var f = Prefix<R, C, Int>.filling(order: FillOrder.Reverse); f.set(1, 1, 5); f.set(1, 0, 6); f.fold(1..=1, 0..=1) == 11`
    pub fn fold(self, rows: Range<R>, cols: Range<C>) -> T:
        if rows.is_empty or cols.is_empty: return T.zero
        if rev:
            let in_filled = rows.l.ord * C.count + cols.l.ord > next
            assert in_filled
            return self.p[
                rows.l,
                cols.l,
            ] - self.p[rows.r, cols.l] - self.p[
                rows.l,
                cols.r,
            ] + self.p[rows.r, cols.r]
        self.prefix(rows.r, cols.r) - self.prefix(rows.l, cols.r) - self.prefix(
            rows.r,
            cols.l,
        ) + self.prefix(
            rows.l,
            cols.l,
        )

/// Internal 2-D shape of `Prefix<R, C, T>`, constructed by `Prefix(a)` or `Prefix<R, C, T>.filling(...)`.
pub struct _Prefix2 <R, C, T: Add + Sub>:
    p: [R.cuts, C.cuts]T

    /// Build 2-D prefix sums over `a: [R, C]T`; `Prefix(a)` on a grid calls it.
    ///
    /// cost: \(O(\lvert R \rvert \cdot \lvert C \rvert)\) time and space
    /// empty: a table whose every query is zero
    /// traps: never
    /// example: `domain R = 0..2; domain C = 0..3; let p = Prefix<R, C, Int>([R, C] (r, c) => 1 + 3 * Int(r) + Int(c)); p.fold(0..=1, 0..=2) == 21`
    pub fn new(a: [R, C]T) -> Prefix<R, C, T>:
        var p: [R.cuts, C.cuts]T = T.zero
        for r in R:
            for c in C:
                p[r.r, c.r] = p[r.l, c.r] + p[r.r, c.l] - p[r.l, c.l] + a[r, c]
        _Prefix2(p)

    /// Create an append-only 2-D prefix/suffix sum table filled via `.set(r, c, x)` in row-major
    /// `order`.
    ///
    /// cost: \(O(\lvert R \rvert \cdot \lvert C \rvert)\) time and space
    /// traps: never
    /// example: `domain R = 0..2; domain C = 0..2; var f = Prefix<R, C, Int>.filling(); f.set(0, 0, 7); f.fold(0..=0, 0..=0) == 7`
    pub fn filling(order: FillOrder = FillOrder.Forward) -> FillPrefix<R, C, T>:
        let rev = order == FillOrder.Reverse
        let p: [R.cuts, C.cuts]T = T.zero
        _FillPrefix2(p, rev, if rev: R.count * C.count - 1 else: 0)

    /// The sum of everything above row boundary `r` and left of column boundary `c`; zero when
    /// either cut is the first.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `domain R = 0..2; domain C = 0..3; let p = Prefix<R, C, Int>([R, C] (r, c) => 1 + 3 * Int(r) + Int(c)); p.prefix(1, 2) == 3`
    /// example: `domain R = 0..2; domain C = 0..3; let p = Prefix<R, C, Int>([R, C] (r, c) => 1 + 3 * Int(r) + Int(c)); p.prefix(0, 3) == 0`
    pub fn prefix(self, r: Cut<R>, c: Cut<C>) -> T = self.p[r, c]

    /// The sum of `a[r, c]` over `rows` by `cols`, or `0` when either range is empty, by
    /// inclusion-exclusion over the four corner cuts.
    ///
    /// cost: \(O(1)\)
    /// empty: zero when `rows` or `cols` is empty
    /// traps: never
    /// example: `domain R = 0..2; domain C = 0..3; let p = Prefix<R, C, Int>([R, C] (r, c) => 1 + 3 * Int(r) + Int(c)); p.fold(0..=1, 1..=2) == 16`
    pub fn fold(self, rows: Range<R>, cols: Range<C>) -> T:
        self.p[rows.r, cols.r] - self.p[rows.l, cols.r] - self.p[
            rows.r,
            cols.l,
        ] + self.p[
            rows.l,
            cols.l,
        ]

/// Internal append-only list prefix/suffix sum table returned by `Prefix<T>.filling(n, ...)`.
pub struct _FillPrefixList <T: Add + Sub>:
    p: List<T>
    pub rev: Bool
    pub next: Int

    /// Write index `i` in fill order (`0..n` for `Forward`, `(0..n).rev` for `Reverse`).
    ///
    /// cost: \(O(1)\)
    /// traps: when `i` is not the next index in fill order (including any `i` outside `0..n`)
    /// example: `var f = Prefix<Int>.filling(3); f.set(0, 3); f.set(1, 1); f.cut(2) == 4`
    pub fn set(mut self, i: Int, x: T):
        let in_order = i == next
        assert in_order
        if rev:
            p[i] = x + p[i + 1]
            next -= 1
        else:
            p[i + 1] = p[i] + x
            next += 1

    /// The sum of the first `k` values; zero for `k == 0`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `k` is not in `0..=n`, or one of the first `k` values is not yet written
    /// example: `var f = Prefix<Int>.filling(3); f.set(0, 3); f.set(1, 1); f.cut(1) == 3`
    pub fn cut(self, k: Int) -> T:
        if k == 0: return T.zero
        let in_filled = if rev: next < 0 else: k <= next
        assert in_filled
        return if rev: self.p[0] - self.p[k] else: self.p[k]

    /// The sum of `a[i]` for `i` in `r`, or zero when `r` is empty.
    ///
    /// cost: \(O(1)\)
    /// empty: zero, wherever `r` lies
    /// traps: when `r` is non-empty and reaches outside `0..n` or holds an index not yet written
    /// example: `var f = Prefix<Int>.filling(3, order: FillOrder.Reverse); f.set(2, 4); f.set(1, 1); f.fold(1..3) == 5`
    pub fn fold(self, r: Range<Int>) -> T:
        if r.start >= r.end: return T.zero
        if rev:
            let in_filled = r.start > next
            assert in_filled
            return self.p[r.start] - self.p[r.end]
        self.cut(r.end) - self.cut(r.start)

/// Internal list shape of `Prefix<T>`, constructed by `Prefix(a)` or `Prefix<T>.filling(n, ...)`.
pub struct _PrefixList <T: Sub>:
    p: List<T>

    /// Build 1-D prefix sums over `a: List<T>`; `Prefix(a)` on a list calls it.
    ///
    /// cost: \(O(a.\mathit{len})\) time and space
    /// empty: a table whose every query is zero
    /// traps: never
    /// example: `Prefix<Int>([3, 1, 4]).cut(3) == 8`
    pub fn new(a: List<T>) -> Prefix<T>:
        var p = List.filled(a.len + 1, T.zero)
        for i in 0..a.len:
            p[i + 1] = p[i] + a[i]
        _PrefixList(p)

    /// Create an append-only 1-D prefix/suffix sum table of length `n`, filled via `.set(i, x)`
    /// in `order`.
    ///
    /// cost: \(O(n)\) time and space
    /// traps: when `n < 0`
    /// example: `var f = Prefix<Int>.filling(2); f.set(0, 9); f.fold(0..1) == 9`
    pub fn filling(n: Int, order: FillOrder = FillOrder.Forward) -> FillPrefix<T>:
        assert n >= 0
        let rev = order == FillOrder.Reverse
        _FillPrefixList(List.filled(n + 1, T.zero), rev, if rev: n - 1 else: 0)

    /// The sum of the first `k` values; zero for `k == 0`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `k` is not in `0..=n`
    /// example: `Prefix<Int>([3, 1, 4]).cut(2) == 4`
    /// example: `Prefix<Int>([3, 1, 4]).cut(0) == 0`
    pub fn cut(self, k: Int) -> T = self.p[k]

    /// The sum of `a[i]` for `i` in `r`, or zero when `r` is empty.
    ///
    /// cost: \(O(1)\)
    /// empty: zero, wherever `r` lies
    /// traps: when `r` is non-empty and reaches outside `0..n`
    /// example: `Prefix<Int>([3, 1, 4]).fold(1..3) == 5`
    /// example: `Prefix<Int>([3, 1, 4]).fold(2..2) == 0`
    pub fn fold(self, r: Range<Int>) -> T:
        self.p[r.end] - self.p[r.start]

/// Prefix monoid folds of `a`, indexed by `Cut<D>`: `p[b]` is the fold of `..b`,
/// with `p[D.cuts.nth(0)] == m.id`. \(O(n)\).
///
/// The fold runs left to right, `p[c.next] == op(p[c], a[c.r])`, so `op` need not be
/// commutative or invertible.
///
/// cost: \(O(n)\) calls of `op`; \(O(n)\) space
/// empty: one cut, holding the identity
/// order: keyed by `D.cuts`, left to right
/// traps: never (beyond what `op` traps on)
/// example: `prefix_fold(*, [2, 3, 4])[2] == 6`
/// example: `prefix_fold(*, [2, 3, 4])[0] == 1`
pub fn prefix_fold<D, T, M>(op: M, a: [D]T) -> [D.cuts]T:
    let m = Monoid<T>.of(op)
    var p: [D.cuts]T = m.id
    var acc = m.id
    for i in D:
        acc = m.op(acc, a[i])
        p[i.r] = acc
    p

/// Offline difference array: `Diff<D, T>()` for 1-D ranges or `Diff<R, C, T>()` for 2-D grids.
/// \(O(1)\) range/rectangle `add`, \(O(\lvert D \rvert)\) or \(O(\lvert R \rvert \cdot \lvert C \rvert)\) `finish`.
pub struct Diff<D, T: Add + Sub>:
    d: [D.cuts]T

    /// Create an all-zero 1-D difference array over `D`.
    ///
    /// cost: \(O(\lvert D \rvert)\) time and space
    /// traps: never
    /// example: `domain D = 0..3; Diff<D, Int>().finish()[D(1)] == 0`
    pub fn new() -> Diff<D, T>:
        let d: [D.cuts]T = T.zero
        Diff(d)

    /// Add `x` to every position in `r`; an empty `r` changes nothing.
    ///
    /// cost: \(O(1)\)
    /// empty: no effect
    /// traps: never
    /// example: `domain D = 0..4; var d = Diff<D, Int>(); d.add(1..3, 5); d.add(2..=3, 1); [d.finish()[v] for v in D] == [0, 5, 6, 1]`
    pub fn add(mut self, r: Range<D>, x: T):
        if r.is_empty: return
        d[r.l] += x
        d[r.r] -= x

    /// The accumulated values across `D`: each position's total of the `add`s covering it. The
    /// structure is unchanged and can take more `add`s.
    ///
    /// cost: \(O(\lvert D \rvert)\) time and space
    /// empty: an empty array
    /// order: keyed by `D`
    /// traps: never
    /// example: `domain D = 0..3; var d = Diff<D, Int>(); d.add(0..=1, 2); d.finish()[D(1)] == 2 and d.finish()[D(2)] == 0`
    pub fn finish(self) -> [D]T:
        var out: [D]T = T.zero
        var cur = T.zero
        for i in D:
            cur = cur + d[i.l]
            out[i] = cur
        out

/// Internal 2-D shape of `Diff<R, C, T>`: \(O(1)\) rectangle `add`, \(O(\lvert R \rvert \cdot \lvert C \rvert)\) `finish`.
pub struct _Diff2 <R, C, T: Add + Sub>:
    d: [R.cuts, C.cuts]T

    /// Create an all-zero 2-D difference array over `(R, C)`.
    ///
    /// cost: \(O(\lvert R \rvert \cdot \lvert C \rvert)\) time and space
    /// traps: never
    /// example: `domain R = 0..2; domain C = 0..2; Diff<R, C, Int>().finish()[R(1), C(1)] == 0`
    pub fn new() -> Diff<R, C, T>:
        let d: [R.cuts, C.cuts]T = T.zero
        _Diff2(d)

    /// Add `x` to every cell in `rows` by `cols`; an empty side changes nothing.
    ///
    /// cost: \(O(1)\)
    /// empty: no effect when `rows` or `cols` is empty
    /// traps: never
    /// example: `domain R = 0..2; domain C = 0..3; var d = Diff<R, C, Int>(); d.add(0..=1, 1..=2, 3); d.finish()[R(1), C(2)] == 3 and d.finish()[R(0), C(0)] == 0`
    pub fn add(mut self, rows: Range<R>, cols: Range<C>, x: T):
        if rows.is_empty or cols.is_empty: return
        d[rows.l, cols.l] += x
        d[rows.l, cols.r] -= x
        d[rows.r, cols.l] -= x
        d[rows.r, cols.r] += x

    /// The accumulated grid across `(R, C)`: each cell's total of the `add`s covering it. The
    /// structure is unchanged and can take more `add`s.
    ///
    /// cost: \(O(\lvert R \rvert \cdot \lvert C \rvert)\) time and space
    /// empty: an empty grid
    /// order: keyed by `(R, C)`, row-major
    /// traps: never
    /// example: `domain R = 0..2; domain C = 0..2; var d = Diff<R, C, Int>(); d.add(1..=1, 0..=1, 4); d.add(0..=1, 1..=1, 1); d.finish()[R(1), C(1)] == 5`
    pub fn finish(self) -> [R, C]T:
        var p: [R.cuts, C.cuts]T = T.zero
        var out: [R, C]T = T.zero
        for r in R:
            for c in C:
                let v = p[r.l, c.r] + p[r.r, c.l] - p[r.l, c.l] + d[r.l, c.l]
                p[r.r, c.r] = v
                out[r, c] = v
        out

/// \(O(\lvert D \rvert)\) space. Online sweep-line difference accumulator: schedule future range adds in \(O(1)\)
/// and advance a cursor through `D` in `order` in \(O(1)\) per position.
pub struct SweepAdd<D, T: Add + Sub>:
    d: [D.cuts]T
    cur: T
    pub rev: Bool

    /// Create an all-zero online sweep accumulator in `order` (`Forward` or `Reverse`), with no
    /// position advanced yet.
    ///
    /// cost: \(O(\lvert D \rvert)\) time and space
    /// traps: never
    /// example: `domain D = 0..3; var s = SweepAdd<D, Int>(); s.advance(0) == 0`
    pub fn new(order: FillOrder = FillOrder.Forward) -> SweepAdd<D, T>:
        let d: [D.cuts]T = T.zero
        SweepAdd(d, T.zero, order == FillOrder.Reverse)

    /// Schedule `+x` across `r`, which must start at or ahead of the current sweep cursor: no
    /// position of `r` may have been advanced past already. An empty `r` changes nothing.
    ///
    /// cost: \(O(1)\)
    /// empty: no effect
    /// traps: when `r` is non-empty and holds a position already advanced to
    /// example: `domain D = 0..4; var s = SweepAdd<D, Int>(); s.add(1..=2, 5); let a = s.advance(0); let b = s.advance(1); a == 0 and b == 5`
    pub fn add(mut self, r: Range<D>, x: T):
        if r.is_empty: return
        if rev:
            d[r.r] += x
            d[r.l] -= x
        else:
            d[r.l] += x
            d[r.r] -= x

    /// Advance the cursor to `i`, the next position in fill order, and return the accumulated
    /// value there: the total of every scheduled `add` covering `i`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `i` is not the next position in fill order
    /// example: `domain D = 0..3; var s = SweepAdd<D, Int>(order: FillOrder.Reverse); s.add(1..=2, 4); let a = s.advance(2); s.add(0..=1, 1); let b = s.advance(1); a == 4 and b == 5`
    pub fn advance(mut self, i: D) -> T:
        cur += (if rev: d[i.r] else: d[i.l])
        cur

    /// The accumulated value at the most recently advanced position, as `advance` returned it.
    ///
    /// cost: \(O(1)\)
    /// traps: when no position has been advanced yet
    /// example: `domain D = 0..3; var s = SweepAdd<D, Int>(); s.add(0..=0, 7); s.advance(0); s.current == 7`
    @get pub fn current(self) -> T:
        cur

/// \(O(\lvert D \rvert)\) space. Second-order offline difference array for arithmetic-progression range adds:
/// \(O(1)\) `add_arith`, \(O(\lvert D \rvert)\) `finish`.
pub struct ArithDiff<D, T: Add + Sub + Mul>:
    d: [D.cuts]T
    e: [D.cuts]T

    /// Create an all-zero second-order difference array over `D`.
    ///
    /// cost: \(O(\lvert D \rvert)\) time and space
    /// traps: never
    /// example: `domain D = 0..3; ArithDiff<D, Int>().finish()[D(2)] == 0`
    pub fn new() -> ArithDiff<D, T>:
        let d: [D.cuts]T = T.zero
        let e: [D.cuts]T = T.zero
        ArithDiff(d, e)

    /// Add `start + step * (i.ord - r.start.ord)` to every position `i` in `r`: `start` at the
    /// first position of `r`, growing by `step` per position. An empty `r` changes nothing.
    ///
    /// cost: \(O(1)\)
    /// empty: no effect
    /// traps: never
    /// example: `domain D = 0..5; var d = ArithDiff<D, Int>(); d.add_arith(1..=3, 10, 2); [d.finish()[v] for v in D] == [0, 10, 12, 14, 0]`
    pub fn add_arith(mut self, r: Range<D>, start: T, step: T):
        if r.is_empty: return
        d[r.l] += start
        e[r.l] += step
        d[r.r] -= (start + step * r.len)
        e[r.r] -= step

    /// Add a constant `x` to every position in `r`: `add_arith(r, x, 0)`.
    ///
    /// cost: \(O(1)\)
    /// empty: no effect
    /// traps: never
    /// example: `domain D = 0..5; var d = ArithDiff<D, Int>(); d.add_arith(1..=3, 10, 2); d.add(0..2, 1); [d.finish()[v] for v in D] == [1, 11, 12, 14, 0]`
    pub fn add(mut self, r: Range<D>, x: T):
        if r.is_empty: return
        d[r.l] += x
        d[r.r] -= x

    /// The accumulated values across `D`. The structure is unchanged and can take more adds.
    ///
    /// cost: \(O(\lvert D \rvert)\) time and space
    /// empty: an empty array
    /// order: keyed by `D`
    /// traps: never
    /// example: `domain D = 0..3; var d = ArithDiff<D, Int>(); d.add_arith(0..=2, 1, 1); [d.finish()[v] for v in D] == [1, 2, 3]`
    pub fn finish(self) -> [D]T:
        var out: [D]T = T.zero
        var val = T.zero
        var s = T.zero
        for i in D:
            s = s + e[i.l]
            val = val + d[i.l]
            out[i] = val
            val = val + s
        out

impl Show for Prefix<D, T>:
    fn show(q): "{q.p}"

@test
fn one_d_sums_match_brute():
    domain V = 1..=9
    let a = [V] _ => rand(-20, 20)
    let p = Prefix<V, Int>(a)
    for l in V.cuts:
        for r in V.cuts:
            if r < l: continue
            let want = sum(a[i] for i in V if l <= i.l and i.r <= r)
            assert p.fold(l..r) == want
            assert p.prefix(r) - p.prefix(l) == want
    for i in V: assert p.fold(i..=i) == a[i]

@test
fn two_d_sums_match_brute():
    domain R = 0..4
    domain C = 1..=5
    let g = [R, C] (_, _) => rand(-9, 9)
    let p = Prefix<R, C, Int>(g)
    for r0 in R.cuts:
        for r1 in R.cuts:
            for c0 in C.cuts:
                for c1 in C.cuts:
                    if r1 < r0 or c1 < c0: continue
                    var want = 0
                    for r in R:
                        for c in C:
                            if r0 <= r.l and r.r <= r1 and c0 <= c.l and c.r <= c1: want += g[r, c]
                    assert p.fold(r0..r1, c0..c1) == want

@test
fn a_plain_range_and_list_take_plain_ints():
    let xs = [4, -1, 7, 0, 3]
    let p = Prefix<Int>(xs)
    for lo in 0..=5:
        for hi in lo..=5:
            let want = sum(xs[i] for i in lo..hi)
            assert p.fold(lo..hi) == want
            assert p.cut(hi) - p.cut(lo) == want

@test
fn prefix_and_suffix_folds_cover_outside_ranges():
    domain V = 1..=8
    let a = [V] _ => rand(-15, 15)
    let pref = prefix_fold(max, a)
    let suff = Prefix<V, Int>.of_rev_scan(a, max, None)
    for l in V:
        for r in l..=V.last!:
            let range = l..=r
            var want = Int.MIN
            for i in V:
                if i < l or i > r: want max= a[i]
            assert max(pref[range.start], suff[range.end]) == want

@test
fn diff_1d_and_2d_match_brute():
    domain V = 1..=9
    var plain: [V]Int = 0
    var d1 = Diff<V, Int>()
    for _ in 0..40:
        let l = V.nth(rand(0, 8))
        let r = V.nth(rand(l.ord, 8))
        let x = rand(-7, 7)
        for i in l..=r: plain[i] += x
        d1.add(l..=r, x)
    d1.add(V(3)..V(3), 99)
    let got1 = d1.finish()
    for i in V: assert got1[i] == plain[i]

    domain R = 0..4
    domain C = 1..=5
    var g: [R, C]Int = 0
    var d2 = Diff<R, C, Int>()
    for _ in 0..40:
        let r1 = R.nth(rand(0, 3))
        let r2 = R.nth(rand(r1.ord, 3))
        let c1 = C.nth(rand(0, 4))
        let c2 = C.nth(rand(c1.ord, 4))
        let x = rand(-5, 5)
        for r in r1..=r2:
            for c in c1..=c2: g[r, c] += x
        d2.add(r1..=r2, c1..=c2, x)
    let got2 = d2.finish()
    for r in R:
        for c in C: assert got2[r, c] == g[r, c]

@test
fn filling_1d_forward_and_reverse_match_brute():
    domain V = 0..7
    let a = [V] _ => rand(-5, 5)
    var f = Prefix<V, Int>.filling()
    for i in V:
        f.set(i, a[i])
        assert f.prefix(i.r) == Prefix<V, Int>(a).prefix(i.r)
    var b = Prefix<V, Int>.filling(order: FillOrder.Reverse)
    for k in (0..7).rev:
        let i = V.nth(k)
        b.set(i, a[i])
        let want = sum(a[j] for j in i..=V.last!)
        assert b.fold(i..=V.last!) == want
        assert b.fold(V.last!.r..V.last!.r) == 0

@test
fn filling_2d_forward_and_reverse_match_brute():
    domain R = 0..3
    domain C = 0..4
    let g = [R, C] (_, _) => rand(-5, 5)
    let full = Prefix<R, C, Int>(g)
    var f = Prefix<R, C, Int>.filling()
    for r in R:
        for c in C:
            f.set(r, c, g[r, c])
            assert f.prefix(r.r, c.r) == full.prefix(r.r, c.r)
    var b = Prefix<R, C, Int>.filling(order: FillOrder.Reverse)
    for rk in (0..3).rev:
        for ck in (0..4).rev:
            let (r, c) = (R.nth(rk), C.nth(ck))
            b.set(r, c, g[r, c])
            assert b.fold(r..=R.last!, c..=C.last!) == full.fold(r..=R.last!, c..=C.last!)
            assert b.fold(r..=R.last!, C.last!.r..C.last!.r) == 0
            assert b.fold(R.last!.r..R.last!.r, c..=C.last!) == 0

@test
fn filling_list_forward_and_reverse_match_brute():
    let xs = [rand(-5, 5) for _ in 0..8]
    let full = Prefix<Int>(xs)
    var f = Prefix<Int>.filling(8)
    for i in 0..8:
        f.set(i, xs[i])
        assert f.cut(i + 1) == full.cut(i + 1)
    var b = Prefix<Int>.filling(8, order: FillOrder.Reverse)
    for i in (0..8).rev:
        b.set(i, xs[i])
        assert b.fold(i..8) == full.fold(i..8)

@test
fn scan_folds_every_prefix():
    let p = [3, 1, 4, 1, 5].scan(+)
    assert [p.prefix(i) for i in 0..=5] == [0, 3, 4, 8, 9, 14]
    let q = [3, 1, 4, 1, 5].scan(max, from: Int.MIN)
    assert [q.prefix(i) for i in 1..=5] == [3, 3, 4, 4, 5]
    for _ in 0..50:
        let xs = [rand(-9, 9) for _ in 0..rand(1, 12)]
        let sums = xs.scan(+)
        for i in 0..=xs.len: assert sums.prefix(i) == sum(xs[j] for j in 0..i)
        for l in 0..=xs.len:
            for r in l..=xs.len: assert sums.fold(l..r) == sum(xs[j] for j in l..r)

@test
fn suffix_scan_matches_right_to_left_fold():
    domain V = 1..=4
    let a = [V] i => [2, 3, 5, 7][i.ord]
    let sf = Prefix<V, Int>.of_rev_scan(a, *, None)
    assert [sf[c] for c in V.cuts] == [210, 105, 35, 7, 1]
    let p = a.scan(+)
    let sp = Prefix<V, Int>.of_rev_scan(a, +, None)
    for c in V.cuts: assert p.suffix(c) == sp[c]

@test
fn rev_scan_folds_every_suffix():
    domain V = 0..7
    let a = [V] _ => rand(-9, 9)
    let s = Prefix<V, Int>.of_rev_scan(a, +, None)
    let m = Prefix<V, Int>.of_rev_scan(a, max, None)
    assert s.all == sum(a[i] for i in V)
    for c in V.cuts:
        let right = [a[i] for i in V if c <= i.l]
        assert s[c] == right.sum()
        assert s.suffix(c) == right.sum()
        assert s.prefix(c) == s.all - right.sum()
        assert m[c] == (right.max() ?? Monoid<Int>.of(max).id)
        for d in V.cuts:
            if d < c: continue
            assert s.fold(c..d) == sum(a[i] for i in V if c <= i.l and i.r <= d)
