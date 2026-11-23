//! Persistent segment tree and static range k-th smallest queries.
//!
//! Each update copies only the \(O(\log n)\) nodes on the root-to-leaf path and returns a
//! new `Version` handle. All earlier versions remain readable in \(O(\log n)\).
use std.algebra.{Action, Monoid}
use std.seq.{deduped, ends}

/// Opaque version handle for `PersistentSegTree`.
pub struct Version(root: Int)

/// Path-copying persistent segment tree over a domain `D` and monoid `Monoid<T>`.
/// `set` and `update` return a new `Version` in \(O(\log n)\) time and memory; `fold` and `get` take \(O(\log n)\).
pub struct PersistentSegTree<D, T: Sub>(m: Monoid<T>):
    left: List<0..=2_000_000_000>
    right: List<0..=2_000_000_000>
    val: List<T>
    var base_root: Int

    /// Build version 0 over `D` from `a:`, a uniform `fill:`, or the monoid identity everywhere, and
    /// keep it as `base`. Every later write returns a new `Version`; no version is ever changed.
    ///
    /// cost: \(O(n)\) time and memory with `a:` or `fill:`, \(O(1)\) otherwise, for n = `|D|`
    /// empty: an empty `D` gives a tree whose every fold is the identity
    /// traps: when both `a:` and `fill:` are given
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); t.all(t.base) == 6`
    pub fn new<M>(op: M, a: ?[D]T = None, fill: ?T = None) -> PersistentSegTree<D, T>:
        let m = Monoid<T>.of(op)
        var pst = PersistentSegTree(m, [0], [0], [m.id], 0)
        if let init = a:
            assert fill == None, "pass either `a:` or `fill:`, not both"
            if D.count > 0:
                let vals = [init[i] for i in D]
                pst.base_root = pst.build_list(vals, 0, D.count)
        elif let v = fill:
            if D.count > 0:
                let vals = List.filled(D.count, v)
                pst.base_root = pst.build_list(vals, 0, D.count)
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

    /// The version built by `new`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); t.get(t.base, 0) == 1`
    @get pub fn base(self) -> Version:
        Version(base_root)

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

    /// A new version equal to `v` with position `i` set to `x`. `v` and every other version are
    /// unchanged; the new version shares all but the \(O(\log n)\) copied nodes with `v`.
    ///
    /// cost: \(O(\log n)\) time and memory
    /// traps: when `i` is not in `D`, or `v` did not come from this tree
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); let v = t.set(t.base, 1, 10); t.all(v) == 14 and t.all(t.base) == 6`
    pub fn set(mut self, v: Version, i: D, x: T) -> Version:
        Version(self.set_rec(v.root, 0, D.count, i.ord, x))

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

    /// A new version equal to `v` with position `i` replaced by `f` of its value in `v`. `f` is called
    /// once.
    ///
    /// cost: \(O(\log n)\) time and memory, one call of `f`
    /// traps: when `i` is not in `D`, or `v` did not come from this tree
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); let v = t.update(t.base, 2, x => x * 5); t.get(v, 2) == 15 and t.get(t.base, 2) == 3`
    pub fn update(mut self, v: Version, i: D, f: T -> T) -> Version:
        Version(self.update_rec(v.root, 0, D.count, i.ord, f))

    /// The value at position `i` in version `v`.
    ///
    /// cost: \(O(\log n)\)
    /// traps: when `i` is not in `D`, or `v` did not come from this tree
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); t.get(t.base, 1) == 2`
    pub fn get(self, v: Version, i: D) -> T:
        var (u, nl, nr, idx) = (v.root, 0, D.count, i.ord)
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

    /// The fold of `r` in version `v`, left to right, so a non-commutative monoid sees its elements in
    /// position order.
    ///
    /// cost: \(O(\log n)\)
    /// empty: the monoid identity
    /// traps: when a non-empty `r` reaches outside `D`, or `v` did not come from this tree
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); t.fold(t.base, 1..3) == 5`
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); t.fold(t.base, 2..2) == 0`
    pub fn fold(self, v: Version, r: Range<D>) -> T:
        let (lo, hi) = r.ends
        if lo == hi: return m.id
        self.fold_rec(v.root, 0, D.count, lo, hi)

    /// The fold of every position in version `v`, read from its root.
    ///
    /// cost: \(O(1)\)
    /// empty: the monoid identity when `D` is empty
    /// traps: when `v` did not come from this tree
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); let v = t.set(t.base, 0, 0); t.all(v) == 5`
    pub fn all(self, v: Version) -> T = val[v.root]

    /// Preallocate room for `nodes` tree nodes in total, so later writes do not regrow the arena.
    /// Changes no version.
    ///
    /// cost: \(O(\mathit{nodes})\)
    /// traps: when `nodes < 0`
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); t.reserve(100); t.all(t.base) == 6`
    pub fn reserve(mut self, nodes: Int):
        assert nodes >= 0, "reserve requires nodes >= 0"
        left.reserve(nodes)
        right.reserve(nodes)
        val.reserve(nodes)

    /// The rightmost cut `b` at or after `from` such that `ok(fold(v, from..b))` holds, reading
    /// version `v`.
    ///
    /// Law: `ok` is monotone (once false, false for every longer range) and `ok(identity)` holds.
    ///
    /// cost: \(O(\log n)\) calls of `ok`
    /// ties: the rightmost qualifying cut
    /// traps: when `from` is not a cut of `D`, when `ok(identity)` is false, or `v` did not come from this tree
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); t.right_while(t.base, 0, s => s <= 3) == 2`
    pub fn right_while(self, v: Version, from: Cut<D>, ok: T -> Bool) -> Cut<D>:
        assert ok(m.id), "`right_while` needs `ok` true on the identity"
        let (p, _, _) = self.rw(v.root, 0, D.count, from.ord, ok, m.id)
        D.cuts.nth(max(p, from.ord))

    fn rw(self, u: Int, nl: Int, nr: Int, l: Int, ok: T -> Bool, acc: T) -> (Int, T, Bool):
        if nr <= l or u == 0: return (nr, acc, false)
        if l <= nl:
            let c = m.op(acc, val[u])
            if ok(c): return (nr, c, false)
            if nr - nl == 1: return (nl, acc, true)
        let mid = (nl + nr) / 2
        let (p, a, stop) = self.rw(left[u], nl, mid, l, ok, acc)
        if stop: return (p, a, true)
        self.rw(right[u], mid, nr, l, ok, a)

    /// The leftmost cut `b` at or before `to` such that `ok(fold(v, b..to))` holds, reading version
    /// `v`.
    ///
    /// Law: `ok` is monotone (once false, false for every longer range) and `ok(identity)` holds.
    ///
    /// cost: \(O(\log n)\) calls of `ok`
    /// ties: the leftmost qualifying cut
    /// traps: when `to` is not a cut of `D`, when `ok(identity)` is false, or `v` did not come from this tree
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); t.left_while(t.base, 3, s => s <= 5) == 1`
    pub fn left_while(self, v: Version, to: Cut<D>, ok: T -> Bool) -> Cut<D>:
        assert ok(m.id), "`left_while` needs `ok` true on the identity"
        let (p, _, _) = self.lw(v.root, 0, D.count, to.ord, ok, m.id)
        D.cuts.nth(min(p, to.ord))

    fn lw(self, u: Int, nl: Int, nr: Int, h: Int, ok: T -> Bool, acc: T) -> (Int, T, Bool):
        if h <= nl or u == 0: return (nl, acc, false)
        if nr <= h:
            let c = m.op(val[u], acc)
            if ok(c): return (nl, c, false)
            if nr - nl == 1: return (nr, acc, true)
        let mid = (nl + nr) / 2
        let (p, a, stop) = self.lw(right[u], mid, nr, h, ok, acc)
        if stop: return (p, a, true)
        self.lw(left[u], nl, mid, h, ok, a)

    /// For a tree of counts under `+`, the position of the `k`-th item (1-based) in the difference
    /// `hi - lo` (0-based, like `RangeKth.kth`), walking both versions at once; `None` when the difference holds at most `k` items.
    /// A position with count `c` holds `c` items.
    ///
    /// Law: `hi` was derived from `lo` by adding counts, so no position of `hi - lo` is negative.
    ///
    /// cost: \(O(\log n)\)
    /// ties: items at one position are interchangeable; the position is returned
    /// traps: when `k < 0`, or a version did not come from this tree
    /// example: `var t = PersistentSegTree(+, a: [0, 0, 0, 0]); let v1 = t.set(t.base, 1, 1); let v2 = t.set(v1, 3, 2); t.diff_kth(v2, t.base, 1) == Some(3) and t.diff_kth(v2, t.base, 3) == None`
    /// example: `var t = PersistentSegTree(+, a: [0, 0, 0, 0]); let v1 = t.set(t.base, 1, 1); let v2 = t.set(v1, 3, 2); t.diff_kth(v2, v1, 0) == Some(3)`
    pub fn diff_kth(self, hi: Version, lo: Version, k: Int) -> ?D:
        assert k >= 0, "`diff_kth` needs `k >= 0`"
        var (a, b, nl, nr, rem) = (hi.root, lo.root, 0, D.count, k)
        if D.count == 0 or val[a] - val[b] <= rem: return None
        while nr - nl > 1:
            let mid = (nl + nr) / 2
            let c = val[left[a]] - val[left[b]]
            if rem < c:
                a = left[a]
                b = left[b]
                nr = mid
            else:
                rem -= c
                a = right[a]
                b = right[b]
                nl = mid
        Some(D.nth(nl))

/// Static range k-th smallest and range rank queries over `[D]T`.
/// Built in \(O(n \log n)\) using prefix versions of a persistent count tree; queries run in \(O(\log n)\).
pub struct RangeKth<D, T>:
    vals: List<T>
    left: List<0..=2_000_000_000>
    right: List<0..=2_000_000_000>
    cnt: List<0..=2_000_000_000>
    roots: List<0..=2_000_000_000>

    /// Build the static structure over `a`: one persistent count tree over the sorted distinct values
    /// per prefix of `a`. `a` is copied; later changes to it are not seen.
    ///
    /// cost: \(O(n \log n)\) time and memory for n = `a.len`
    /// empty: an empty `a` builds; every query range is then empty
    /// order: queries take ranges of `a`'s domain
    /// traps: never
    /// example: `RangeKth([5, 1, 4, 2]).kth(0..4, 0) == 1`
    pub fn new(a: [D]T) -> RangeKth<D, T>:
        let vals = deduped(sorted([a[i] for i in D]))
        let m = vals.len
        var rk = RangeKth(vals, [0], [0], [0], [0])
        if m == 0:
            for _ in 0..D.count: rk.roots.push(0)
            return rk
        let nodes = 1 + D.count * (bit_width(m - 1) + 1)
        rk.left.reserve(nodes)
        rk.right.reserve(nodes)
        rk.cnt.reserve(nodes)
        rk.roots.reserve(D.count + 1)
        for i in D:
            let prev = rk.roots.last!
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

    /// The `k`-th smallest element (0-based) of `a` over `r`, counting repeats.
    ///
    /// cost: \(O(\log n)\)
    /// empty: traps (no `k` is in range)
    /// ties: equal values are indistinguishable; the value is returned
    /// traps: when `k < 0` or `k >= r.len`, or `r` reaches outside `D`
    /// example: `RangeKth([5, 1, 4, 2]).kth(1..4, 1) == 2`
    pub fn kth(self, r: Range<D>, k: Int) -> T:
        let (lo, hi) = r.ends
        assert 0 <= k < hi - lo, "RangeKth.kth index out of range"
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

    /// The number of elements of `a` over `r` that are at most `x`.
    ///
    /// cost: \(O(\log n)\)
    /// empty: 0
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `RangeKth([5, 1, 4, 2]).count_le(0..4, 4) == 3`
    pub fn count_le(self, r: Range<D>, x: T) -> Int:
        let (lo, hi) = r.ends
        if lo == hi or vals.is_empty: return 0
        let pos = self.upper_val(x)
        self.count_prefix(roots[lo], roots[hi], 0, vals.len, pos)

    /// The number of elements of `a` over `r` that are less than `x`.
    ///
    /// cost: \(O(\log n)\)
    /// empty: 0
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `RangeKth([5, 1, 4, 2]).count_lt(0..4, 4) == 2`
    pub fn count_lt(self, r: Range<D>, x: T) -> Int:
        let (lo, hi) = r.ends
        if lo == hi or vals.is_empty: return 0
        self.count_prefix(roots[lo], roots[hi], 0, vals.len, self.lower_val(x))

    /// The number of elements of `a` over `r` in `lo_v ..= hi_v` (both ends inclusive); 0 when
    /// `lo_v > hi_v`.
    ///
    /// cost: \(O(\log n)\)
    /// empty: 0
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `RangeKth([5, 1, 4, 2]).count_in(0..4, 2, 5) == 3`
    /// example: `RangeKth([5, 1, 4, 2]).count_in(0..4, 5, 2) == 0`
    pub fn count_in(self, r: Range<D>, lo_v: T, hi_v: T) -> Int:
        if hi_v < lo_v: return 0
        self.count_le(r, hi_v) - self.count_lt(r, lo_v)

/// \(O(q \log \lvert D \rvert)\) space. Path-copying persistent segment tree with lazy range updates:
/// every `apply`/`set` returns a new `Version` in \(O(\log \lvert D \rvert)\) time and memory.
pub struct PersistentLazySegTree<D, T, F>(act: Action<T, F>):
    left: List<0..=2_000_000_000>
    right: List<0..=2_000_000_000>
    val: List<T>
    tag: List<F>
    var base_root: Int

    fn push_node(mut self, l: Int, r: Int, v: T, f: F) -> Int:
        let id = val.len
        left.push(l)
        right.push(r)
        val.push(v)
        tag.push(f)
        id

    fn build_list(mut self, vals: List<T>, nl: Int, nr: Int) -> Int:
        if nr - nl == 1: return self.push_node(0, 0, vals[nl], act.id)
        let mid = (nl + nr) / 2
        let lc = self.build_list(vals, nl, mid)
        let rc = self.build_list(vals, mid, nr)
        self.push_node(lc, rc, act.over.op(val[lc], val[rc]), act.id)

    fn tagged(mut self, u: Int, span: Int, f: F) -> Int:
        self.push_node(left[u], right[u], act.apply(f, val[u], span), act.compose(f, tag[u]))

    fn apply_rec(mut self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int, f: F) -> Int:
        if qr <= nl or nr <= ql: return u
        if ql <= nl and nr <= qr: return self.tagged(u, nr - nl, f)
        let mid = (nl + nr) / 2
        let lc = self.apply_rec(self.tagged(left[u], mid - nl, tag[u]), nl, mid, ql, qr, f)
        let rc = self.apply_rec(self.tagged(right[u], nr - mid, tag[u]), mid, nr, ql, qr, f)
        self.push_node(lc, rc, act.over.op(val[lc], val[rc]), act.id)

    fn set_rec(mut self, u: Int, nl: Int, nr: Int, idx: Int, x: T) -> Int:
        if nr - nl == 1: return self.push_node(0, 0, x, act.id)
        let mid = (nl + nr) / 2
        var lc = self.tagged(left[u], mid - nl, tag[u])
        var rc = self.tagged(right[u], nr - mid, tag[u])
        if idx < mid: lc = self.set_rec(lc, nl, mid, idx, x)
        else: rc = self.set_rec(rc, mid, nr, idx, x)
        self.push_node(lc, rc, act.over.op(val[lc], val[rc]), act.id)

    /// `acc` holds the tags of the ancestors of `u`, which are newer than every tag below them.
    pub fn fold_rec(self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int, acc: F) -> T:
        if qr <= nl or nr <= ql: return act.over.id
        if ql <= nl and nr <= qr: return act.apply(acc, val[u], nr - nl)
        let mid = (nl + nr) / 2
        let down = act.compose(acc, tag[u])
        act.over.op(
            self.fold_rec(left[u], nl, mid, ql, qr, down),
            self.fold_rec(right[u], mid, nr, ql, qr, down),
        )

    /// Build version 0 over `D` from `a:`, a uniform `fill:`, or the identity of `act`'s monoid, and
    /// keep it as `base`. Range writes copy \(O(\log n)\) nodes and their pending tags, never changing an
    /// older version.
    ///
    /// cost: \(O(n)\) time and memory with `a:` or `fill:`, \(O(1)\) otherwise, for n = `|D|`
    /// empty: an empty `D` gives a tree whose every fold is the identity
    /// traps: when both `a:` and `fill:` are given
    /// example: `var t = PersistentLazySegTree(Action<Int, Int>(apply: (d, x, span) => x + d * span), a: [1, 2, 3]); t.all(t.base) == 6`
    pub fn new(
        act: Action<T,
        F>,
        a: ?[D]T = None,
        fill: ?T = None,
    ) -> PersistentLazySegTree<D, T, F>:
        var t = PersistentLazySegTree<D, T, F>(act, [0], [0], [act.over.id], [act.id], 0)
        if let init = a:
            assert fill == None, "pass either `a:` or `fill:`, not both"
            if D.count > 0: t.base_root = t.build_list([init[i] for i in D], 0, D.count)
        elif let v = fill:
            if D.count > 0: t.base_root = t.build_list(List.filled(D.count, v), 0, D.count)
        t

    /// The version built by `new`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var t = PersistentLazySegTree(Action<Int, Int>(apply: (d, x, span) => x + d * span), a: [1, 2, 3]); t.get(t.base, 2) == 3`
    @get pub fn base(self) -> Version:
        Version(base_root)

    /// A new version equal to `v` with the tag `f` applied to every position of `r`.
    ///
    /// cost: \(O(\log n)\) time and memory
    /// empty: an empty `r` returns a version equal to `v`
    /// traps: when a non-empty `r` reaches outside `D`, or `v` did not come from this tree
    /// example: `var t = PersistentLazySegTree(Action<Int, Int>(apply: (d, x, span) => x + d * span), a: [1, 2, 3]); let v = t.apply(t.base, 0..2, 10); t.all(v) == 26 and t.all(t.base) == 6`
    pub fn apply(mut self, v: Version, r: Range<D>, f: F) -> Version:
        let (lo, hi) = r.ends
        if lo == hi: return v
        Version(self.apply_rec(v.root, 0, D.count, lo, hi, f))

    /// A new version equal to `v` with position `i` set to `x`, discarding tags pending above it.
    ///
    /// cost: \(O(\log n)\) time and memory
    /// traps: when `i` is not in `D`, or `v` did not come from this tree
    /// example: `var t = PersistentLazySegTree(Action<Int, Int>(apply: (d, x, span) => x + d * span), a: [1, 2, 3]); let v = t.apply(t.base, 0..3, 1); let w = t.set(v, 0, 0); t.all(w) == 7`
    pub fn set(mut self, v: Version, i: D, x: T) -> Version:
        Version(self.set_rec(v.root, 0, D.count, i.ord, x))

    /// The fold of `r` in version `v`, left to right, with every applied tag accounted for.
    ///
    /// cost: \(O(\log n)\)
    /// empty: the monoid identity
    /// traps: when a non-empty `r` reaches outside `D`, or `v` did not come from this tree
    /// example: `var t = PersistentLazySegTree(Action<Int, Int>(apply: (d, x, span) => x + d * span), a: [1, 2, 3]); let v = t.apply(t.base, 1..3, 1); t.fold(v, 1..3) == 7`
    pub fn fold(self, v: Version, r: Range<D>) -> T:
        let (lo, hi) = r.ends
        if lo == hi: return act.over.id
        self.fold_rec(v.root, 0, D.count, lo, hi, act.id)

    /// The value at position `i` in version `v`, with every applied tag accounted for.
    ///
    /// cost: \(O(\log n)\)
    /// traps: when `i` is not in `D`, or `v` did not come from this tree
    /// example: `var t = PersistentLazySegTree(Action<Int, Int>(apply: (d, x, span) => x + d * span), a: [1, 2, 3]); let v = t.apply(t.base, 1..3, 1); t.get(v, 1) == 3 and t.get(t.base, 1) == 2`
    pub fn get(self, v: Version, i: D) -> T:
        self.fold_rec(v.root, 0, D.count, i.ord, i.ord + 1, act.id)

    /// The fold of every position in version `v`, read from its root.
    ///
    /// cost: \(O(1)\)
    /// empty: the monoid identity when `D` is empty
    /// traps: when `v` did not come from this tree
    /// example: `var t = PersistentLazySegTree(Action<Int, Int>(apply: (d, x, span) => x + d * span), a: [1, 2, 3]); let v = t.apply(t.base, 0..3, 2); t.all(v) == 12`
    pub fn all(self, v: Version) -> T:
        val[v.root]

