//! Persistent segment tree and static range k-th smallest queries.
//!
//! start: `PersistentSegTree`, `RangeKth`, `PersistentArray`, `PArena`
//! example: `var t = PersistentSegTree(+, a: [1, 2, 3]); t.all(t.base) == 6`
//!
//! Each update copies only the \(O(\log n)\) nodes on the root-to-leaf path and returns a
//! new `Version` handle. All earlier versions remain readable in \(O(\log n)\).
use std.algebra.{Action, Monoid}
use std.marks.{Id}
use std.hash.{splitmix64}
use std.math.{bit_width}
use std.range.{Len, ends}
use std.seq.{each, Seq, deduped, all, sorted}

/// An append-only node pool for persistent structures, held by the user: `alloc` returns a 4-byte
/// `Id<T>`, and a node never changes after it is allocated, so every version that shares it stays
/// valid. Persistent values such as `PMultiset` keep their nodes here and take the arena at each
/// call; the library keeps no pool of its own, so a pool declared inside a test case is fresh in
/// every case.
///
/// not a container: a pool; it has no `set` or `clear`, since either would change a node some
/// version still reads.
pub struct PArena<T>:
    _xs: List<T>

    /// An empty arena with room for `cap` nodes before it reallocates.
    ///
    /// cost: \(O(1)\); the room is not touched until it is used
    /// traps: when `cap < 0` or `cap >= 2^31`
    /// example: `PArena<Int>(cap: 8).is_empty`
    pub fn new(cap: Len = 0) -> PArena<T>:
        var xs: List<T> = []
        xs.reserve(cap)
        PArena(xs)

    /// The number of nodes allocated; valid handles have raw values `0..len`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var a = PArena<Int>(); a.alloc(5); a.alloc(6); a.len == 2`
    @get pub fn len(self) -> Int:
        _xs.len

    /// Whether no node is allocated; `len == 0`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `PArena<Int>().is_empty`
    @get pub fn is_empty(self) -> Bool:
        _xs.len == 0

    /// Store `x` as a new node and return its handle; handles are issued in order `0, 1, 2, ...`.
    ///
    /// cost: \(O(1)\) amortized
    /// traps: when the arena already holds `2_000_000_001` nodes
    /// example: `var a = PArena<Str>(); a.alloc("p"); a.alloc("q").raw == 1`
    pub fn alloc(mut self, x: T) -> Id<T>:
        _xs.push(x)
        Id(_xs.len - 1)

    /// The node behind `id`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `id` is not below `len`, so a handle from another arena that is out of range
    /// example: `var a = PArena<Int>(); let i = a.alloc(5); a.alloc(6); a.get(i) == 5`
    pub fn get(self, id: Id<T>) -> T:
        _xs[id.raw]

    /// Allocate room for at least `cap` nodes in total; changes no handle or node.
    ///
    /// cost: \(O(\mathit{len})\) when it grows (it moves the nodes), \(O(1)\) otherwise
    /// traps: when `cap < 0` or `cap >= 2^31`
    /// example: `var a = PArena<Int>(); a.reserve(100); a.alloc(1); a.len == 1`
    pub fn reserve(mut self, cap: Len):
        _xs.reserve(cap)

/// Opaque version handle for `PersistentSegTree`.
/// One version of a persistent tree or array of type `S`: the root of its nodes in the `S` value
/// that made it. Only `S`'s own methods build one, so a version of one structure type can't be
/// read through another.
pub struct Version<S>(root: Int)

/// A store of versions read one position at a time: `PersistentSegTree`, `PersistentLazySegTree`
/// and `PersistentArray`. `v.within(store)` reads any version of one as a container.
pub trait Versioned<D, T>:
    /// The value at position `i` in version `v`.
    fn get(self, v: Version<Self>, i: D) -> T

impl<S: Versioned<D, T>, D: Domain, T> Version<S>:
    /// This version read through `store`, the structure that made it, as a container of its values
    /// in key order.
    ///
    /// cost: \(O(1)\); it lends `store` and copies nothing
    /// traps: never; reading it traps when `store` did not make this version
    /// example: `var p = PersistentArray(a: [1, 2, 3]); let v = p.set(p.base, 0, 9); v.within(p).each().collect() == [9, 2, 3]`
    pub fn within(self, store: S) -> lends VersionWithin<S, D, T> = VersionWithin(store, self)

