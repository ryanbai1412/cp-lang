//! Persistent segment tree and static range k-th smallest queries.
//!
//! Each update copies only the O(log n) nodes on the root-to-leaf path and returns a
//! new `Version` handle. All earlier versions remain readable in O(log n).
use std.algebra.{Monoid}
use std.seq.{deduped, ends}

/// Opaque version handle for `PersistentSegTree`.
pub struct Version(root: Int)

/// Path-copying persistent segment tree over a domain `D` and monoid `Monoid<T>`.
/// `set` and `update` return a new `Version` in O(log n) time and memory; `fold` and `get` take O(log n).
pub struct PersistentSegTree<D, T>(m: Monoid<T>):
    left: List<0..=2_000_000_000>
    right: List<0..=2_000_000_000>
    val: List<T>
    base_root: Int

    /// Build version 0 from `a:`, a uniform `fill:`, or the monoid identity. O(n) when `a:` or `fill:` is given, O(1) otherwise.
    pub fn new<M>(op: M, a: ?[D]T = none, fill: ?T = none) -> PersistentSegTree<D, T>:
        let m = Monoid<T>.of(op)
        var pst = PersistentSegTree(m, [0], [0], [m.id], 0)
        if let init = a:
            assert fill == none, "pass either `a:` or `fill:`, not both"
            if D.K > 0:
                let vals = [init[i] for i in D]
                pst.base_root = pst.build_list(vals, 0, D.K)
        elif let v = fill:
            if D.K > 0:
                let vals = List.filled(D.K, v)
                pst.base_root = pst.build_list(vals, 0, D.K)
        pst

    fn push_node(mut self, l: Int, r: Int, v: T) -> Int:
        let id = val.len
        left.push(l)
        right.push(r)
        val.push(v)
        id

    fn build_list(mut self, vals: List<T>, nl: Int, nr: Int) -> Int:
        if nr - nl == 1: return self.push_node(0, 0, vals[nl])
        let mid = (nl + nr) / 2
        let lc = self.build_list(vals, nl, mid)
        let rc = self.build_list(vals, mid, nr)
        self.push_node(lc, rc, m.op(val[lc], val[rc]))

    /// The initial version created by `new`. O(1).
    pub @get fn base(self) -> Version = Version(base_root)

    fn set_rec(mut self, u: Int, nl: Int, nr: Int, idx: Int, x: T) -> Int:
        if nr - nl == 1: return self.push_node(0, 0, x)
        let mid = (nl + nr) / 2
        if idx < mid:
            let lc = self.set_rec(left[u], nl, mid, idx, x)
            let rc = right[u]
            self.push_node(lc, rc, m.op(val[lc], val[rc]))
        else:
            let lc = left[u]
            let rc = self.set_rec(right[u], mid, nr, idx, x)
            self.push_node(lc, rc, m.op(val[lc], val[rc]))

    /// Return a new `Version` equal to `v` with position `i` set to `x`. O(log n) time and memory.
    pub fn set(mut self, v: Version, i: D, x: T) -> Version:
        Version(self.set_rec(v.root, 0, D.K, i.to_idx, x))

    fn update_rec(mut self, u: Int, nl: Int, nr: Int, idx: Int, f: T -> T) -> Int:
        if nr - nl == 1: return self.push_node(0, 0, f(val[u]))
        let mid = (nl + nr) / 2
        if idx < mid:
            let lc = self.update_rec(left[u], nl, mid, idx, f)
            let rc = right[u]
            self.push_node(lc, rc, m.op(val[lc], val[rc]))
        else:
            let lc = left[u]
            let rc = self.update_rec(right[u], mid, nr, idx, f)
            self.push_node(lc, rc, m.op(val[lc], val[rc]))

    /// Return a new `Version` equal to `v` with position `i` replaced by `f(current)`. O(log n) time and memory.
    pub fn update(mut self, v: Version, i: D, f: T -> T) -> Version:
        Version(self.update_rec(v.root, 0, D.K, i.to_idx, f))

    /// Element at position `i` in version `v`. O(log n).
    pub fn get(self, v: Version, i: D) -> T:
        var (u, nl, nr, idx) = (v.root, 0, D.K, i.to_idx)
        while u != 0 and nr - nl > 1:
            let mid = (nl + nr) / 2
            if idx < mid:
                u = left[u]
                nr = mid
            else:
                u = right[u]
                nl = mid
        val[u]

    fn fold_rec(self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int) -> T:
        if u == 0 or qr <= nl or nr <= ql: return m.id
        if ql <= nl and nr <= qr: return val[u]
        let mid = (nl + nr) / 2
        m.op(self.fold_rec(left[u], nl, mid, ql, qr), self.fold_rec(right[u], mid, nr, ql, qr))

    /// Fold `r` in version `v`. O(log n).
    pub fn fold(self, v: Version, r: Range<D>) -> T:
        let (lo, hi) = r.ends
        if lo == hi: return m.id
        self.fold_rec(v.root, 0, D.K, lo, hi)

    /// Fold the entire domain in version `v`. O(1).
    pub fn all(self, v: Version) -> T = val[v.root]

