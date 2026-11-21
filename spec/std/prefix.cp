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
    p: [D.bounds]T
    rev: Bool
    next: Int

    /// Write the next cell in fill order (`D.first..=D.last` for `Forward`, reverse for `Reverse`). O(1).
    pub fn set(mut self, i: D, x: T):
        let in_order = i.ord == next
        assert in_order
        if rev:
            p[i.l] = x + p[i.r]
            next -= 1
        else:
            p[i.r] = p[i.l] + x
            next += 1

    /// The sum of everything left of boundary `b`. O(1).
    pub fn prefix(self, b: Bounds<D>) -> T:
        if b.ord == 0: return T.zero
        let in_filled = if rev: next < 0 else: b.ord <= next
        assert in_filled
        return if rev: self.p[D.bounds.nth(0)] - self.p[b] else: self.p[b]

    /// The sum of `a[i]` for `i` in `r`, or `0` when `r` is empty. O(1).
    pub fn sum(self, r: Range<D>) -> T:
        if r.is_empty: return T.zero
        let in_filled = if rev: r.start.ord > next else: r.end.ord <= next
        assert in_filled
        return if rev: self.p[r.start] - self.p[r.end] else: self.p[r.end] - self.p[r.start]

/// Prefix sums over a domain `D`, a 2-D grid `(R, C)`, or a `List<T>`, of `Int`s,
/// `F64`s, `Mod`s, or anything else with `+`, `-` and a zero.
/// Construct statically with `prefix(a)` in O(|D|) / O(|R| * |C|) / O(a.len), or incrementally
/// with `Prefix<...>.filling(order: FillOrder.Forward | FillOrder.Reverse)` and O(1) `.set(...)`
/// in fill order; `.sum(...)` and `.prefix(...)` are O(1) over the already-filled prefix or suffix.
pub struct Prefix<D, T>:
    p: [D.bounds]T

    /// Build 1-D prefix sums over `a: [D]T`. O(|D|).
    pub fn new(a: [D]T) -> Prefix<D, T>:
        var p: [D.bounds]T = T.zero
        for i in D:
            p[i.r] = p[i.l] + a[i]
        Prefix(p)

    /// Create an append-only 1-D prefix/suffix sum table filled via `.set(i, x)` in `order`. O(|D|).
    pub fn filling(order: FillOrder = FillOrder.Forward) -> FillPrefix<D, T>:
        let rev = order == FillOrder.Reverse
        var p: [D.bounds]T = T.zero
        FillPrefix(p, rev, if rev: D.K - 1 else: 0)

    /// The sum of everything left of boundary `b`. O(1).
    pub fn prefix(self, b: Bounds<D>) -> T = self.p[b]

    /// The sum of `a[i]` for `i` in `r`, or `0` when `r` is empty. O(1).
    pub fn sum(self, r: Range<D>) -> T = if r.is_empty: T.zero else: self.p[r.end] - self.p[r.start]

    /// O(1). Sum of a wrapping range `r` on a cyclic domain `D`.
    pub fn cyclic_sum(self, r: Range<D>) -> T = todo

    /// O(1). Recover the original element `a[i]`.
    pub fn get(self, i: D) -> T = todo

    /// O(1). Sum of everything right of boundary `b`.
    pub fn suffix(self, b: Bounds<D>) -> T = todo

    /// O(1). Total sum of the entire domain.
    pub @get fn all(self) -> T = todo

    /// O(log |D|). Smallest boundary `b` with `prefix(b) >= target`, for non-negative values, or `none`.
    pub fn first_ge(self, target: T) -> ?Bounds<D> = todo

    /// O(log |D|). Smallest boundary `b` with `prefix(b) > target`, for non-negative values, or `none`.
    pub fn first_gt(self, target: T) -> ?Bounds<D> = todo

