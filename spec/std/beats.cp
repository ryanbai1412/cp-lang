//! Segment Tree Beats (Ji Driver) for range chmin, chmax, add, sum, min, and max.
//!
//! Maintains exact `min`, second `min`, `max`, second `max`, and `sum` on every node.
//! Range `chmin`, `chmax`, `add`, and `sum`/`min`/`max` queries run in O(log n) amortized
//! (O(log^2 n) amortized when mixing `chmin`/`chmax` with `add`).
use std.seq.{ends}

/// Segment Tree Beats over a domain `D` of `Int` values.
/// Supports `chmin`, `chmax`, `add`, `sum`, `min_val`, `max_val`, and `get`.
pub struct BeatsSegTree<D>:
    s: List<Int>
    mx1: List<Int>
    mx2: List<Int>
    mx_c: List<Int>
    mn1: List<Int>
    mn2: List<Int>
    mn_c: List<Int>
    lz_add: List<Int>
    n: Int

    /// A Segment Tree Beats over `D` holding the values of `a`, or `fill` at every position, or
    /// zero everywhere when neither is given. Every node keeps the exact maximum, second maximum
    /// and its count, the same for the minimum, and the sum.
    ///
    /// cost: O(|D|)
    /// empty: an empty `D` builds a tree whose sums are zero and whose extrema are `None`
    /// order: positions keep the domain `D` (of `a`, when given)
    /// traps: when both `a` and `fill` are given
    /// example: `BeatsSegTree([5, 1, 4]).all_sum == 10`
    /// example: `BeatsSegTree<0..3>(fill: 2).all_sum == 6`
    pub fn new(a: ?[D]Int = None, fill: ?Int = None) -> BeatsSegTree<D>:
        let n = D.K
        let sz = max(1, 4 * n)
        var st = BeatsSegTree(
            List.filled(sz, 0),
            List.filled(sz, Int.MIN),
            List.filled(sz, Int.MIN),
            List.filled(sz, 0),
            List.filled(sz, Int.MAX),
            List.filled(sz, Int.MAX),
            List.filled(sz, 0),
            List.filled(sz, 0),
            n,
        )
        if n == 0: return st
        var vals: List<Int> = []
        if let init = a:
            assert fill == None, "pass either `a:` or `fill:`, not both"
            vals = [init[i] for i in D]
        elif let v = fill:
            vals = List.filled(n, v)
        else:
            vals = List.filled(n, 0)
        st.build(1, 0, n, vals)
        st

    fn pull(mut self, u: Int):
        let (l, r) = (2 * u, 2 * u + 1)
        s[u] = s[l] + s[r]
        if mx1[l] == mx1[r]:
            mx1[u] = mx1[l]
            mx_c[u] = mx_c[l] + mx_c[r]
            mx2[u] = max(mx2[l], mx2[r])
        elif mx1[l] > mx1[r]:
            mx1[u] = mx1[l]
            mx_c[u] = mx_c[l]
            mx2[u] = max(mx2[l], mx1[r])
        else:
            mx1[u] = mx1[r]
            mx_c[u] = mx_c[r]
            mx2[u] = max(mx1[l], mx2[r])
        if mn1[l] == mn1[r]:
            mn1[u] = mn1[l]
            mn_c[u] = mn_c[l] + mn_c[r]
            mn2[u] = min(mn2[l], mn2[r])
        elif mn1[l] < mn1[r]:
            mn1[u] = mn1[l]
            mn_c[u] = mn_c[l]
            mn2[u] = min(mn2[l], mn1[r])
        else:
            mn1[u] = mn1[r]
            mn_c[u] = mn_c[r]
            mn2[u] = min(mn1[l], mn2[r])

    fn build(mut self, u: Int, nl: Int, nr: Int, vals: List<Int>):
        if nr - nl == 1:
            let v = vals[nl]
            s[u] = v
            mx1[u] = v
            mx2[u] = Int.MIN
            mx_c[u] = 1
            mn1[u] = v
            mn2[u] = Int.MAX
            mn_c[u] = 1
            return
        let mid = (nl + nr) / 2
        self.build(2 * u, nl, mid, vals)
        self.build(2 * u + 1, mid, nr, vals)
        self.pull(u)

    fn apply_add(mut self, u: Int, len: Int, x: Int):
        s[u] += x * len
        mx1[u] += x
        if mx2[u] != Int.MIN: mx2[u] += x
        mn1[u] += x
        if mn2[u] != Int.MAX: mn2[u] += x
        lz_add[u] += x

    fn apply_chmin(mut self, u: Int, x: Int):
        if x >= mx1[u]: return
        s[u] += (x - mx1[u]) * mx_c[u]
        if mn1[u] == mx1[u]: mn1[u] = x
        elif mn2[u] == mx1[u]: mn2[u] = x
        mx1[u] = x

    fn apply_chmax(mut self, u: Int, x: Int):
        if x <= mn1[u]: return
        s[u] += (x - mn1[u]) * mn_c[u]
        if mx1[u] == mn1[u]: mx1[u] = x
        elif mx2[u] == mn1[u]: mx2[u] = x
        mn1[u] = x

    fn push(mut self, u: Int, nl: Int, nr: Int):
        let mid = (nl + nr) / 2
        let (l, r) = (2 * u, 2 * u + 1)
        if lz_add[u] != 0:
            self.apply_add(l, mid - nl, lz_add[u])
            self.apply_add(r, nr - mid, lz_add[u])
            lz_add[u] = 0
        self.apply_chmin(l, mx1[u])
        self.apply_chmin(r, mx1[u])
        self.apply_chmax(l, mn1[u])
        self.apply_chmax(r, mn1[u])

    fn chmin_rec(mut self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int, x: Int):
        if qr <= nl or nr <= ql or x >= mx1[u]: return
        if ql <= nl and nr <= qr and x > mx2[u]:
            self.apply_chmin(u, x)
            return
        self.push(u, nl, nr)
        let mid = (nl + nr) / 2
        self.chmin_rec(2 * u, nl, mid, ql, qr, x)
        self.chmin_rec(2 * u + 1, mid, nr, ql, qr, x)
        self.pull(u)

    /// Replace every `a[i]` in `r` with `min(a[i], x)`. Descends only into nodes whose maximum
    /// exceeds `x` and whose second maximum does not, where the update is a tag.
    ///
    /// cost: O(log |D|) amortized; O(log^2 |D|) amortized when mixed with `add`
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var t = BeatsSegTree([5, 1, 4]); t.chmin(0..3, 3); t.all_sum == 7`
    pub fn chmin(mut self, r: Range<D>, x: Int):
        let (lo, hi) = r.ends
        if lo == hi: return
        self.chmin_rec(1, 0, n, lo, hi, x)

    fn chmax_rec(mut self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int, x: Int):
        if qr <= nl or nr <= ql or x <= mn1[u]: return
        if ql <= nl and nr <= qr and x < mn2[u]:
            self.apply_chmax(u, x)
            return
        self.push(u, nl, nr)
        let mid = (nl + nr) / 2
        self.chmax_rec(2 * u, nl, mid, ql, qr, x)
        self.chmax_rec(2 * u + 1, mid, nr, ql, qr, x)
        self.pull(u)

    /// Replace every `a[i]` in `r` with `max(a[i], x)`; the mirror of `chmin`.
    ///
    /// cost: O(log |D|) amortized; O(log^2 |D|) amortized when mixed with `add`
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var t = BeatsSegTree([5, 1, 4]); t.chmax(0..2, 2); t.all_sum == 11`
    pub fn chmax(mut self, r: Range<D>, x: Int):
        let (lo, hi) = r.ends
        if lo == hi: return
        self.chmax_rec(1, 0, n, lo, hi, x)

    fn add_rec(mut self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int, x: Int):
        if qr <= nl or nr <= ql: return
        if ql <= nl and nr <= qr:
            self.apply_add(u, nr - nl, x)
            return
        self.push(u, nl, nr)
        let mid = (nl + nr) / 2
        self.add_rec(2 * u, nl, mid, ql, qr, x)
        self.add_rec(2 * u + 1, mid, nr, ql, qr, x)
        self.pull(u)

    /// Add `x` to every `a[i]` in `r`.
    ///
    /// cost: O(log |D|)
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `D` (overflow wraps in release, traps in debug)
    /// example: `var t = BeatsSegTree([5, 1, 4]); t.add(1..3, 10); t.sum(0..3) == 30`
    pub fn add(mut self, r: Range<D>, x: Int):
        let (lo, hi) = r.ends
        if lo == hi: return
        self.add_rec(1, 0, n, lo, hi, x)

    fn sum_rec(mut self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int) -> Int:
        if qr <= nl or nr <= ql: return 0
        if ql <= nl and nr <= qr: return s[u]
        self.push(u, nl, nr)
        let mid = (nl + nr) / 2
        self.sum_rec(2 * u, nl, mid, ql, qr) + self.sum_rec(2 * u + 1, mid, nr, ql, qr)

    /// The sum of the current values in `r`. Takes `mut self` because it pushes pending tags.
    ///
    /// cost: O(log |D|)
    /// empty: zero
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `BeatsSegTree([5, 1, 4]).sum(1..3) == 5`
    /// example: `BeatsSegTree([5, 1, 4]).sum(1..1) == 0`
    pub fn sum(mut self, r: Range<D>) -> Int:
        let (lo, hi) = r.ends
        if lo == hi: return 0
        self.sum_rec(1, 0, n, lo, hi)

    fn min_rec(mut self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int) -> Int:
        if qr <= nl or nr <= ql: return Int.MAX
        if ql <= nl and nr <= qr: return mn1[u]
        self.push(u, nl, nr)
        let mid = (nl + nr) / 2
        min(self.min_rec(2 * u, nl, mid, ql, qr), self.min_rec(2 * u + 1, mid, nr, ql, qr))

    /// The least current value in `r`, or `None` when `r` is empty.
    ///
    /// cost: O(log |D|)
    /// empty: `None`
    /// ties: returns a value, so equal candidates are indistinguishable
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `BeatsSegTree([5, 1, 4]).min_val(0..3) == Some(1)`
    /// example: `BeatsSegTree([5, 1, 4]).min_val(2..2) == None`
    pub fn min_val(mut self, r: Range<D>) -> ?Int:
        let (lo, hi) = r.ends
        if lo == hi: return None
        self.min_rec(1, 0, n, lo, hi)

    fn max_rec(mut self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int) -> Int:
        if qr <= nl or nr <= ql: return Int.MIN
        if ql <= nl and nr <= qr: return mx1[u]
        self.push(u, nl, nr)
        let mid = (nl + nr) / 2
        max(self.max_rec(2 * u, nl, mid, ql, qr), self.max_rec(2 * u + 1, mid, nr, ql, qr))

    /// The greatest current value in `r`, or `None` when `r` is empty.
    ///
    /// cost: O(log |D|)
    /// empty: `None`
    /// ties: returns a value, so equal candidates are indistinguishable
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var t = BeatsSegTree([5, 1, 4]); t.chmin(0..1, 2); t.max_val(0..3) == Some(4)`
    pub fn max_val(mut self, r: Range<D>) -> ?Int:
        let (lo, hi) = r.ends
        if lo == hi: return None
        self.max_rec(1, 0, n, lo, hi)

    /// The current value at position `i`.
    ///
    /// cost: O(log |D|)
    /// traps: when `i` is outside `D`
    /// example: `BeatsSegTree([5, 1, 4]).get(2) == 4`
    pub fn get(mut self, i: D) -> Int = self.sum(i..=i)

    fn div_rec(mut self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int, d: Int, rem: Bool):
        if qr <= nl or nr <= ql: return
        if ql <= nl and nr <= qr:
            if rem and mn1[u] >= 0 and mx1[u] < d: return
            let a = if rem: -d * (mx1[u] / d) else: mx1[u] / d - mx1[u]
            let b = if rem: -d * (mn1[u] / d) else: mn1[u] / d - mn1[u]
            if a == b:
                self.apply_add(u, nr - nl, a)
                return
        self.push(u, nl, nr)
        let mid = (nl + nr) / 2
        self.div_rec(2 * u, nl, mid, ql, qr, d, rem)
        self.div_rec(2 * u + 1, mid, nr, ql, qr, d, rem)
        self.pull(u)

    /// Assign `a[i] = x`, replacing whatever earlier updates left there.
    ///
    /// cost: O(log |D|)
    /// traps: when `i` is outside `D`
    /// example: `var t = BeatsSegTree([5, 1, 4]); t.set(1, 9); t.all_max == Some(9)`
    pub fn set(mut self, i: D, x: Int):
        let cur = self.get(i)
        self.add(i..=i, x - cur)

    /// Overwrite every position in `r` with `x`.
    ///
    /// cost: O(log |D|) amortized
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var t = BeatsSegTree([5, 1, 4]); t.assign(0..2, 0); t.all_sum == 4`
    pub fn assign(mut self, r: Range<D>, x: Int):
        self.chmin(r, x)
        self.chmax(r, x)

    /// Replace every `a[i]` in `r` with `a[i] / d`, rounding toward negative infinity. A node whose
    /// values all map to the same shift becomes a single `add` tag, so each value is divided
    /// O(log V) times before it settles, for values of magnitude at most `V`.
    ///
    /// cost: O(log |D| log V) amortized
    /// empty: changes nothing
    /// traps: when `d <= 0`, or a non-empty `r` reaches outside `D`
    /// example: `var t = BeatsSegTree([5, 1, 4]); t.chdiv(0..3, 2); t.all_sum == 4`
    /// example: `var t = BeatsSegTree([-3]); t.chdiv(0..1, 2); t.get(0) == -2`
    pub fn chdiv(mut self, r: Range<D>, d: Int):
        assert d > 0, "`chdiv` needs `d > 0`"
        let (lo, hi) = r.ends
        if lo == hi: return
        self.div_rec(1, 0, n, lo, hi, d, false)

    /// Replace every `a[i]` in `r` with `a[i] mod m`, the remainder in `0..m` (floored, so
    /// negative values map into `0..m` too). Nodes whose maximum is below `m` are skipped.
    ///
    /// cost: O(log |D| log V) amortized
    /// empty: changes nothing
    /// traps: when `m <= 0`, or a non-empty `r` reaches outside `D`
    /// example: `var t = BeatsSegTree([5, 1, 4]); t.chmod(0..3, 3); t.all_sum == 4`
    pub fn chmod(mut self, r: Range<D>, m: Int):
        assert m > 0, "`chmod` needs `m > 0`"
        let (lo, hi) = r.ends
        if lo == hi: return
        self.div_rec(1, 0, n, lo, hi, m, true)

    /// The sum of every current value in `D`.
    ///
    /// cost: O(1)
    /// empty: zero on an empty domain
    /// traps: never
    /// example: `BeatsSegTree([5, 1, 4]).all_sum == 10`
    @get pub fn all_sum(self) -> Int:
        if n == 0: 0 else: s[1]

    /// The least current value in `D`, or `None` when `D` is empty.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// ties: returns a value, so equal candidates are indistinguishable
    /// traps: never
    /// example: `BeatsSegTree([5, 1, 4]).all_min == Some(1)`
    @get pub fn all_min(self) -> ?Int:
        if n == 0: None else: Some(mn1[1])

    /// The greatest current value in `D`, or `None` when `D` is empty.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// ties: returns a value, so equal candidates are indistinguishable
    /// traps: never
    /// example: `BeatsSegTree([5, 1, 4]).all_max == Some(5)`
    @get pub fn all_max(self) -> ?Int:
        if n == 0: None else: Some(mx1[1])