impl<S: Versioned<D, T>, D: Domain, T> Seq<T> for VersionWithin<S, D, T>:
    /// Visit the value at every position.
    ///
    /// cost: \(O(\lvert D \rvert \log \lvert D \rvert)\)
    /// empty: `var p = PersistentArray<0..0, Int>(fill: 0); p.base.within(p).each().collect() == []`
    /// order: ascending `D` order
    /// traps: when `store` did not make this version
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); let v = t.set(t.base, 1, 7); v.within(t).each().collect() == [1, 7, 3]`
    fn iter(self) -> yields T:
        for i in D: yield store.get(v, i)

/// One version of a `Versioned` store read through it, from `v.within(store)`. It lends the store,
/// so it lives no longer than it and copies nothing.
pub struct VersionWithin<S: Versioned<D, T>, D: Domain, T>(store: S, v: Version<S>):
    /// The number of positions, `D.count`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var t = PersistentSegTree(+, a: [0, 0, 0]); t.base.within(t).len == 3`
    @get pub fn len(self) -> Int = D.count

    /// Whether `D` has no position.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var p = PersistentArray<0..0, Int>(fill: 0); p.base.within(p).is_empty`
    @get pub fn is_empty(self) -> Bool = D.count == 0

    /// Visit every position with its value.
    ///
    /// cost: \(O(\lvert D \rvert \log \lvert D \rvert)\)
    /// empty: `var p = PersistentArray<0..0, Int>(fill: 0); p.base.within(p).items.collect() == []`
    /// order: ascending `D` order
    /// traps: when `store` did not make this version
    /// example: `var p = PersistentArray(a: [4, 5]); [(i, x) for (i, x) in p.base.within(p).items] == [(0, 4), (1, 5)]`
    @get pub fn items(self) -> yields (D, T):
        for i in D: yield (i, store.get(v, i))

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
    /// empty: `var t = PersistentSegTree<0..0, Int>(min); t.all(t.base) == Int.MAX`
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
    @get pub fn base(self) -> Version<PersistentSegTree<D, T>>:
        Version<PersistentSegTree<D, T>>(base_root)

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
    pub fn set(
        mut self,
        v: Version<PersistentSegTree<D, T>>,
        i: D,
        x: T,
    ) -> Version<PersistentSegTree<D, T>>:
        Version<PersistentSegTree<D, T>>(self.set_rec(v.root, 0, D.count, i.ord, x))

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
    pub fn update(
        mut self,
        v: Version<PersistentSegTree<D, T>>,
        i: D,
        f: T -> T,
    ) -> Version<PersistentSegTree<D, T>>:
        Version<PersistentSegTree<D, T>>(self.update_rec(v.root, 0, D.count, i.ord, f))

    /// The value at position `i` in version `v`.
    ///
    /// cost: \(O(\log n)\)
    /// traps: when `i` is not in `D`, or `v` did not come from this tree
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); t.get(t.base, 1) == 2`
    pub fn get(self, v: Version<PersistentSegTree<D, T>>, i: D) -> T:
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
    /// empty: `var t = PersistentSegTree(min, a: [1, 2]); t.fold(t.base, 1..1) == Int.MAX`
    /// traps: when a non-empty `r` reaches outside `D`, or `v` did not come from this tree
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); t.fold(t.base, 1..3) == 5`
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); t.fold(t.base, 2..2) == 0`
    pub fn fold(self, v: Version<PersistentSegTree<D, T>>, r: Range<D>) -> T:
        let (lo, hi) = r.ends
        if lo == hi: return m.id
        self.fold_rec(v.root, 0, D.count, lo, hi)

    /// The fold of every position in version `v`, read from its root.
    ///
    /// cost: \(O(1)\)
    /// empty: `var t = PersistentSegTree<0..0, Int>(+); t.all(t.base) == 0`
    /// traps: when `v` did not come from this tree
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); let v = t.set(t.base, 0, 0); t.all(v) == 5`
    pub fn all(self, v: Version<PersistentSegTree<D, T>>) -> T = val[v.root]

    /// Preallocate room for `nodes` tree nodes in total, so later writes do not regrow the arena.
    /// Changes no version.
    ///
    /// cost: \(O(\mathit{len})\) when it grows (it moves the nodes it holds), \(O(1)\) otherwise
    /// traps: when `nodes < 0` or `nodes >= 2^31`
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); t.reserve(100); t.all(t.base) == 6`
    pub fn reserve(mut self, nodes: Len):
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
    /// ties: `var t = PersistentSegTree(+, a: [1, 0, 0, 5]); t.right_while(t.base, 0, s => s <= 1) == 3`
    /// traps: when `from` is not a cut of `D`, when `ok(identity)` is false, or `v` did not come from this tree
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); t.right_while(t.base, 0, s => s <= 3) == 2`
    pub fn right_while(
        self,
        v: Version<PersistentSegTree<D, T>>,
        from: Cut<D>,
        ok: T -> Bool,
    ) -> Cut<D>:
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
    /// ties: `var t = PersistentSegTree(+, a: [5, 0, 0, 1]); t.left_while(t.base, 4, s => s <= 1) == 1`
    /// traps: when `to` is not a cut of `D`, when `ok(identity)` is false, or `v` did not come from this tree
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); t.left_while(t.base, 3, s => s <= 5) == 1`
    pub fn left_while(
        self,
        v: Version<PersistentSegTree<D, T>>,
        to: Cut<D>,
        ok: T -> Bool,
    ) -> Cut<D>:
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
    /// `hi - lo` (0-based, like `RangeKth.kth`), walking both versions at once; `None` when the
    /// difference holds at most `k` items.
    /// Items at one position are interchangeable; the position is returned.
    ///
    /// A position with count `c` holds `c` items.
    ///
    /// Law: `hi` was derived from `lo` by adding counts, so no position of `hi - lo` is negative.
    ///
    /// cost: \(O(\log n)\)
    /// ties: `var t = PersistentSegTree(+, a: [0, 0]); let v = t.set(t.base, 1, 2); t.diff_kth(v, t.base, 0) == Some(1) and t.diff_kth(v, t.base, 1) == Some(1)`
    /// traps: when `k < 0`, or a version did not come from this tree
    /// example: `var t = PersistentSegTree(+, a: [0, 0, 0, 0]); let v1 = t.set(t.base, 1, 1); let v2 = t.set(v1, 3, 2); t.diff_kth(v2, t.base, 1) == Some(3) and t.diff_kth(v2, t.base, 3) == None`
    /// example: `var t = PersistentSegTree(+, a: [0, 0, 0, 0]); let v1 = t.set(t.base, 1, 1); let v2 = t.set(v1, 3, 2); t.diff_kth(v2, v1, 0) == Some(3)`
    pub fn diff_kth(
        self,
        hi: Version<PersistentSegTree<D, T>>,
        lo: Version<PersistentSegTree<D, T>>,
        k: Int,
    ) -> ?D:
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

struct KthNode(left: 0..=2_000_000_000, right: 0..=2_000_000_000, cnt: 0..=2_000_000_000)

/// Static range k-th smallest and range rank queries over `[D]T`.
/// Built in \(O(n \log n)\) using prefix versions of a persistent count tree; queries run in \(O(\log n)\).
pub struct RangeKth<D, T>:
    vals: List<T>
    // One list of nodes, so a query step reads one cache line per version, not three.
    nodes: List<KthNode>
    roots: List<0..=2_000_000_000>

    /// Build the static structure over `a`: one persistent count tree over the sorted distinct values
    /// per prefix of `a`. `a` is copied; later changes to it are not seen.
    ///
    /// cost: \(O(n \log n)\) time and memory for n = `a.len`
    /// empty: `RangeKth<0..0, Int>([]).count_le(0..0, 5) == 0`
    /// order: queries take ranges of `a`'s domain
    /// traps: never
    /// example: `RangeKth([5, 1, 4, 2]).kth(0..4, 0) == 1`
    pub fn new(a: [D]T) -> RangeKth<D, T>:
        let vals = deduped(sorted([a[i] for i in D]))
        let m = vals.len
        var rk = RangeKth(vals, [KthNode(0, 0, 0)], [0])
        if m == 0:
            for _ in 0..D.count: rk.roots.push(0)
            return rk
        let nodes = 1 + D.count * (bit_width(m - 1) + 1)
        rk.nodes.reserve(nodes)
        rk.roots.reserve(D.count + 1)
        for i in D:
            let prev = rk.roots.last!
            let pos = rk.lower_val(a[i])
            let nxt = rk.inc(prev, m, pos)
            rk.roots.push(nxt)
        rk

    fn lower_val(self, x: T) -> Int where T: Ord:
        var (lo, hi) = (0, vals.len)
        unchecked:
            while lo < hi:
                let mid = (lo + hi) / 2
                if vals[mid] < x: lo = mid + 1
                else: hi = mid
        lo

    fn upper_val(self, x: T) -> Int where T: Ord:
        var (lo, hi) = (0, vals.len)
        unchecked:
            while lo < hi:
                let mid = (lo + hi) / 2
                if vals[mid] <= x: lo = mid + 1
                else: hi = mid
        lo

    fn inc(mut self, root: Int, m: Int, pos: Int) -> Int:
        unchecked:
            let new_root = nodes.len
            var (u, nl, nr) = (root, 0, m)
            while true:
                let next = nodes.len + 1
                let (lu, ru, c) = (nodes[u].left, nodes[u].right, nodes[u].cnt + 1)
                if nr - nl == 1:
                    nodes.push(KthNode(lu, ru, c))
                    break
                let mid = (nl + nr) / 2
                if pos < mid:
                    nodes.push(KthNode(next, ru, c))
                    u = lu
                    nr = mid
                else:
                    nodes.push(KthNode(lu, next, c))
                    u = ru
                    nl = mid
            return new_root

    /// The `k`-th smallest element (0-based) of `a` over `r`, counting repeats.
    /// Equal values are indistinguishable; the value is returned.
    ///
    /// cost: \(O(\log n)\)
    /// empty: `RangeKth([5, 1, 4]).kth(1..1, 0) == 0` (traps R0401)
    /// ties: `let k = RangeKth([2, 1, 2]); k.kth(0..3, 1) == 2 and k.kth(0..3, 2) == 2`
    /// traps: when `k < 0` or `k >= r.len`, or `r` reaches outside `D`
    /// example: `RangeKth([5, 1, 4, 2]).kth(1..4, 1) == 2`
    pub fn kth(self, r: Range<D>, k: 0..) -> T:
        let (lo, hi) = r.ends
        assert 0 <= k < hi - lo, "RangeKth.kth index out of range"
        unchecked:
            var (u, v, nl, nr, rem) = (roots[lo], roots[hi], 0, vals.len, k)
            while nr - nl > 1:
                let mid = (nl + nr) / 2
                let left_cnt = nodes[nodes[v].left].cnt - nodes[nodes[u].left].cnt
                if rem < left_cnt:
                    u = nodes[u].left
                    v = nodes[v].left
                    nr = mid
                else:
                    rem -= left_cnt
                    u = nodes[u].right
                    v = nodes[v].right
                    nl = mid
            return vals[nl]

    fn count_prefix(self, u: Int, v: Int, nl: Int, nr: Int, limit: Int) -> Int:
        if limit <= nl or (u == 0 and v == 0): return 0
        if nr <= limit: return nodes[v].cnt - nodes[u].cnt
        let mid = (nl + nr) / 2
        let lc = self.count_prefix(nodes[u].left, nodes[v].left, nl, mid, limit)
        let rc = self.count_prefix(nodes[u].right, nodes[v].right, mid, nr, limit)
        lc + rc

    /// The number of elements of `a` over `r` that are at most `x`.
    ///
    /// cost: \(O(\log n)\)
    /// empty: `RangeKth([5, 1, 4]).count_le(1..1, 9) == 0`
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
    /// empty: `RangeKth([5, 1, 4]).count_lt(1..1, 9) == 0`
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
    /// empty: `RangeKth([5, 1, 4]).count_in(1..1, 0, 9) == 0`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `RangeKth([5, 1, 4, 2]).count_in(0..4, 2, 5) == 3`
    /// example: `RangeKth([5, 1, 4, 2]).count_in(0..4, 5, 2) == 0`
    pub fn count_in(self, r: Range<D>, lo_v: T, hi_v: T) -> Int where T: Ord:
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

    // `acc` holds the tags of the ancestors of `u`, which are newer than every tag below them.
    fn fold_rec(self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int, acc: F) -> T:
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
    /// empty: `var t = PersistentLazySegTree<0..0, Int, Int>(Action<Int, Int>(apply: (d, x, span) => x + d * span)); t.all(t.base) == 0`
    /// traps: when both `a:` and `fill:` are given
    /// example: `var t = PersistentLazySegTree(Action<Int, Int>(apply: (d, x, span) => x + d * span), a: [1, 2, 3]); t.all(t.base) == 6`
    pub fn new(
        act: Action<T, F>,
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
    @get pub fn base(self) -> Version<PersistentLazySegTree<D, T, F>>:
        Version<PersistentLazySegTree<D, T, F>>(base_root)

    /// A new version equal to `v` with the tag `f` applied to every position of `r`.
    ///
    /// cost: \(O(\log n)\) time and memory
    /// empty: `var t = PersistentLazySegTree(Action<Int, Int>(apply: (d, x, span) => x + d * span), a: [1, 2, 3]); let v = t.apply(t.base, 1..1, 10); t.all(v) == 6 and t.fold(v, 0..3) == 6`
    /// traps: when a non-empty `r` reaches outside `D`, or `v` did not come from this tree
    /// example: `var t = PersistentLazySegTree(Action<Int, Int>(apply: (d, x, span) => x + d * span), a: [1, 2, 3]); let v = t.apply(t.base, 0..2, 10); t.all(v) == 26 and t.all(t.base) == 6`
    pub fn apply(
        mut self,
        v: Version<PersistentLazySegTree<D, T, F>>,
        r: Range<D>,
        f: F,
    ) -> Version<PersistentLazySegTree<D, T, F>>:
        let (lo, hi) = r.ends
        if lo == hi: return v
        Version<PersistentLazySegTree<D, T, F>>(self.apply_rec(v.root, 0, D.count, lo, hi, f))

    /// A new version equal to `v` with position `i` set to `x`, discarding tags pending above it.
    ///
    /// cost: \(O(\log n)\) time and memory
    /// traps: when `i` is not in `D`, or `v` did not come from this tree
    /// example: `var t = PersistentLazySegTree(Action<Int, Int>(apply: (d, x, span) => x + d * span), a: [1, 2, 3]); let v = t.apply(t.base, 0..3, 1); let w = t.set(v, 0, 0); t.all(w) == 7`
    pub fn set(
        mut self,
        v: Version<PersistentLazySegTree<D, T, F>>,
        i: D,
        x: T,
    ) -> Version<PersistentLazySegTree<D, T, F>>:
        Version<PersistentLazySegTree<D, T, F>>(self.set_rec(v.root, 0, D.count, i.ord, x))

    /// The fold of `r` in version `v`, left to right, with every applied tag accounted for.
    ///
    /// cost: \(O(\log n)\)
    /// empty: `var t = PersistentLazySegTree(Action<Int, Int>(apply: (d, x, span) => x + d * span), a: [1, 2, 3]); t.fold(t.base, 1..1) == 0`
    /// traps: when a non-empty `r` reaches outside `D`, or `v` did not come from this tree
    /// example: `var t = PersistentLazySegTree(Action<Int, Int>(apply: (d, x, span) => x + d * span), a: [1, 2, 3]); let v = t.apply(t.base, 1..3, 1); t.fold(v, 1..3) == 7`
    pub fn fold(self, v: Version<PersistentLazySegTree<D, T, F>>, r: Range<D>) -> T:
        let (lo, hi) = r.ends
        if lo == hi: return act.over.id
        self.fold_rec(v.root, 0, D.count, lo, hi, act.id)

    /// The value at position `i` in version `v`, with every applied tag accounted for.
    ///
    /// cost: \(O(\log n)\)
    /// traps: when `i` is not in `D`, or `v` did not come from this tree
    /// example: `var t = PersistentLazySegTree(Action<Int, Int>(apply: (d, x, span) => x + d * span), a: [1, 2, 3]); let v = t.apply(t.base, 1..3, 1); t.get(v, 1) == 3 and t.get(t.base, 1) == 2`
    pub fn get(self, v: Version<PersistentLazySegTree<D, T, F>>, i: D) -> T:
        self.fold_rec(v.root, 0, D.count, i.ord, i.ord + 1, act.id)

    /// The fold of every position in version `v`, read from its root.
    ///
    /// cost: \(O(1)\)
    /// empty: `var t = PersistentLazySegTree<0..0, Int, Int>(Action<Int, Int>(apply: (d, x, span) => x + d * span)); t.all(t.base) == 0`
    /// traps: when `v` did not come from this tree
    /// example: `var t = PersistentLazySegTree(Action<Int, Int>(apply: (d, x, span) => x + d * span), a: [1, 2, 3]); let v = t.apply(t.base, 0..3, 2); t.all(v) == 12`
    pub fn all(self, v: Version<PersistentLazySegTree<D, T, F>>) -> T:
        val[v.root]

/// Persistent array over domain `D`: \(O(\log \lvert D \rvert)\) `get` and path-copying `set`.
/// \(O(q \log \lvert D \rvert)\) space.
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
    /// empty: `domain E = 0..0; var p = PersistentArray<E, Int>(fill: 7); [p.get(p.base, i) for i in E] == []`
    /// traps: when both or neither of `a:` and `fill:` are given
    /// example: `var p = PersistentArray<0..3, Int>(fill: 7); p.get(p.base, 0) == 7`
    /// trap: `PersistentArray<0..3, Int>()`
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
    @get pub fn base(self) -> Version<PersistentArray<D, T>>:
        Version<PersistentArray<D, T>>(base_root)

    /// The value at position `i` in version `v`.
    ///
    /// cost: \(O(\log n)\)
    /// traps: when `i` is not in `D`, or `v` did not come from this array
    /// example: `var p = PersistentArray(a: [1, 2, 3]); p.get(p.base, 1) == 2`
    pub fn get(self, v: Version<PersistentArray<D, T>>, i: D) -> T:
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
    pub fn set(
        mut self,
        v: Version<PersistentArray<D, T>>,
        i: D,
        x: T,
    ) -> Version<PersistentArray<D, T>>:
        Version<PersistentArray<D, T>>(self.set_rec(v.root, 0, D.count, i.ord, x))

/// A hashed persistent multiset over domain `D`, an immutable value whose nodes live in a
/// `PArena<PMultiset<D>>` the caller holds and passes to every call that reads or adds nodes.
/// `with`/`without` copy the \(O(\log \lvert D \rvert)\) nodes on one root-to-leaf path; `==` is
/// \(O(1)\) by size and hash; `cmp` finds the first differing key by hash in
/// \(O(\log \lvert D \rvert)\).
///
/// A multiset is a segment tree over `D.ord` whose node at a range holds the count and hash of the
/// keys in it, and the hash adds a fixed 64-bit weight per copy of a key; two different multisets
/// share a hash with probability about \(2^{-64}\) per comparison. A multiset only reads nodes of
/// the arena that built it.
pub struct PMultiset<D>:
    var _len: Int = 0
    var _hash: U64 = U64(0)
    var _l: ?Id<PMultiset<D>> = None
    var _r: ?Id<PMultiset<D>> = None

    /// The empty multiset over `D`; it needs no arena node.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `PMultiset<0..10>.empty().len == 0`
    pub fn empty() -> PMultiset<D> = PMultiset()

    /// The number of elements, counting copies.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `PMultiset<0..10>.empty().is_empty`
    @get pub fn len(self) -> Int = _len

    /// Whether the multiset has no element.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `PMultiset<0..10>.empty().is_empty`
    @get pub fn is_empty(self) -> Bool = _len == 0

    /// \(O(1)\). The multiset read through the arena it lives in: a lent view that walks, counts
    /// and prints like any container. This is the walk of a `PMultiset`, since its nodes are in `ar`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var ar = PArena<PMultiset<0..10>>(); let s = PMultiset<0..10>.empty().with(mut ar, 3).with(mut ar, 1); s.within(ar).each().collect() == [1, 3]`
    pub fn within(self, ar: PArena<PMultiset<D>>) -> lends PMultisetWithin<D> = PMultisetWithin(ar, self)

    /// A new multiset with one more copy of `x`; `self` is unchanged.
    ///
    /// cost: \(O(\log \lvert D \rvert)\) time and arena nodes
    /// traps: when `x` is not in `D`
    /// example: `var ar = PArena<PMultiset<0..10>>(); let s = PMultiset<0..10>.empty().with(mut ar, 3).with(mut ar, 3); s.count(ar, 3) == 2`
    pub fn with(self, mut ar: PArena<PMultiset<D>>, x: D) -> PMultiset<D> =
        self.step(mut ar, x.ord, 1, 0, D.count)

    /// A new multiset with one copy of `x` removed, or `self` when `x` is absent; `self` is
    /// unchanged.
    ///
    /// cost: \(O(\log \lvert D \rvert)\) time and arena nodes
    /// traps: when `x` is not in `D`
    /// example: `var ar = PArena<PMultiset<0..10>>(); let s = PMultiset<0..10>.empty().with(mut ar, 3).with(mut ar, 3); s.without(mut ar, 3).count(ar, 3) == 1 and s.count(ar, 3) == 2`
    /// example: `var ar = PArena<PMultiset<0..10>>(); let s = PMultiset<0..10>.empty(); s.without(mut ar, 3) == s`
    pub fn without(self, mut ar: PArena<PMultiset<D>>, x: D) -> PMultiset<D>:
        if self.count(ar, x) == 0: return self
        self.step(mut ar, x.ord, -1, 0, D.count)

    /// The number of copies of `x`.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// traps: when `x` is not in `D`
    /// example: `var ar = PArena<PMultiset<0..10>>(); PMultiset<0..10>.empty().with(mut ar, 4).count(ar, 4) == 1`
    pub fn count(self, ar: PArena<PMultiset<D>>, x: D) -> Int:
        var (node, lo, hi) = (self, 0, D.count)
        while hi - lo > 1 and node._len > 0:
            let mid = lo + (hi - lo) / 2
            if x.ord < mid:
                node = kid(ar, node._l)
                hi = mid
            else:
                node = kid(ar, node._r)
                lo = mid
        node._len

    /// Compare the elements of `self` and `other` as sorted sequences, lexicographically: ascending,
    /// or largest first with `desc: true`. A proper prefix is smaller. Returns `-1`, `0` or `1`.
    /// It returns `0` only for equal multisets.
    ///
    /// The walk goes down the side where the two first differ by hash, so it reads one path.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// ties: `var ar = PArena<PMultiset<0..10>>(); let e = PMultiset<0..10>.empty(); let a = e.with(mut ar, 2).with(mut ar, 5); let b = e.with(mut ar, 5).with(mut ar, 2); a.cmp(ar, b) == 0`
    /// traps: never
    /// example: `var ar = PArena<PMultiset<0..10>>(); let e = PMultiset<0..10>.empty(); let a = e.with(mut ar, 1).with(mut ar, 3); let b = e.with(mut ar, 2); a.cmp(ar, b) == -1`
    /// example: `var ar = PArena<PMultiset<0..10>>(); let e = PMultiset<0..10>.empty(); let a = e.with(mut ar, 1).with(mut ar, 3); let b = e.with(mut ar, 2); a.cmp(ar, b, desc: true) == 1`
    /// example: `var ar = PArena<PMultiset<0..10>>(); let e = PMultiset<0..10>.empty(); let a = e.with(mut ar, 1); let b = a.with(mut ar, 2); a.cmp(ar, b) == -1`
    pub fn cmp(self, ar: PArena<PMultiset<D>>, other: PMultiset<D>, desc: Bool = false) -> Int:
        if self == other: return 0
        var (a, b, lo, hi, before) = (self, other, 0, D.count, 0)
        while hi - lo > 1:
            let mid = lo + (hi - lo) / 2
            let (al, ar_, bl, br) = (kid(ar, a._l), kid(ar, a._r), kid(ar, b._l), kid(ar, b._r))
            let (
                first_a,
                first_b,
                rest_a,
                rest_b,
            ) = if desc: (ar_, br, al, bl) else: (al, bl, ar_, br)
            if first_a == first_b:
                before += first_a._len
                (a, b) = (rest_a, rest_b)
                if desc: hi = mid else: lo = mid
            else:
                (a, b) = (first_a, first_b)
                if desc: lo = mid else: hi = mid
        // `a` and `b` are one key's counts, and every key visited before it agrees.
        if desc: return if a._len > b._len: 1 else: -1
        if a._len > b._len: return if other._len - before - b._len > 0: -1 else: 1
        if self._len - before - a._len > 0: 1 else: -1

    fn step(self, mut ar: PArena<PMultiset<D>>, k: Int, d: Int, lo: Int, hi: Int) -> PMultiset<D>:
        var n = self
        n._len += d
        let w = weight(k)
        if d > 0: n._hash += w else: n._hash -= w
        if hi - lo == 1: return n
        let mid = lo + (hi - lo) / 2
        if k < mid: n._l = Some(ar.alloc(kid(ar, _l).step(mut ar, k, d, lo, mid)))
        else: n._r = Some(ar.alloc(kid(ar, _r).step(mut ar, k, d, mid, hi)))
        n

