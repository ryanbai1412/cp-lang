//! Segment Tree Beats (Ji Driver) for range chmin, chmax, add, sum, min, and max.
//!
//! Maintains exact `min`, second `min`, `max`, second `max`, and `sum` on every node.
//! Range `chmin`, `chmax`, `add`, and `sum`/`min`/`max` queries run in \(O(\log n)\) amortized
//! (\(O(\log^2 n)\) amortized when mixing `chmin`/`chmax` with `add`).
use std.range.{ends}

// One node's aggregates as seen from above: the stored ones with every ancestor's pending tag
// applied, and `add` the pending add its children have not seen yet.
struct BeatsView:
    var s: Int
    var mx1: Int
    var mx2: Int
    var mx_c: Int
    var mn1: Int
    var mn2: Int
    var mn_c: Int
    var add: Int

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
    /// cost: \(O(\lvert D \rvert)\)
    /// empty: `BeatsSegTree<0..0>().all_sum == 0 and BeatsSegTree<0..0>().all_max == None`
    /// order: positions keep the domain `D` (of `a`, when given)
    /// traps: when both `a` and `fill` are given
    /// example: `BeatsSegTree([5, 1, 4]).all_sum == 10`
    /// example: `BeatsSegTree<0..3>(fill: 2).all_sum == 6`
    pub fn new(a: ?[D]Int = None, fill: ?Int = None) -> BeatsSegTree<D>:
        let n = D.count
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
    /// cost: \(O(\log \lvert D \rvert)\) amortized; \(O(\log^2 \lvert D \rvert)\) amortized when mixed with `add`
    /// empty: `var t = BeatsSegTree([5, 1, 4]); t.chmin(1..1, 0); t.all_sum == 10`
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
    /// cost: \(O(\log \lvert D \rvert)\) amortized; \(O(\log^2 \lvert D \rvert)\) amortized when mixed with `add`
    /// empty: `var t = BeatsSegTree([5, 1, 4]); t.chmax(1..1, 9); t.all_sum == 10`
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
    /// cost: \(O(\log \lvert D \rvert)\)
    /// empty: `var t = BeatsSegTree([5, 1, 4]); t.add(1..1, 9); t.all_sum == 10`
    /// traps: when a non-empty `r` reaches outside `D` (overflow wraps in release, traps in debug)
    /// example: `var t = BeatsSegTree([5, 1, 4]); t.add(1..3, 10); t.sum(0..3) == 30`
    pub fn add(mut self, r: Range<D>, x: Int):
        let (lo, hi) = r.ends
        if lo == hi: return
        self.add_rec(1, 0, n, lo, hi, x)

    fn root_view(self) -> BeatsView =
        BeatsView(s[1], mx1[1], mx2[1], mx_c[1], mn1[1], mn2[1], mn_c[1], lz_add[1])

    // Node `c` as a push from parent view `p` would leave it, without writing it back.
    fn seen(self, c: Int, len: Int, p: BeatsView) -> BeatsView:
        let a = p.add
        var v = BeatsView(
            s[c] + a * len,
            mx1[c] + a,
            if mx2[c] != Int.MIN: mx2[c] + a else: Int.MIN,
            mx_c[c],
            mn1[c] + a,
            if mn2[c] != Int.MAX: mn2[c] + a else: Int.MAX,
            mn_c[c],
            lz_add[c] + a,
        )
        if p.mx1 < v.mx1:
            v.s += (p.mx1 - v.mx1) * v.mx_c
            if v.mn1 == v.mx1: v.mn1 = p.mx1
            elif v.mn2 == v.mx1: v.mn2 = p.mx1
            v.mx1 = p.mx1
        if p.mn1 > v.mn1:
            v.s += (p.mn1 - v.mn1) * v.mn_c
            if v.mx1 == v.mn1: v.mx1 = p.mn1
            elif v.mx2 == v.mn1: v.mx2 = p.mn1
            v.mn1 = p.mn1
        v

    // `kind` 0 sums, 1 takes the minimum, 2 the maximum of `ql..qr` under view `v` of node `u`.
    fn read(self, u: Int, v: BeatsView, nl: Int, nr: Int, ql: Int, qr: Int, kind: Int) -> Int:
        if qr <= nl or nr <= ql: return if kind == 0: 0 elif kind == 1: Int.MAX else: Int.MIN
        if ql <= nl and nr <= qr: return if kind == 0: v.s elif kind == 1: v.mn1 else: v.mx1
        let mid = (nl + nr) / 2
        let a = self.read(2 * u, self.seen(2 * u, mid - nl, v), nl, mid, ql, qr, kind)
        let b = self.read(2 * u + 1, self.seen(2 * u + 1, nr - mid, v), mid, nr, ql, qr, kind)
        if kind == 0: a + b elif kind == 1: min(a, b) else: max(a, b)

    /// The sum of the current values in `r`. Reads pending tags on the way down instead of
    /// pushing them, so a query never changes the tree.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// empty: `BeatsSegTree([5, 1, 4]).sum(2..2) == 0`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `BeatsSegTree([5, 1, 4]).sum(1..3) == 5`
    /// example: `var t = BeatsSegTree([5, 1, 4]); t.chmin(0..3, 2); t.sum(0..2) == 3`
    /// example: `BeatsSegTree([5, 1, 4]).sum(1..1) == 0`
    pub fn sum(self, r: Range<D>) -> Int:
        let (lo, hi) = r.ends
        if lo == hi: return 0
        self.read(1, self.root_view(), 0, n, lo, hi, 0)

    /// The least current value in `r`, or `None` when `r` is empty.
    /// It returns a value, so equal candidates are indistinguishable.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// empty: `BeatsSegTree([5, 1, 4]).min_val(1..1) == None`
    /// ties: `BeatsSegTree([3, 1, 1]).min_val(0..3) == Some(1)`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `BeatsSegTree([5, 1, 4]).min_val(0..3) == Some(1)`
    /// example: `BeatsSegTree([5, 1, 4]).min_val(2..2) == None`
    pub fn min_val(self, r: Range<D>) -> ?Int:
        let (lo, hi) = r.ends
        if lo == hi: return None
        self.read(1, self.root_view(), 0, n, lo, hi, 1)

    /// The greatest current value in `r`, or `None` when `r` is empty.
    /// It returns a value, so equal candidates are indistinguishable.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// empty: `BeatsSegTree([5, 1, 4]).max_val(1..1) == None`
    /// ties: `BeatsSegTree([4, 4, 1]).max_val(0..3) == Some(4)`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var t = BeatsSegTree([5, 1, 4]); t.chmin(0..1, 2); t.max_val(0..3) == Some(4)`
    pub fn max_val(self, r: Range<D>) -> ?Int:
        let (lo, hi) = r.ends
        if lo == hi: return None
        self.read(1, self.root_view(), 0, n, lo, hi, 2)

    /// The current value at position `i`.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// traps: when `i` is outside `D`
    /// example: `BeatsSegTree([5, 1, 4]).get(2) == 4`
    pub fn get(self, i: D) -> Int = self.sum(i..=i)

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
    /// cost: \(O(\log \lvert D \rvert)\)
    /// traps: when `i` is outside `D`
    /// example: `var t = BeatsSegTree([5, 1, 4]); t.set(1, 9); t.all_max == Some(9)`
    pub fn set(mut self, i: D, x: Int):
        let cur = self.get(i)
        self.add(i..=i, x - cur)

    /// Overwrite every position in `r` with `x`.
    ///
    /// cost: \(O(\log \lvert D \rvert)\) amortized
    /// empty: `var t = BeatsSegTree([5, 1, 4]); t.assign(1..1, 0); t.all_sum == 10`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var t = BeatsSegTree([5, 1, 4]); t.assign(0..2, 0); t.all_sum == 4`
    pub fn assign(mut self, r: Range<D>, x: Int):
        self.chmin(r, x)
        self.chmax(r, x)

    /// Replace every `a[i]` in `r` with `a[i] / d`, rounding toward negative infinity.
    ///
    /// A node whose values all map to the same shift becomes a single `add` tag, so each value is
    /// divided \(O(\log V)\) times before it settles, for values of magnitude at most `V`.
    ///
    /// cost: \(O(\log \lvert D \rvert \log V)\) amortized
    /// empty: `var t = BeatsSegTree([5, 1, 4]); t.chdiv(1..1, 2); t.all_sum == 10`
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
    /// cost: \(O(\log \lvert D \rvert \log V)\) amortized
    /// empty: `var t = BeatsSegTree([5, 1, 4]); t.chmod(1..1, 3); t.all_sum == 10`
    /// traps: when `m <= 0`, or a non-empty `r` reaches outside `D`
    /// example: `var t = BeatsSegTree([5, 1, 4]); t.chmod(0..3, 3); t.all_sum == 4`
    pub fn chmod(mut self, r: Range<D>, m: Int):
        assert m > 0, "`chmod` needs `m > 0`"
        let (lo, hi) = r.ends
        if lo == hi: return
        self.div_rec(1, 0, n, lo, hi, m, true)

    /// The sum of every current value in `D`.
    ///
    /// cost: \(O(1)\)
    /// empty: `BeatsSegTree<0..0>().all_sum == 0`
    /// traps: never
    /// example: `BeatsSegTree([5, 1, 4]).all_sum == 10`
    @get pub fn all_sum(self) -> Int:
        if n == 0: 0 else: s[1]

    /// The least current value in `D`, or `None` when `D` is empty.
    /// It returns a value, so equal candidates are indistinguishable.
    ///
    /// cost: \(O(1)\)
    /// empty: `BeatsSegTree<0..0>().all_min == None`
    /// ties: `BeatsSegTree([3, 1, 1]).all_min == Some(1)`
    /// traps: never
    /// example: `BeatsSegTree([5, 1, 4]).all_min == Some(1)`
    @get pub fn all_min(self) -> ?Int:
        if n == 0: None else: Some(mn1[1])

    /// The greatest current value in `D`, or `None` when `D` is empty.
    /// It returns a value, so equal candidates are indistinguishable.
    ///
    /// cost: \(O(1)\)
    /// empty: `BeatsSegTree<0..0>().all_max == None`
    /// ties: `BeatsSegTree([5, 5, 1]).all_max == Some(5)`
    /// traps: never
    /// example: `BeatsSegTree([5, 1, 4]).all_max == Some(5)`
    @get pub fn all_max(self) -> ?Int:
        if n == 0: None else: Some(mx1[1])