/// O(|D|) space. Segment Tree Beats with historic maximum and historic sum tracking under
/// range `chmin`, `chmax`, and `add` (Ji Driver with historic tags).
pub struct HistoricBeatsSegTree<D>:
    n: Int

    /// A Beats tree that also tracks history, holding `a`, or `fill` everywhere, or zeros. Each
    /// position remembers the greatest value it has held and the running sum of its values over
    /// versions; version 0 is the initial array, and every `chmin`, `chmax` or `add` call makes
    /// one new version of the whole array.
    ///
    /// cost: O(|D|)
    /// empty: an empty `D` builds a tree whose sums are zero and whose maxima are `None`
    /// order: positions keep the domain `D` (of `a`, when given)
    /// traps: when both `a` and `fill` are given
    /// example: `HistoricBeatsSegTree([1, 2]).hist_sum(0..2) == 3`
    pub fn new(a: ?[D]Int = None, fill: ?Int = None) -> HistoricBeatsSegTree<D> = todo

    /// Replace every `a[i]` in `r` with `min(a[i], x)`, then record a new version.
    ///
    /// cost: O(log |D|) amortized; O(log^2 |D|) amortized when mixed with `add`
    /// empty: changes no value but still records a version
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var t = HistoricBeatsSegTree([5, 1]); t.chmin(0..2, 2); t.sum(0..2) == 3 and t.hist_max(0..1) == Some(5)`
    pub fn chmin(mut self, r: Range<D>, x: Int) = todo

    /// Replace every `a[i]` in `r` with `max(a[i], x)`, then record a new version.
    ///
    /// cost: O(log |D|) amortized; O(log^2 |D|) amortized when mixed with `add`
    /// empty: changes no value but still records a version
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var t = HistoricBeatsSegTree([5, 1]); t.chmax(0..2, 3); t.sum(0..2) == 8`
    pub fn chmax(mut self, r: Range<D>, x: Int) = todo

    /// Add `x` to every `a[i]` in `r`, then record a new version.
    ///
    /// cost: O(log |D|)
    /// empty: changes no value but still records a version
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var t = HistoricBeatsSegTree([1, 2]); t.add(0..1, 3); t.sum(0..2) == 6`
    pub fn add(mut self, r: Range<D>, x: Int) = todo

    /// The sum of the current values in `r`.
    ///
    /// cost: O(log |D|)
    /// empty: zero
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `HistoricBeatsSegTree([1, 2]).sum(0..2) == 3`
    pub fn sum(mut self, r: Range<D>) -> Int = todo

    /// The greatest current value in `r`, or `None` when `r` is empty.
    ///
    /// cost: O(log |D|)
    /// empty: `None`
    /// ties: returns a value, so equal candidates are indistinguishable
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var t = HistoricBeatsSegTree([1, 2]); t.add(0..2, 5); t.add(0..2, -10); t.max_val(0..1) == Some(-4)`
    pub fn max_val(mut self, r: Range<D>) -> ?Int = todo

    /// The greatest value any position in `r` has held in any version, the initial one included;
    /// `None` when `r` is empty.
    ///
    /// cost: O(log |D|)
    /// empty: `None`
    /// ties: returns a value, so equal candidates are indistinguishable
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var t = HistoricBeatsSegTree([1, 2]); t.add(0..2, 5); t.add(0..2, -10); t.hist_max(0..1) == Some(6)`
    /// example: `HistoricBeatsSegTree([1, 2]).hist_max(1..1) == None`
    pub fn hist_max(mut self, r: Range<D>) -> ?Int = todo

    /// The sum over positions `i` in `r` of every value `a[i]` has had, one term per version:
    /// the initial array plus the array after each update call.
    ///
    /// cost: O(log |D|)
    /// empty: zero
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var t = HistoricBeatsSegTree([1, 2]); t.add(0..1, 3); t.hist_sum(0..2) == 9`
    /// example: `var t = HistoricBeatsSegTree([1, 2]); t.add(0..1, 3); t.hist_sum(1..2) == 4`
    pub fn hist_sum(mut self, r: Range<D>) -> Int = todo

@test
fn beats_segtree_chmin_chmax_add_matches_brute():
    domain I = 1..=16
    var ref = [I] _ => rand(-20, 20)
    var st = BeatsSegTree<I>(ref)
    for _ in 0..200:
        let (a, b) = (rand(1, 16), rand(1, 16))
        let rng = I(min(a, b))..=I(max(a, b))
        let kind = rand(0, 3)
        let x = rand(-25, 25)
        if kind == 0:
            st.chmin(rng, x)
            for i in rng: ref[i] min= x
        elif kind == 1:
            st.chmax(rng, x)
            for i in rng: ref[i] max= x
        elif kind == 2:
            st.add(rng, x)
            for i in rng: ref[i] += x
        else:
            assert st.sum(rng) == sum(ref[i] for i in rng)
            assert st.min_val(rng) == min(ref[i] for i in rng)
            assert st.max_val(rng) == max(ref[i] for i in rng)
    for i in I:
        assert st.get(i) == ref[i]