impl<D> Seq<D> for PMultisetWithin<D>:
    /// Visit every element, one per copy, skipping empty subtrees.
    ///
    /// cost: \(O(\min(\mathit{len}, \lvert D \rvert) \log \lvert D \rvert + \mathit{len})\)
    /// empty: `var ar = PArena<PMultiset<0..10>>(); PMultiset<0..10>.empty().within(ar).each().collect() == []`
    /// order: ascending `D` order, each element repeated by its count
    /// traps: never
    /// example: `var ar = PArena<PMultiset<0..10>>(); let s = PMultiset<0..10>.empty().with(mut ar, 7).with(mut ar, 2).with(mut ar, 7); s.within(ar).each().collect() == [2, 7, 7]`
    fn iter(self) -> yields D:
        var stack = [(node: set, lo: 0, hi: D.count)]
        while let top = stack.pop():
            if top.node._len == 0: continue
            if top.hi - top.lo == 1:
                for _ in 0..top.node._len: yield D.nth(top.lo)
                continue
            let mid = top.lo + (top.hi - top.lo) / 2
            stack.push((node: kid(ar, top.node._r), lo: mid, hi: top.hi))
            stack.push((node: kid(ar, top.node._l), lo: top.lo, hi: mid))

/// A `PMultiset` read through its `PArena`, from `s.within(ar)`. It lends both, so it lives no
/// longer than either and copies nothing.
pub struct PMultisetWithin<D>(ar: PArena<PMultiset<D>>, set: PMultiset<D>):
    /// The number of elements, counting copies.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var ar = PArena<PMultiset<0..10>>(); let s = PMultiset<0..10>.empty().with(mut ar, 3).with(mut ar, 3); s.within(ar).len == 2`
    @get pub fn len(self) -> Int = set.len

    /// Whether the multiset has no element.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var ar = PArena<PMultiset<0..10>>(); PMultiset<0..10>.empty().within(ar).is_empty`
    @get pub fn is_empty(self) -> Bool = set.is_empty