/// \(O(q \log \lvert D \rvert)\) space. Persistent array over domain `D`: \(O(\log \lvert D \rvert)\) `get` and path-copying `set`.
pub struct PersistentArray<D, T>:
    left: List<0..=2_000_000_000>
    right: List<0..=2_000_000_000>
    val: List<T>
    var base_root: Int

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
        self.push_node(lc, rc, vals[nl])

    fn set_rec(mut self, u: Int, nl: Int, nr: Int, idx: Int, x: T) -> Int:
        if nr - nl == 1: return self.push_node(0, 0, x)
        let mid = (nl + nr) / 2
        if idx < mid: return self.push_node(
            self.set_rec(left[u], nl, mid, idx, x),
            right[u],
            val[u],
        )
        self.push_node(left[u], self.set_rec(right[u], mid, nr, idx, x), val[u])

    /// Build version 0 over `D` from `a:` or a uniform `fill:`, and keep it as `base`.
    ///
    /// cost: \(O(n)\) time and memory for n = `|D|`
    /// empty: an empty `D` gives an array with no positions
    /// traps: when both or neither of `a:` and `fill:` are given
    /// example: `var p = PersistentArray<0..3, Int>(fill: 7); p.get(p.base, 0) == 7`
    pub fn new(a: ?[D]T = None, fill: ?T = None) -> PersistentArray<D, T>:
        var p = PersistentArray<D, T>([0], [0], [], 0)
        if let init = a:
            assert fill == None, "pass either `a:` or `fill:`, not both"
            if D.count > 0: p.val.push(init[D.nth(0)])
            if D.count > 0: p.base_root = p.build_list([init[i] for i in D], 0, D.count)
        elif let v = fill:
            p.val.push(v)
            if D.count > 0: p.base_root = p.build_list(List.filled(D.count, v), 0, D.count)
        else:
            assert false, "pass `a:` or `fill:`"
        p

    /// The version built by `new`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var p = PersistentArray(a: [1, 2, 3]); p.get(p.base, 2) == 3`
    @get pub fn base(self) -> Version:
        Version(base_root)

    /// The value at position `i` in version `v`.
    ///
    /// cost: \(O(\log n)\)
    /// traps: when `i` is not in `D`, or `v` did not come from this array
    /// example: `var p = PersistentArray(a: [1, 2, 3]); p.get(p.base, 1) == 2`
    pub fn get(self, v: Version, i: D) -> T:
        var (u, nl, nr, idx) = (v.root, 0, D.count, i.ord)
        while nr - nl > 1:
            let mid = (nl + nr) / 2
            if idx < mid:
                u = left[u]
                nr = mid
            else:
                u = right[u]
                nl = mid
        val[u]

    /// A new version equal to `v` with position `i` set to `x`; `v` is unchanged.
    ///
    /// cost: \(O(\log n)\) time and memory
    /// traps: when `i` is not in `D`, or `v` did not come from this array
    /// example: `var p = PersistentArray(a: [1, 2, 3]); let v = p.set(p.base, 0, 9); p.get(v, 0) == 9 and p.get(p.base, 0) == 1`
    pub fn set(mut self, v: Version, i: D, x: T) -> Version:
        Version(self.set_rec(v.root, 0, D.count, i.ord, x))

