//! Prefix sums, prefix/suffix monoid folds, and offline difference arrays.
//!
//! Build once in O(n) or O(r * c), then answer any range or rectangle sum in O(1).
use std.algebra.{Monoid}
use std.seq.{ends}

/// Static 1-D prefix sums over a domain `D`, of `Int`s, `F64`s, `Mod`s
/// or anything else with `+`, `-` and a zero.
pub struct Prefix1<D, T>:
    p: [D.bounds]T

    /// Build 1D prefix sums over `a: [D]T`. O(|D|).
    pub fn new(a: [D]T) -> Prefix1<D, T>:
        var p: [D.bounds]T = T.zero
        for i in D:
            p[i.r] = p[i.l] + a[i]
        Prefix1(p)

    /// The sum of everything left of boundary `b`.
    pub fn prefix(self, b: Bounds<D>) -> T = self.p[b]

    /// The sum of `a[i]` for `i` in `r`, or `0` when `r` is empty.
    pub fn sum(self, r: Range<D>) -> T = if r.is_empty: T.zero else: self.p[r.hi] - self.p[r.lo]

/// Static 2-D prefix sums over domains `R` and `C`.
pub struct Prefix2<R, C, T>:
    p: [R.bounds, C.bounds]T

    /// Build 2D prefix sums over `a: [R, C]T`. O(|R| * |C|).
    pub fn new(a: [R, C]T) -> Prefix2<R, C, T>:
        var p: [R.bounds, C.bounds]T = T.zero
        for r in R:
            for c in C:
                p[r.r, c.r] = p[r.l, c.r] + p[r.r, c.l] - p[r.l, c.l] + a[r, c]
        Prefix2(p)

    /// The sum of `a[r, c]` over `rows` by `cols`, or `0` when either range is empty.
    pub fn sum(self, rows: Range<R>, cols: Range<C>) -> T:
        if rows.is_empty or cols.is_empty: T.zero
        else:
            self.p[rows.hi, cols.hi] - self.p[rows.lo, cols.hi] - self.p[rows.hi, cols.lo] + self.p[
                rows.lo,
                cols.lo,
            ]

/// Static prefix sums of a list or an array indexed by plain `Int`s
/// from 0.
pub struct PrefixN<T>:
    p: List<T>

    /// Build 1D prefix sums over `a: List<T>`. O(a.len).
    pub fn new(a: List<T>) -> PrefixN<T>:
        var p = List.filled(a.len + 1, T.zero)
        for i in 0..a.len:
            p[i + 1] = p[i] + a[i]
        PrefixN(p)

    /// The sum of the first `k` values.
    pub fn prefix(self, k: Int) -> T = self.p[k]

    /// The sum of `a[i]` for `i` in `r`, or zero when `r` is empty.
    pub fn sum(self, r: Range<Int>) -> T = if r.is_empty: T.zero else: self.p[r.hi] - self.p[r.lo]

/// Prefix sums of a list.
pub fn prefix_n<T>(a: List<T>) -> PrefixN<T> = PrefixN<T>(a)

/// 1-D prefix sums of `a`.
pub fn prefix1<D, T>(a: [D]T) -> Prefix1<D, T> = Prefix1<D, T>(a)

/// 2-D prefix sums of `a`.
pub fn prefix2<R, C, T>(a: [R, C]T) -> Prefix2<R, C, T> = Prefix2<R, C, T>(a)

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
/// `m.op(pref[r.lo], suff[r.hi])`. O(n).
pub fn suffix_fold<D, T, M>(op: M, a: [D]T) -> [D.bounds]T:
    let m = Monoid<T>.of(op)
    var s: [D.bounds]T = m.id
    for i in D.rev:
        s[i.l] = m.op(a[i], s[i.r])
    s

/// Offline 1-D difference array: O(1) range add, O(n) `finish` returning `[D]T`.
pub struct Diff1<D, T>:
    d: [D.bounds]T

    /// Create an all-zero 1D difference array over `D`. O(|D|).
    pub fn new() -> Diff1<D, T> = Diff1([D.bounds] _ => T.zero)

    /// Add `x` to every position in `r`. O(1).
    pub fn add(mut self, r: Range<D>, x: T):
        if r.is_empty: return
        d[r.lo] = d[r.lo] + x
        d[r.hi] = d[r.hi] - x

    /// Materialize the accumulated values across `D`. O(n).
    pub fn finish(self) -> [D]T:
        var out: [D]T = T.zero
        var cur = T.zero
        for i in D:
            cur = cur + d[i.l]
            out[i] = cur
        out

/// Offline 2-D difference array: O(1) rectangle add, O(r * c) `finish` returning `[R, C]T`.
pub struct Diff2<R, C, T>:
    d: [R.bounds, C.bounds]T

    /// Create an all-zero 2D difference array over `(R, C)`. O(|R| * |C|).
    pub fn new() -> Diff2<R, C, T> = Diff2([R.bounds, C.bounds] (_, _) => T.zero)

    /// Add `x` to every cell in `rows` by `cols`. O(1).
    pub fn add(mut self, rows: Range<R>, cols: Range<C>, x: T):
        if rows.is_empty or cols.is_empty: return
        d[rows.lo, cols.lo] = d[rows.lo, cols.lo] + x
        d[rows.lo, cols.hi] = d[rows.lo, cols.hi] - x
        d[rows.hi, cols.lo] = d[rows.hi, cols.lo] - x
        d[rows.hi, cols.hi] = d[rows.hi, cols.hi] + x

    /// Materialize the accumulated grid across `(R, C)`. O(r * c).
    pub fn finish(self) -> [R, C]T:
        var p: [R.bounds, C.bounds]T = T.zero
        var out: [R, C]T = T.zero
        for r in R:
            for c in C:
                let v = p[r.l, c.r] + p[r.r, c.l] - p[r.l, c.l] + d[r.l, c.l]
                p[r.r, c.r] = v
                out[r, c] = v
        out

@test
fn one_d_sums_match_brute():
    domain V = 1..=9
    let a = [V] _ => rand(-10, 10)
    let pre = prefix1(a)
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
    let pre = prefix2(a)
    for r1 in R:
        for r2 in r1..=R.last:
            for c1 in C:
                for c2 in c1..=C.last:
                    let want = sum(a[r, c] for r in r1..=r2 for c in c1..=c2)
                    assert pre.sum(r1..=r2, c1..=c2) == want
    assert pre.sum(R(2)..R(2), C(1)..=C(6)) == 0
    assert pre.sum(R(0)..=R(4), C(3)..C(3)) == 0

@test
fn a_plain_range_takes_plain_ints():
    let a: [0..5]Int = [3, 1, 4, 1, 5]
    let p = prefix1(a)
    assert p.prefix(0) == 0 and p.prefix(2) == 4 and p.prefix(5) == 14
    assert p.sum(1..3) == 5 and p.sum(1..=3) == 6 and p.sum(2..2) == 0
    let b: [1..=3]Int = [10, 20, 30]
    let q = prefix1(b)
    assert q.sum(1..=3) == 60 and q.sum(2..=3) == 50 and q.prefix(1) == 0

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
            assert max(pref[range.lo], suff[range.hi]) == want

@test
fn diff1_and_diff2_match_brute():
    domain V = 1..=9
    var plain: [V]Int = 0
    var d1 = Diff1<V, Int>()
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
    var d2 = Diff2<R, C, Int>()
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