impl<D> Eq for PMultiset<D>:
    fn eq(a, b) = a._len == b._len and a._hash == b._hash

// The node behind `id`, or the empty multiset for a missing child.
fn kid<D>(ar: PArena<PMultiset<D>>, id: ?Id<PMultiset<D>>) -> PMultiset<D>:
    if let i = id: return ar.get(i)
    PMultiset()

// The hash weight of one copy of key `k`: splitmix64 of the key, so weights are fixed and spread.
fn weight(k: Int) -> U64 = splitmix64(U64(k) ^ U64(0x2545F4914F6CDD1D))

@test
fn persistent_segtree_and_range_kth_match_brute():
    domain I = 1..=10
    let a = [I] _ => rand(-15, 15)
    var pst = PersistentSegTree<I, Int>(+, a)
    var versions = [pst.base]
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
                assert rk.count_le(rng, x) == s.count(y => y <= x)

@test
fn pmultiset_within_walks_and_shows():
    var ar = PArena<PMultiset<0..8>>()
    let e = PMultiset<0..8>.empty()
    let s = e.with(mut ar, 5).with(mut ar, 0).with(mut ar, 5).with(mut ar, 7)
    let t = s.without(mut ar, 5)
    assert s.within(ar).each().collect() == [0, 5, 5, 7] and "{s.within(ar)}" == "0 5 5 7"
    assert t.within(ar).each().collect() == [0, 5, 7] and t.within(ar).len == 3
    assert e.within(ar).is_empty and "{e.within(ar)}" == ""

