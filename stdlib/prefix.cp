//! Prefix sums over 1-D and 2-D arrays.
//!
//! Build once in O(n) or O(r * c), then answer any range or rectangle sum in O(1).

/// Static 1-D prefix sums over a domain `D`, of `Int`s, `F64`s, `Mod`s
/// or anything else with `+`, `-` and a zero.
pub struct Prefix1<D, T>:
    p: [D.bounds]T

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
