//! Fenwick tree (binary indexed tree) over a domain: point add, range sum.
use std.seq.{ends}

/// Point add and range sum over anything with `+` and `-` (`Int`, `Mod<P>`).
/// `Fenwick<D, Int>()` starts at zero and `Fenwick<D, Int>(a)` from an array, in O(n).
/// `add`, `prefix` and `sum` are O(log n); memory is n + 1 values.
/// A range is one prefix minus another, so there is no "Fenwick on max": that is a `SegTree`.
pub struct Fenwick<D, T>:
    t: List<T>

    pub fn new(a: ?[D]T = none) -> Fenwick<D, T>:
        var t = List.filled(D.K + 1, T.zero)
        if let init = a:
            for i in D:
                let p = i.to_idx + 1
                t[p] = t[p] + init[i]
                let q = p + (p & -p)
                if q <= D.K: t[q] = t[q] + t[p]
        Fenwick(t)

    /// `a[i] += x`.
    pub fn add(mut self, i: D, x: T):
        var p = i.to_idx + 1
        while p < t.len:
            t[p] = t[p] + x
            p += p & -p

    /// The sum of everything left of the boundary `b`: `prefix(v.r)` includes `v`, `prefix(v.l)` does not.
    pub fn prefix(self, b: Bounds<D>) -> T:
        var p = b.ord
        var acc = T.zero
        while p > 0:
            acc = acc + t[p]
            p -= p & -p
        acc

    /// The sum of a range: `f.sum(l..=r)`, `f.sum(..i)`, `f.sum(i..)`. An empty range is zero.
    pub fn sum(self, r: Range<D>) -> T:
        let (lo, hi) = r.ends
        if lo == hi: T.zero else: self.prefix(r.hi) - self.prefix(r.lo)

    /// The first boundary where the prefix sum reaches `target`, for non-negative values:
    /// the smallest `b` with `prefix(b) >= target`, or `none`. O(log n).
    pub fn lower_bound(self, target: Int) -> ?Bounds<D>:
        if target <= 0: return D.bounds.nth(0)
        var p = 0
        var rest = target
        var step = bit_ceil(D.K + 1) / 2
        while step > 0:
            if p + step <= D.K:
                // The walk is a binary search, so it is wrong once a block sums below zero.
                assert t[p + step] >= 0, "`lower_bound` and `kth` need non-negative values"
                if t[p + step] < rest:
                    p += step
                    rest -= t[p]
            step /= 2
        if p == D.K: none else: D.bounds.nth(p + 1)

    /// The `k`-th position, counting from 1, when each position stands for as many items as
    /// its value: the first `d` with `prefix` through `d` at least `k`, or `none` when there
    /// are fewer than `k` items. For non-negative values. O(log n).
    pub fn kth(self, k: Int) -> ?D:
        assert k >= 1, "`kth` counts from 1"
        let b = self.lower_bound(k)
        if b == none: none else: b!.left

/// Point add and rectangle sum on a grid. `add` and `sum` are O(log R * log C);
/// memory is (R + 1) * (C + 1) values.
pub struct Fenwick2D<R, C, T>:
    t: List<T>

    pub fn new() -> Fenwick2D<R, C, T> = Fenwick2D(List.filled((R.K + 1) * (C.K + 1), T.zero))

    /// `a[r, c] += x`.
    pub fn add(mut self, r: R, c: C, x: T):
        var p = r.to_idx + 1
        while p <= R.K:
            var q = c.to_idx + 1
            while q <= C.K:
                t[p * (C.K + 1) + q] = t[p * (C.K + 1) + q] + x
                q += q & -q
            p += p & -p

    fn corner(self, r: Int, c: Int) -> T:
        var acc = T.zero
        var p = r
        while p > 0:
            var q = c
            while q > 0:
                acc = acc + t[p * (C.K + 1) + q]
                q -= q & -q
            p -= p & -p
        acc

    /// The sum of the rectangle `rows` by `cols`.
    pub fn sum(self, rows: Range<R>, cols: Range<C>) -> T:
        let ((r1, r2), (c1, c2)) = (rows.ends, cols.ends)
        if r1 == r2 or c1 == c2: return T.zero
        self.corner(r2, c2) - self.corner(r1, c2) - self.corner(r2, c1) + self.corner(r1, c1)

@test
fn sums_match_a_plain_array():
    domain V = 1..=9
    var a = [V] v => rand(-5, 5)
    var f = Fenwick<V, Int>(a)
    for _ in 0..60:
        let i = V.nth(rand(0, 8))
        let x = rand(-9, 9)
        a[i] += x
        f.add(i, x)
        let l = V.nth(rand(0, 8))
        let r = V.nth(rand(l.to_idx, 8))
        assert f.sum(l..=r) == sum(a[v] for v in l..=r)
        assert f.sum(..l) == sum(a[v] for v in V if v < l)
        assert f.sum(l..) == sum(a[v] for v in V if v >= l)
    assert f.prefix(V.first.l) == 0

@test
fn empty_and_half_open_ranges():
    domain V = 0..5
    var f = Fenwick<V, Int>()
    f.add(V(1), 6)
    f.add(V(3), 5)
    assert f.sum(V(0)..=V(4)) == 11
    assert f.sum(V(1)..V(3)) == 6
    assert f.sum(V(2)..V(2)) == 0

@test
fn lower_bound_finds_the_kth():
    domain V = 0..8
    var f = Fenwick<V, Int>()
    for v in [1, 4, 4, 6]: f.add(V(v), 1)
    assert f.lower_bound(1)!.left == V(1)
    assert f.lower_bound(3)!.left == V(4)
    assert f.lower_bound(4)!.left == V(6)
    assert f.lower_bound(5) == none
    assert f.kth(1) == V(1)
    assert f.kth(3) == V(4)
    assert f.kth(4) == V(6)
    assert f.kth(5) == none

@test
fn two_dimensions_match_a_plain_grid():
    domain R = 0..5
    domain C = 1..=6
    var g: [R, C]Int = 0
    var f = Fenwick2D<R, C, Int>()
    for _ in 0..80:
        let (r, c) = (R.nth(rand(0, 4)), C.nth(rand(0, 5)))
        let x = rand(-9, 9)
        g[r, c] += x
        f.add(r, c, x)
        let (r1, c1) = (R.nth(rand(0, 4)), C.nth(rand(0, 5)))
        let (r2, c2) = (R.nth(rand(r1.to_idx, 4)), C.nth(rand(c1.to_idx, 5)))
        assert f.sum(r1..=r2, c1..=c2) == sum(g[i, j] for i in r1..=r2 for j in c1..=c2)
    assert f.sum(R(2)..R(2), C(1)..=C(6)) == 0