/// Internal append-only 2-D prefix/suffix sum table returned by `Prefix<R, C, T>.filling(...)`.
pub struct _FillPrefix2 < R, C, T >:
    p: [R.bounds, C.bounds]T
    rev: Bool
    next: Int

    /// Write the next cell in row-major fill order (top-left to bottom-right for `Forward`, bottom-right to top-left for `Reverse`). O(1).
    pub fn set(mut self, r: R, c: C, x: T):
        let in_order = r.ord * C.K + c.ord == next
        assert in_order
        if rev:
            p[r.l, c.l] = x + p[r.r, c.l] + p[r.l, c.r] - p[r.r, c.r]
            next -= 1
        else:
            p[r.r, c.r] = p[r.l, c.r] + p[r.r, c.l] - p[r.l, c.l] + x
            next += 1

    /// The sum of everything above row boundary `r` and left of column boundary `c`. O(1).
    pub fn prefix(self, r: Bounds<R>, c: Bounds<C>) -> T:
        if r.ord == 0 or c.ord == 0: return T.zero
        let in_filled = if rev: next < 0 else: (r.ord - 1) * C.K + (c.ord - 1) < next
        assert in_filled
        let r0 = R.bounds.nth(0)
        let c0 = C.bounds.nth(0)
        return if rev:
            self.p[r0, c0] - self.p[r, c0] - self.p[r0, c] + self.p[r, c]
        else:
            self.p[r, c]

    /// The sum of `a[r, c]` over `rows` by `cols`, or `0` when either range is empty. O(1).
    pub fn sum(self, rows: Range<R>, cols: Range<C>) -> T:
        if rows.is_empty or cols.is_empty: return T.zero
        let in_filled = if rev: rows.start.ord * C.K + cols.start.ord > next
        else: (rows.end.ord - 1) * C.K + (cols.end.ord - 1) < next
        assert in_filled
        return if rev:
            self.p[
                rows.start,
                cols.start,
            ] - self.p[rows.end, cols.start] - self.p[
                rows.start,
                cols.end,
            ] + self.p[rows.end, cols.end]
        else:
            self.p[rows.end, cols.end] - self.p[rows.start, cols.end] - self.p[
                rows.end,
                cols.start,
            ] + self.p[
                rows.start,
                cols.start,
            ]

/// Internal 2-D shape of `Prefix<R, C, T>`, constructed by `prefix(a)` or `Prefix<R, C, T>.filling(...)`.
pub struct _Prefix2 < R, C, T >:
    p: [R.bounds, C.bounds]T

    /// Build 2-D prefix sums over `a: [R, C]T`. O(|R| * |C|).
    pub fn new(a: [R, C]T) -> Prefix<R, C, T>:
        var p: [R.bounds, C.bounds]T = T.zero
        for r in R:
            for c in C:
                p[r.r, c.r] = p[r.l, c.r] + p[r.r, c.l] - p[r.l, c.l] + a[r, c]
        _Prefix2(p)

    /// Create an append-only 2-D prefix/suffix sum table filled via `.set(r, c, x)` in row-major `order`. O(|R| * |C|).
    pub fn filling(order: FillOrder = FillOrder.Forward) -> FillPrefix<R, C, T>:
        let rev = order == FillOrder.Reverse
        var p: [R.bounds, C.bounds]T = T.zero
        _FillPrefix2(p, rev, if rev: R.K * C.K - 1 else: 0)

    /// The sum of everything above row boundary `r` and left of column boundary `c`. O(1).
    pub fn prefix(self, r: Bounds<R>, c: Bounds<C>) -> T = self.p[r, c]

    /// The sum of `a[r, c]` over `rows` by `cols`, or `0` when either range is empty. O(1).
    pub fn sum(self, rows: Range<R>, cols: Range<C>) -> T:
        if rows.is_empty or cols.is_empty: return T.zero
        return self.p[rows.end, cols.end] - self.p[rows.start, cols.end] - self.p[
            rows.end,
            cols.start,
        ] + self.p[
            rows.start,
            cols.start,
        ]