/// \(O(\log \lvert D \rvert)\) per update. Hashed persistent multiset over domain `D`:
/// \(O(1)\) equality, \(O(\log \lvert D \rvert)\) lexicographic comparison (`desc: true` compares largest element first).
pub struct PMultiset<D>:
    root: Int

    /// The empty multiset over `D`. Every multiset is an immutable value; `==` compares two in \(O(1)\)
    /// by hash.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `PMultiset<0..10>.empty().count(3) == 0`
    pub fn empty() -> PMultiset<D> = todo

    /// A new multiset with one more copy of `x`; `self` is unchanged.
    ///
    /// cost: \(O(\log \lvert D \rvert)\) time and memory
    /// traps: when `x` is not in `D`
    /// example: `let s = PMultiset<0..10>.empty().with(3).with(3); s.count(3) == 2`
    pub fn with(self, x: D) -> PMultiset<D> = todo

    /// A new multiset with one copy of `x` removed, or equal to `self` when `x` is absent; `self` is
    /// unchanged.
    ///
    /// cost: \(O(\log \lvert D \rvert)\) time and memory
    /// traps: when `x` is not in `D`
    /// example: `let s = PMultiset<0..10>.empty().with(3).with(3); s.without(3).count(3) == 1 and s.count(3) == 2`
    /// example: `let s = PMultiset<0..10>.empty(); s.without(3) == s`
    pub fn without(self, x: D) -> PMultiset<D> = todo

    /// The number of copies of `x`.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// traps: when `x` is not in `D`
    /// example: `PMultiset<0..10>.empty().with(4).count(4) == 1`
    pub fn count(self, x: D) -> Int = todo

    /// Compare the elements of `self` and `other` as sorted sequences, lexicographically: ascending,
    /// or largest first with `desc: true`. A proper prefix is smaller. Returns `-1`, `0` or `1`.
    ///
    /// The first differing key is found by hash in \(O(\log \lvert D \rvert)\); `0` exactly when the multisets are
    /// equal.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// ties: `0` only for equal multisets
    /// traps: never
    /// example: `let e = PMultiset<0..10>.empty(); e.with(1).with(3).cmp(e.with(2)) == -1`
    /// example: `let e = PMultiset<0..10>.empty(); e.with(1).with(3).cmp(e.with(2), desc: true) == 1`
    /// example: `let e = PMultiset<0..10>.empty(); e.with(1).cmp(e.with(1).with(2)) == -1`
    pub fn cmp(self, other: PMultiset<D>, desc: Bool = false) -> Int = todo

@test
fn persistent_segtree_and_range_kth_match_brute():
    domain I = 1..=10
    let a = [I] _ => rand(-15, 15)
    var pst = PersistentSegTree<I, Int>(+, a)
    var versions: List<Version> = [pst.base]
    var snaps: List<[I]Int> = [a]
    fn total(arr: [I]Int, r: Range<I>) -> Int:
        let s = sum(arr[i] for i in r)
        s
    for _ in 0..30:
        let base_id = rand(0, versions.len - 1)
        let i = I(rand(1, 10))
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
                assert pst.fold(v, I(l)..=I(r)) == total(arr, I(l)..=I(r))
        assert pst.get(v, I(3)) == arr[I(3)]
        assert pst.all(v) == total(arr, I(1)..=I(10))
    let rk = RangeKth<I, Int>(a)
    for l in 1..=10:
        for r in l..=10:
            let rng = I(l)..=I(r)
            let s = [a[i] for i in rng].sorted()
            for k in 0..s.len: assert rk.kth(rng, k) == s[k]
            for x in [-20, -5, 0, 5, 20]:
                var c = 0
                for y in s:
                    if y <= x: c += 1
                assert rk.count_le(rng, x) == c