impl Show for PMultisetWithin<D: Show>:
    fn show(v):
        var out = ""
        for x in v.each():
            if not out.is_empty: out += " "
            out += "{x}"
        out

impl Show for PMultiset<D>:
    /// The size the handle knows, as `multiset of len`; its elements live in the arena, so
    /// `s.within(ar)` prints them.
    ///
    /// example: `var ar = PArena<PMultiset<0..10>>(); let s = PMultiset<0..10>.empty().with(mut ar, 3).with(mut ar, 3); "{s}" == "multiset of 2"`
    fn show(s) = "multiset of {s.len}"

impl Show for PArena<T: Show>:
    /// The allocated values in allocation order, space-separated.
    ///
    /// example: `var a = PArena<Int>(); a.alloc(5); a.alloc(6); "{a}" == "5 6"`
    fn show(a):
        var out = ""
        for x in a._xs:
            if not out.is_empty: out += " "
            out += "{x}"
        out

impl<D: Domain, T: Sub> Versioned<D, T> for PersistentSegTree<D, T>:
    fn get(self, v: Version<PersistentSegTree<D, T>>, i: D) -> T = self.get(v, i)

impl<D: Domain, T, F> Versioned<D, T> for PersistentLazySegTree<D, T, F>:
    fn get(self, v: Version<PersistentLazySegTree<D, T, F>>, i: D) -> T = self.get(v, i)

