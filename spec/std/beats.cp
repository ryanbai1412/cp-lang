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

    /// Build from `a: [D]Int` or a uniform `fill:` value (default 0). O(n).
    pub fn new(a: ?[D]Int = none, fill: ?Int = none) -> BeatsSegTree<D>:
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
            assert fill == none, "pass either `a:` or `fill:`, not both"
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

    /// Replace every `a[i]` in `r` with `min(a[i], x)`. O(log n) amortized.
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

    /// Replace every `a[i]` in `r` with `max(a[i], x)`. O(log n) amortized.
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

    /// Add `x` to every `a[i]` in `r`. O(log n).
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

    /// Sum of elements in `r`. O(log n).
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

    /// Minimum element in `r`, or `none` if `r` is empty. O(log n).
    pub fn min_val(mut self, r: Range<D>) -> ?Int:
        let (lo, hi) = r.ends
        if lo == hi: return none
        self.min_rec(1, 0, n, lo, hi)

    fn max_rec(mut self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int) -> Int:
        if qr <= nl or nr <= ql: return Int.MIN
        if ql <= nl and nr <= qr: return mx1[u]
        self.push(u, nl, nr)
        let mid = (nl + nr) / 2
        max(self.max_rec(2 * u, nl, mid, ql, qr), self.max_rec(2 * u + 1, mid, nr, ql, qr))

    /// Maximum element in `r`, or `none` if `r` is empty. O(log n).
    pub fn max_val(mut self, r: Range<D>) -> ?Int:
        let (lo, hi) = r.ends
        if lo == hi: return none
        self.max_rec(1, 0, n, lo, hi)

    /// Value at a single position `i`. O(log n).
    pub fn get(mut self, i: D) -> Int = self.sum(i..=i)

    /// O(log |D|). Assign position `i` to `x`.
    pub fn set(mut self, i: D, x: Int) = todo

    /// O(log |D|) amortized. Overwrite every position in `r` with `x`.
    pub fn assign(mut self, r: Range<D>, x: Int) = todo

    /// O(log |D| * log V) amortized. Replace every `a[i]` in `r` with `a[i] / d` (floor division, `d > 0`).
    pub fn chdiv(mut self, r: Range<D>, d: Int) = todo

    /// O(log |D| * log V) amortized. Replace every `a[i]` in `r` with `a[i] % m` (`m > 0`).
    pub fn chmod(mut self, r: Range<D>, m: Int) = todo

    /// O(1). Sum of the entire domain.
    pub @get fn all_sum(self) -> Int = todo

    /// O(1). Minimum of the entire domain, or `none` when empty.
    pub @get fn all_min(self) -> ?Int = todo

    /// O(1). Maximum of the entire domain, or `none` when empty.
    pub @get fn all_max(self) -> ?Int = todo

/// O(|D|) space. Segment Tree Beats with historic maximum and historic sum tracking under
/// range `chmin`, `chmax`, and `add` (Ji Driver with historic tags).
pub struct HistoricBeatsSegTree<D>:
    n: Int

    /// O(|D|). Build from `a: [D]Int` or uniform `fill:` (default 0).
    pub fn new(a: ?[D]Int = none, fill: ?Int = none) -> HistoricBeatsSegTree<D> = todo

    /// O(log |D|) amortized. Replace every `a[i]` in `r` with `min(a[i], x)`.
    pub fn chmin(mut self, r: Range<D>, x: Int) = todo

    /// O(log |D|) amortized. Replace every `a[i]` in `r` with `max(a[i], x)`.
    pub fn chmax(mut self, r: Range<D>, x: Int) = todo

    /// O(log |D|). Add `x` to every `a[i]` in `r`.
    pub fn add(mut self, r: Range<D>, x: Int) = todo

    /// O(log |D|). Current sum of elements in `r`.
    pub fn sum(mut self, r: Range<D>) -> Int = todo

    /// O(log |D|). Current maximum in `r`, or `none` when empty.
    pub fn max_val(mut self, r: Range<D>) -> ?Int = todo

    /// O(log |D|). Historic maximum ever attained at any position in `r`, or `none` when empty.
    pub fn hist_max(mut self, r: Range<D>) -> ?Int = todo

    /// O(log |D|). Historic sum of all versions across `r`.
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