/// Internal append-only list prefix/suffix sum table returned by `Prefix<T>.filling(n, ...)`.
pub struct _FillPrefixList < T >:
    p: List<T>
    rev: Bool
    next: Int

    /// Write index `i` in fill order (`0..n` for `Forward`, `(0..n).rev` for `Reverse`). O(1).
    pub fn set(mut self, i: Int, x: T):
        let in_order = i == next
        assert in_order
        if rev:
            p[i] = x + p[i + 1]
            next -= 1
        else:
            p[i + 1] = p[i] + x
            next += 1

    /// The sum of the first `k` values. O(1).
    pub fn prefix(self, k: Int) -> T:
        if k == 0: return T.zero
        let in_filled = if rev: next < 0 else: k <= next
        assert in_filled
        return if rev: self.p[0] - self.p[k] else: self.p[k]

    /// The sum of `a[i]` for `i` in `r`, or zero when `r` is empty. O(1).
    pub fn sum(self, r: Range<Int>) -> T:
        if r.is_empty: return T.zero
        let in_filled = if rev: r.start > next else: r.end <= next
        assert in_filled
        return if rev: self.p[r.start] - self.p[r.end] else: self.p[r.end] - self.p[r.start]

/// Internal list shape of `Prefix<T>`, constructed by `prefix(a)` or `Prefix<T>.filling(n, ...)`.
pub struct _PrefixList < T >:
    p: List<T>

    /// Build 1-D prefix sums over `a: List<T>`. O(a.len).
    pub fn new(a: List<T>) -> Prefix<T>:
        var p = List.filled(a.len + 1, T.zero)
        for i in 0..a.len:
            p[i + 1] = p[i] + a[i]
        _PrefixList(p)

    /// Create an append-only 1-D prefix/suffix sum table of length `n`, filled via `.set(i, x)` in `order`. O(n).
    pub fn filling(n: Int, order: FillOrder = FillOrder.Forward) -> FillPrefix<T>:
        assert n >= 0
        let rev = order == FillOrder.Reverse
        _FillPrefixList(List.filled(n + 1, T.zero), rev, if rev: n - 1 else: 0)

    /// The sum of the first `k` values. O(1).
    pub fn prefix(self, k: Int) -> T = self.p[k]

    /// The sum of `a[i]` for `i` in `r`, or zero when `r` is empty. O(1).
    pub fn sum(
        self,
        r: Range<Int>,
    ) -> T = if r.is_empty: T.zero else: self.p[r.end] - self.p[r.start]

/// Prefix monoid folds of `a`, indexed by `Bounds<D>`: `p[b]` is the fold of `..b`,
/// with `p[D.bounds.nth(0)] == m.id`. O(n).
pub fn prefix_fold<D, T, M>(op: M, a: [D]T) -> [D.bounds]T:
    let m = Monoid<T>.of(op)
    var p: [D.bounds]T = m.id
    for i in D:
        p[i.r] = m.op(p[i.l], a[i])
    p

/// Suffix monoid folds of `a`, indexed by `Bounds<D>`: `s[b]` is the fold of `b..`,
/// with `s[D.bounds.nth(D.K)] == m.id`. With `prefix_fold`, the fold outside `r` is
/// `m.op(pref[r.start], suff[r.end])`. O(n).
pub fn suffix_fold<D, T, M>(op: M, a: [D]T) -> [D.bounds]T:
    let m = Monoid<T>.of(op)
    var s: [D.bounds]T = m.id
    for i in D.rev:
        s[i.l] = m.op(a[i], s[i.r])
    s

/// Offline difference array: `Diff<D, T>()` for 1-D ranges or `Diff<R, C, T>()` for 2-D grids.
/// O(1) range/rectangle `add`, O(|D|) or O(|R| * |C|) `finish`.
pub struct Diff<D, T>:
    d: [D.bounds]T

    /// Create an all-zero 1-D difference array over `D`. O(|D|).
    pub fn new() -> Diff<D, T>:
        var d: [D.bounds]T = T.zero
        Diff(d)

    /// Add `x` to every position in `r`. O(1).
    pub fn add(mut self, r: Range<D>, x: T):
        if r.is_empty: return
        d[r.start] = d[r.start] + x
        d[r.end] = d[r.end] - x

    /// Materialize the accumulated values across `D`. O(|D|).
    pub fn finish(self) -> [D]T:
        var out: [D]T = T.zero
        var cur = T.zero
        for i in D:
            cur = cur + d[i.l]
            out[i] = cur
        out