/// \(O(\lvert D \rvert)\) space. Segment Tree Beats with historic maximum and historic sum tracking under
/// range `chmin`, `chmax`, and `add` (Ji Driver with historic tags).
pub struct HistoricBeatsSegTree<D>:
    n: Int
    s: List<Int> = []
    c: List<Int> = []
    hm: List<Int> = []
    mx1: List<Int> = []
    mx2: List<Int> = []
    mxc: List<Int> = []
    mn1: List<Int> = []
    mn2: List<Int> = []
    mnc: List<Int> = []
    tg: List<Int> = []
    ver: Int = 0

    /// A Beats tree that also tracks history, holding `a`, or `fill` everywhere, or zeros.
    ///
    /// Each position remembers the greatest value it has held and the running sum of its values
    /// over versions; version 0 is the initial array, and every `chmin`, `chmax` or `add` call
    /// makes one new version of the whole array.
    ///
    /// cost: \(O(\lvert D \rvert)\)
    /// empty: `var t = HistoricBeatsSegTree<0..0>(); t.hist_sum(0..0) == 0 and t.hist_max(0..0) == None`
    /// order: positions keep the domain `D` (of `a`, when given)
    /// traps: when both `a` and `fill` are given
    /// example: `var t = HistoricBeatsSegTree([1, 2]); t.hist_sum(0..2) == 3`
    pub fn new(a: ?[D]Int = None, fill: ?Int = None) -> HistoricBeatsSegTree<D>:
        let n = D.count
        let sz = max(1, 4 * n)
        var t = HistoricBeatsSegTree<D>(
            n,
            List.filled(sz, 0),
            List.filled(sz, 0),
            List.filled(sz, Int.MIN),
            List.filled(sz, Int.MIN),
            List.filled(sz, Int.MIN),
            List.filled(sz, 0),
            List.filled(sz, Int.MAX),
            List.filled(sz, Int.MAX),
            List.filled(sz, 0),
            List.filled(9 * sz, 0),
            0,
        )
        if n == 0: return t
        var vals: List<Int> = []
        if let init = a:
            assert fill == None, "pass either `a:` or `fill:`, not both"
            vals = [init[i] for i in D]
        elif let v = fill:
            vals = List.filled(n, v)
        else:
            vals = List.filled(n, 0)
        t.build(1, 0, n, vals)
        t

    fn build(mut self, u: Int, nl: Int, nr: Int, vals: List<Int>):
        if nr - nl == 1:
            let v = vals[nl]
            s[u] = v
            hm[u] = v
            mx1[u] = v
            mxc[u] = 1
            mn1[u] = v
            mnc[u] = 1
            return
        let mid = (nl + nr) / 2
        self.build(2 * u, nl, mid, vals)
        self.build(2 * u + 1, mid, nr, vals)
        self.pull(u)

    fn pull(mut self, u: Int):
        let (l, r) = (2 * u, 2 * u + 1)
        s[u] = s[l] + s[r]
        c[u] = c[l] + c[r]
        hm[u] = max(hm[l], hm[r])
        if mx1[l] == mx1[r]:
            mx1[u] = mx1[l]
            mxc[u] = mxc[l] + mxc[r]
            mx2[u] = max(mx2[l], mx2[r])
        elif mx1[l] > mx1[r]:
            mx1[u] = mx1[l]
            mxc[u] = mxc[l]
            mx2[u] = max(mx2[l], mx1[r])
        else:
            mx1[u] = mx1[r]
            mxc[u] = mxc[r]
            mx2[u] = max(mx1[l], mx2[r])
        if mn1[l] == mn1[r]:
            mn1[u] = mn1[l]
            mnc[u] = mnc[l] + mnc[r]
            mn2[u] = min(mn2[l], mn2[r])
        elif mn1[l] < mn1[r]:
            mn1[u] = mn1[l]
            mnc[u] = mnc[l]
            mn2[u] = min(mn2[l], mn1[r])
        else:
            mn1[u] = mn1[r]
            mnc[u] = mnc[r]
            mn2[u] = min(mn1[l], mn2[r])

    fn compose(mut self, u: Int, k: Int, a: Int, h: Int, cc: Int):
        let b = 9 * u + 3 * k
        tg[b + 1] max= tg[b] + h
        tg[b] += a
        tg[b + 2] += cc

    fn apply(
        mut self,
        u: Int,
        len: Int,
        a1: Int,
        h1: Int,
        c1: Int,
        a2: Int,
        h2: Int,
        c2: Int,
        a3: Int,
        h3: Int,
        c3: Int,
    ):
        if mx1[u] == mn1[u]:
            hm[u] max= mx1[u] + h1
            s[u] += a1 * len
            c[u] += c1 * len
            mx1[u] += a1
            mn1[u] = mx1[u]
            self.compose(u, 0, a1, h1, c1)
            self.compose(u, 1, a1, h1, c1)
            return
        let rest = len - mxc[u] - mnc[u]
        let two = mx2[u] == mn1[u]
        hm[u] = max(hm[u], max(mx1[u] + h1, mn1[u] + h2))
        if not two: hm[u] max= mx2[u] + h3
        s[u] += a1 * mxc[u] + a2 * mnc[u] + a3 * rest
        c[u] += c1 * mxc[u] + c2 * mnc[u] + c3 * rest
        mx1[u] += a1
        mn1[u] += a2
        if two:
            mx2[u] = mn1[u]
            mn2[u] = mx1[u]
        else:
            mx2[u] += a3
            mn2[u] += a3
        self.compose(u, 0, a1, h1, c1)
        self.compose(u, 1, a2, h2, c2)
        self.compose(u, 2, a3, h3, c3)

    fn give(mut self, v: Int, len: Int, u: Int, pmax: Int, pmin: Int):
        let b = 9 * u
        let k1 = if mx1[v] == pmax: 0 elif mx1[v] == mn1[v] and mn1[v] == pmin: 1 else: 2
        let k2 = if mx1[v] == mn1[v]: k1 elif mn1[v] == pmin: 1 else: 2
        self.apply(
            v,
            len,
            tg[b + 3 * k1],
            tg[b + 3 * k1 + 1],
            tg[b + 3 * k1 + 2],
            tg[b + 3 * k2],
            tg[b + 3 * k2 + 1],
            tg[b + 3 * k2 + 2],
            tg[b + 6],
            tg[b + 7],
            tg[b + 8],
        )

    fn push(mut self, u: Int, nl: Int, nr: Int):
        let b = 9 * u
        if not (0..9).any(k => tg[b + k] != 0): return
        let mid = (nl + nr) / 2
        let (l, r) = (2 * u, 2 * u + 1)
        let pmax = max(mx1[l], mx1[r])
        let pmin = min(mn1[l], mn1[r])
        self.give(l, mid - nl, u, pmax, pmin)
        self.give(r, nr - mid, u, pmax, pmin)
        for k in 0..9: tg[b + k] = 0

    fn upd(mut self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int, op: Int, x: Int):
        if qr <= nl or nr <= ql: return
        if op == 0 and x >= mx1[u]: return
        if op == 1 and x <= mn1[u]: return
        if ql <= nl and nr <= qr:
            let len = nr - nl
            if op == 2:
                self.apply(
                    u,
                    len,
                    x,
                    max(0, x),
                    -ver * x,
                    x,
                    max(0, x),
                    -ver * x,
                    x,
                    max(0, x),
                    -ver * x,
                )
                return
            if op == 0 and x > mx2[u]:
                let d = x - mx1[u]
                if mx1[u] == mn1[u]: self.apply(u, len, d, 0, -ver * d, d, 0, -ver * d, 0, 0, 0)
                else: self.apply(u, len, d, 0, -ver * d, 0, 0, 0, 0, 0, 0)
                return
            if op == 1 and x < mn2[u]:
                let d = x - mn1[u]
                if mx1[u] == mn1[u]: self.apply(u, len, d, d, -ver * d, d, d, -ver * d, 0, 0, 0)
                else: self.apply(u, len, 0, 0, 0, d, d, -ver * d, 0, 0, 0)
                return
        self.push(u, nl, nr)
        let mid = (nl + nr) / 2
        self.upd(2 * u, nl, mid, ql, qr, op, x)
        self.upd(2 * u + 1, mid, nr, ql, qr, op, x)
        self.pull(u)

    fn update(mut self, r: Range<D>, op: Int, x: Int):
        ver += 1
        let (lo, hi) = r.ends
        if lo < hi: self.upd(1, 0, n, lo, hi, op, x)

    /// Replace every `a[i]` in `r` with `min(a[i], x)`, then record a new version.
    /// An empty `r` changes no value but still records a version.
    ///
    /// cost: \(O(\log \lvert D \rvert)\) amortized; \(O(\log^2 \lvert D \rvert)\) amortized when mixed with `add`
    /// empty: `var t = HistoricBeatsSegTree([1, 2]); t.chmin(1..1, 0); t.sum(0..2) == 3 and t.hist_sum(0..2) == 6`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var t = HistoricBeatsSegTree([5, 1]); t.chmin(0..2, 2); t.sum(0..2) == 3 and t.hist_max(0..1) == Some(5)`
    pub fn chmin(mut self, r: Range<D>, x: Int) = self.update(r, 0, x)
    /// Replace every `a[i]` in `r` with `max(a[i], x)`, then record a new version.
    /// An empty `r` changes no value but still records a version.
    ///
    /// cost: \(O(\log \lvert D \rvert)\) amortized; \(O(\log^2 \lvert D \rvert)\) amortized when mixed with `add`
    /// empty: `var t = HistoricBeatsSegTree([1, 2]); t.chmax(1..1, 9); t.sum(0..2) == 3 and t.hist_sum(0..2) == 6`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var t = HistoricBeatsSegTree([5, 1]); t.chmax(0..2, 3); t.sum(0..2) == 8`
    pub fn chmax(mut self, r: Range<D>, x: Int) = self.update(r, 1, x)
    /// Add `x` to every `a[i]` in `r`, then record a new version.
    /// An empty `r` changes no value but still records a version.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// empty: `var t = HistoricBeatsSegTree([1, 2]); t.add(1..1, 9); t.sum(0..2) == 3 and t.hist_sum(0..2) == 6`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var t = HistoricBeatsSegTree([1, 2]); t.add(0..1, 3); t.sum(0..2) == 6`
    pub fn add(mut self, r: Range<D>, x: Int) = self.update(r, 2, x)

    fn ask(mut self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int, what: Int) -> Int:
        if qr <= nl or nr <= ql: return if what == 2: Int.MIN elif what == 3: Int.MIN else: 0
        if ql <= nl and nr <= qr:
            return if what == 0: s[u] elif what == 1: c[u] elif what == 2: mx1[u] else: hm[u]
        self.push(u, nl, nr)
        let mid = (nl + nr) / 2
        let x = self.ask(2 * u, nl, mid, ql, qr, what)
        let y = self.ask(2 * u + 1, mid, nr, ql, qr, what)
        if what <= 1: x + y else: max(x, y)

    fn query(mut self, r: Range<D>, what: Int) -> ?Int:
        let (lo, hi) = r.ends
        if lo == hi: return None
        Some(self.ask(1, 0, n, lo, hi, what))

    /// The sum of the current values in `r`.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// empty: `var t = HistoricBeatsSegTree([1, 2]); t.sum(1..1) == 0`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var t = HistoricBeatsSegTree([1, 2]); t.sum(0..2) == 3`
    pub fn sum(mut self, r: Range<D>) -> Int = self.query(r, 0) ?? 0
    /// The greatest current value in `r`, or `None` when `r` is empty.
    /// It returns a value, so equal candidates are indistinguishable.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// empty: `var t = HistoricBeatsSegTree([1, 2]); t.max_val(1..1) == None`
    /// ties: `var t = HistoricBeatsSegTree([2, 2]); t.max_val(0..2) == Some(2)`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var t = HistoricBeatsSegTree([1, 2]); t.add(0..2, 5); t.add(0..2, -10); t.max_val(0..1) == Some(-4)`
    pub fn max_val(mut self, r: Range<D>) -> ?Int = self.query(r, 2)
    /// The greatest value any position in `r` has held in any version, the initial one included;
    /// `None` when `r` is empty.
    /// It returns a value, so equal candidates are indistinguishable.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// empty: `var t = HistoricBeatsSegTree([1, 2]); t.hist_max(2..2) == None`
    /// ties: `var t = HistoricBeatsSegTree([2, 2]); t.hist_max(0..2) == Some(2)`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var t = HistoricBeatsSegTree([1, 2]); t.add(0..2, 5); t.add(0..2, -10); t.hist_max(0..1) == Some(6)`
    /// example: `var t = HistoricBeatsSegTree([1, 2]); t.hist_max(1..1) == None`
    pub fn hist_max(mut self, r: Range<D>) -> ?Int = self.query(r, 3)
    /// The sum over positions `i` in `r` of every value `a[i]` has had, one term per version:
    /// the initial array plus the array after each update call.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// empty: `var t = HistoricBeatsSegTree([1, 2]); t.add(0..1, 3); t.hist_sum(1..1) == 0`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var t = HistoricBeatsSegTree([1, 2]); t.add(0..1, 3); t.hist_sum(0..2) == 9`
    /// example: `var t = HistoricBeatsSegTree([1, 2]); t.add(0..1, 3); t.hist_sum(1..2) == 4`
    pub fn hist_sum(
        mut self,
        r: Range<D>,
    ) -> Int = (self.query(r, 1) ?? 0) + (ver + 1) * self.sum(r)

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
            assert st.min_val(rng) == [ref[i] for i in rng].min()!
            assert st.max_val(rng) == [ref[i] for i in rng].max()!
    for i in I:
        assert st.get(i) == ref[i]