impl<D: Domain, T> Versioned<D, T> for PersistentArray<D, T>:
    fn get(self, v: Version<PersistentArray<D, T>>, i: D) -> T = self.get(v, i)

impl Show for VersionWithin<S, D, T: Show>:
    /// The version's values in key order, space-separated.
    ///
    /// example: `domain P = 0..3; var t = PersistentLazySegTree<P, Int, Int>(Action<Int, Int>(apply: (d, x, span) => x + d * span), a: [P] i => i.ord + 1); let v = t.apply(t.base, P(1)..=P(2), 1); "{v.within(t)}" == "1 3 4"`
    fn show(w):
        var out = ""
        for x in w.each():
            if not out.is_empty: out += " "
            out += "{x}"
        out

impl Show for Version<S>:
    /// The root node the version names, as `version @root`.
    ///
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); "{t.base}" == "version @5"`
    fn show(v) = "version @{v.root}"

impl Show for PersistentSegTree<D, T: Sub + Show>:
    /// The base version's values in key order, space-separated.
    ///
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); "{t}" == "1 2 3"`
    fn show(t):
        var out = ""
        for i in 0..D.count:
            if not out.is_empty: out += " "
            out += "{t.get(t.base, D(i))}"
        out

impl Show for PersistentLazySegTree<D, T: Show, F>:
    /// The base version's values in key order, space-separated.
    ///
    /// example: `var t = PersistentLazySegTree(Action<Int, Int>(apply: (d, x, span) => x + d * span), a: [1, 2, 3]); "{t}" == "1 2 3"`
    fn show(t):
        var out = ""
        for i in 0..D.count:
            if not out.is_empty: out += " "
            out += "{t.get(t.base, D(i))}"
        out

impl Show for PersistentArray<D, T: Show>:
    /// The base version's values in key order, space-separated.
    ///
    /// example: `var t = PersistentArray(a: [4, 5, 6]); "{t}" == "4 5 6"`
    fn show(t):
        var out = ""
        for i in 0..D.count:
            if not out.is_empty: out += " "
            out += "{t.get(t.base, D(i))}"
        out

impl Show for RangeKth<D, T: Show>:
    /// The values it was built from, in key order, space-separated.
    ///
    /// example: `"{RangeKth([5, 1, 4, 2])}" == "5 1 4 2"`
    fn show(rk):
        var out = ""
        for i in 0..D.count:
            if not out.is_empty: out += " "
            out += "{rk.kth(D(i)..=D(i), 0)}"
        out