/// Internal 2-D shape of `Diff<R, C, T>`: O(1) rectangle `add`, O(|R| * |C|) `finish`.
pub struct _Diff2 < R, C, T >:
    d: [R.bounds, C.bounds]T

    /// Create an all-zero 2-D difference array over `(R, C)`. O(|R| * |C|).
    pub fn new() -> Diff<R, C, T>:
        var d: [R.bounds, C.bounds]T = T.zero
        _Diff2(d)

    /// Add `x` to every cell in `rows` by `cols`. O(1).
    pub fn add(mut self, rows: Range<R>, cols: Range<C>, x: T):
        if rows.is_empty or cols.is_empty: return
        d[rows.start, cols.start] = d[rows.start, cols.start] + x
        d[rows.start, cols.end] = d[rows.start, cols.end] - x
        d[rows.end, cols.start] = d[rows.end, cols.start] - x
        d[rows.end, cols.end] = d[rows.end, cols.end] + x

    /// Materialize the accumulated grid across `(R, C)`. O(|R| * |C|).
    pub fn finish(self) -> [R, C]T:
        var p: [R.bounds, C.bounds]T = T.zero
        var out: [R, C]T = T.zero
        for r in R:
            for c in C:
                let v = p[r.l, c.r] + p[r.r, c.l] - p[r.l, c.l] + d[r.l, c.l]
                p[r.r, c.r] = v
                out[r, c] = v
        out

/// O(|D|) space. Online sweep-line difference accumulator: schedule future range adds in O(1)
/// and advance a cursor through `D` in `order` in O(1) per position.
pub struct SweepAdd<D, T>:
    d: [D.bounds]T

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
    d: [D.bounds]T

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
fn one_d_sums_match_brute():
    domain V = 1..=9
    let a = [V] _ => rand(-10, 10)
    let pre = prefix(a)
    for l in V:
        for r in V:
            assert pre.sum(l..=r) == sum(a[i] for i in l..=r)
            assert pre.sum(l..r) == sum(a[i] for i in l..r)
    assert pre.prefix(V.first.l) == 0
    assert pre.prefix(V.last.r) == sum(a[i] for i in V)

@test
fn two_d_sums_match_brute():
    domain R = 0..5
    domain C = 1..=6
    let a = [R, C] (_, _) => rand(-10, 10)
    let pre = prefix(a)
    for r1 in R:
        for r2 in r1..=R.last:
            for c1 in C:
                for c2 in c1..=C.last:
                    let want = sum(a[r, c] for r in r1..=r2 for c in c1..=c2)
                    assert pre.sum(r1..=r2, c1..=c2) == want
    assert pre.prefix(R.first.l, C.first.l) == 0
    assert pre.prefix(R.last.r, C.last.r) == sum(a[r, c] for r in R for c in C)
    assert pre.sum(R(2)..R(2), C(1)..=C(6)) == 0
    assert pre.sum(R(0)..=R(4), C(3)..C(3)) == 0

@test
fn a_plain_range_and_list_take_plain_ints():
    let a: [0..5]Int = [3, 1, 4, 1, 5]
    let p = prefix(a)
    assert p.prefix(0) == 0 and p.prefix(2) == 4 and p.prefix(5) == 14
    assert p.sum(1..3) == 5 and p.sum(1..=3) == 6 and p.sum(2..2) == 0
    let b: [1..=3]Int = [10, 20, 30]
    let q = prefix(b)
    assert q.sum(1..=3) == 60 and q.sum(2..=3) == 50 and q.prefix(1) == 0
    let xs = [3, 1, 4, 1, 5]
    let r = prefix(xs)
    assert r.prefix(0) == 0 and r.prefix(3) == 8 and r.sum(1..4) == 6 and r.sum(2..2) == 0