/// Static range k-th smallest and range rank queries over `[D]T`.
/// Built in O(n log n) using prefix versions of a persistent count tree; queries run in O(log n).
pub struct RangeKth<D, T>:
    vals: List<T>
    left: List<0..=2_000_000_000>
    right: List<0..=2_000_000_000>
    cnt: List<0..=2_000_000_000>
    roots: List<0..=2_000_000_000>

    /// Build from `a: [D]T`. O(n log n) time and memory.
    pub fn new(a: [D]T) -> RangeKth<D, T>:
        let vals = deduped(sorted([a[i] for i in D]))
        let m = vals.len
        var rk = RangeKth(vals, [0], [0], [0], [0])
        if m == 0:
            for _ in 0..D.K: rk.roots.push(0)
            return rk
        for i in D:
            let prev = rk.roots[rk.roots.len - 1]
            let pos = rk.lower_val(a[i])
            let nxt = rk.inc(prev, m, pos)
            rk.roots.push(nxt)
        rk

    fn lower_val(self, x: T) -> Int:
        var (lo, hi) = (0, vals.len)
        unchecked:
            while lo < hi:
                let mid = (lo + hi) / 2
                if vals[mid] < x: lo = mid + 1
                else: hi = mid
        lo

    fn upper_val(self, x: T) -> Int:
        var (lo, hi) = (0, vals.len)
        unchecked:
            while lo < hi:
                let mid = (lo + hi) / 2
                if vals[mid] <= x: lo = mid + 1
                else: hi = mid
        lo

    fn inc(mut self, root: Int, m: Int, pos: Int) -> Int:
        unchecked:
            let new_root = cnt.len
            var (u, nl, nr) = (root, 0, m)
            while true:
                let id = cnt.len
                let lu = left[u]
                let ru = right[u]
                left.push(lu)
                right.push(ru)
                cnt.push(cnt[u] + 1)
                if nr - nl == 1: break
                let mid = (nl + nr) / 2
                if pos < mid:
                    left[id] = id + 1
                    u = lu
                    nr = mid
                else:
                    right[id] = id + 1
                    u = ru
                    nl = mid
            return new_root

    /// The `k`-th smallest element (0-based) in `r`. Traps if `k < 0` or `k >= r.len`. O(log n).
    pub fn kth(self, r: Range<D>, k: Int) -> T:
        let (lo, hi) = r.ends
        assert 0 <= k and k < hi - lo, "RangeKth.kth index out of range"
        unchecked:
            var (u, v, nl, nr, rem) = (roots[lo], roots[hi], 0, vals.len, k)
            while nr - nl > 1:
                let mid = (nl + nr) / 2
                let left_cnt = cnt[left[v]] - cnt[left[u]]
                if rem < left_cnt:
                    u = left[u]
                    v = left[v]
                    nr = mid
                else:
                    rem -= left_cnt
                    u = right[u]
                    v = right[v]
                    nl = mid
            return vals[nl]

    fn count_prefix(self, u: Int, v: Int, nl: Int, nr: Int, limit: Int) -> Int:
        if limit <= nl or (u == 0 and v == 0): return 0
        if nr <= limit: return cnt[v] - cnt[u]
        let mid = (nl + nr) / 2
        let lc = self.count_prefix(left[u], left[v], nl, mid, limit)
        let rc = self.count_prefix(right[u], right[v], mid, nr, limit)
        lc + rc

    /// Count elements in `r` less than or equal to `x`. O(log n).
    pub fn count_le(self, r: Range<D>, x: T) -> Int:
        let (lo, hi) = r.ends
        if lo == hi or vals.is_empty: return 0
        let pos = self.upper_val(x)
        self.count_prefix(roots[lo], roots[hi], 0, vals.len, pos)

@test
fn persistent_segtree_and_range_kth_match_brute():
    domain I = 1..=10
    let a = [I] _ => rand(-15, 15)
    var pst = PersistentSegTree<I, Int>(+, a)
    var versions: List<Version> = [pst.base]
    var snaps: List<[I]Int> = [a]
    for _ in 0..30:
        let base_id = rand(0, versions.len - 1)
        let i = I.nth(rand(0, 9))
        let x = rand(-20, 20)
        var next_arr = snaps[base_id]
        if rand(0, 1) == 0:
            versions.push(pst.set(versions[base_id], i, x))
            next_arr[i] = x
        else:
            versions.push(pst.update(versions[base_id], i, cur => cur + x))
            next_arr[i] += x
        snaps.push(next_arr)
    for vid in 0..versions.len:
        let v = versions[vid]
        let arr = snaps[vid]
        for l in 1..=10:
            for r in l..=10:
                assert pst.fold(v, I(l)..=I(r)) == sum(arr[i] for i in I(l)..=I(r))
        assert pst.get(v, I(3)) == arr[I(3)]
        assert pst.all(v) == sum(arr[i] for i in I)

    let rk = RangeKth<I, Int>(a)
    for l in 1..=10:
        for r in l..=10:
            let rng = I(l)..=I(r)
            let s = sorted([a[i] for i in rng])
            for k in 0..s.len:
                assert rk.kth(rng, k) == s[k]
            for x in [-20, -5, 0, 5, 20]:
                assert rk.count_le(rng, x) == count(y <= x for y in s)