@test
fn prefix_and_suffix_folds_cover_outside_ranges():
    domain V = 1..=8
    let a = [V] _ => rand(-15, 15)
    let pref = prefix_fold(max, a)
    let suff = suffix_fold(max, a)
    for l in V:
        for r in l..=V.last:
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
        let r = V.nth(rand(l.to_idx, 8))
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
        let r2 = R.nth(rand(r1.to_idx, 3))
        let c1 = C.nth(rand(0, 4))
        let c2 = C.nth(rand(c1.to_idx, 4))
        let x = rand(-5, 5)
        for r in r1..=r2:
            for c in c1..=c2: g[r, c] += x
        d2.add(r1..=r2, c1..=c2, x)
    let got2 = d2.finish()
    for r in R:
        for c in C: assert got2[r, c] == g[r, c]

@test
fn filling_1d_forward_and_reverse_match_brute():
    domain V = 1..=8
    let a = [V] _ => rand(-10, 10)
    var fwd = Prefix<V, Int>.filling()
    for i in V:
        fwd.set(i, a[i])
        for l in V.first..=i:
            for r in l..=i:
                assert fwd.sum(l..=r) == sum(a[k] for k in l..=r)
        assert fwd.prefix(i.r) == sum(a[k] for k in V.first..=i)
    assert fwd.sum(V(3)..V(3)) == 0

    var rev = Prefix<V, Int>.filling(order: FillOrder.Reverse)
    for i in V.rev:
        rev.set(i, a[i])
        for l in i..=V.last:
            for r in l..=V.last:
                assert rev.sum(l..=r) == sum(a[k] for k in l..=r)
    assert rev.prefix(V.first.l) == 0
    assert rev.prefix(V.last.r) == sum(a[k] for k in V)

@test
fn filling_2d_forward_and_reverse_match_brute():
    domain R = 0..4
    domain C = 1..=5
    let a = [R, C] (_, _) => rand(-10, 10)

    var fwd = Prefix<R, C, Int>.filling()
    for r in R:
        for c in C:
            fwd.set(r, c, a[r, c])
            for r1 in R.first..=r:
                for c1 in C.first..=c:
                    let want = sum(a[x, y] for x in r1..=r for y in c1..=c)
                    assert fwd.sum(r1..=r, c1..=c) == want
            assert fwd.prefix(r.r, c.r) == sum(a[x, y] for x in R.first..=r for y in C.first..=c)

    var rev = Prefix<R, C, Int>.filling(order: FillOrder.Reverse)
    for r in R.rev:
        for c in C.rev:
            rev.set(r, c, a[r, c])
            for r2 in r..=R.last:
                for c2 in c..=C.last:
                    let want = sum(a[x, y] for x in r..=r2 for y in c..=c2)
                    assert rev.sum(r..=r2, c..=c2) == want
    assert rev.prefix(R.last.r, C.last.r) == sum(a[x, y] for x in R for y in C)

@test
fn filling_list_forward_and_reverse_match_brute():
    let xs = [3, -1, 4, 1, 5, -9]
    var fwd = Prefix<Int>.filling(xs.len)
    for i in 0..xs.len:
        fwd.set(i, xs[i])
        for l in 0..=i:
            assert fwd.sum(l..=i) == sum(xs[k] for k in l..=i)
        assert fwd.prefix(i + 1) == sum(xs[k] for k in 0..=i)

    var rev = Prefix<Int>.filling(xs.len, order: FillOrder.Reverse)
    for i in (0..xs.len).rev:
        rev.set(i, xs[i])
        for r in i..xs.len:
            assert rev.sum(i..=r) == sum(xs[k] for k in i..=r)
    assert rev.prefix(xs.len) == sum(xs[k] for k in 0..xs.len)
