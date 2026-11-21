//! Segment tree over a domain, for any associative operation.
use std.algebra.{Monoid}
use std.math.{gcd}
use std.seq.{ends}

/// Point update and range fold. The operation is a bare operator (`+`, `*`, `^`, `&`, `|`,
/// `min`, `max`, `gcd`) or your own `Monoid(id, op)`: `SegTree<V, Int>(max, a)`,
/// `SegTree<V, Int>(+)`, `SegTree<V, Run>(Monoid(Run.empty, Run.join), a)`.
/// `set` and `fold` are O(log n), and so are `max_right` and `min_left`; memory is
/// 2 * bit_ceil(n) values. The fold keeps the order of the positions, so
/// non-commutative operations work.
pub struct SegTree<D, T>(m: Monoid<T>):
    t: List<T>
    size: Int
    /// What each position holds, for a tree built with `of`.
    source: ?(D -> T)

    /// A tree over `a`, filled with `fill`, or with the identity everywhere. O(n).
    pub fn new<M>(op: M, a: ?[D]T = none, fill: ?T = none) -> SegTree<D, T>:
        let m = Monoid<T>.of(op)
        let size = bit_ceil(D.K)
        var t = List.filled(2 * size, m.id)
        if let init = a:
            assert fill == none, "pass either `a` or `fill`, not both"
            for i in D: t[size + i.to_idx] = init[i]
            for p in (1..size).rev: t[p] = m.op(t[2 * p], t[2 * p + 1])
        elif let v = fill:
            for i in 0..D.K: t[size + i] = v
            for p in (1..size).rev: t[p] = m.op(t[2 * p], t[2 * p + 1])
        SegTree(m, t, size, none)

    /// A tree whose position `i` holds `f(i)`. When something `f` reads changes, `refresh(i)`
    /// brings position `i` up to date, so the formula is written once, not at every update. O(n).
    pub fn of<M>(op: M, f: D -> T) -> SegTree<D, T>:
        var s = SegTree<D, T>.new(op, [D] i => f(i))
        s.source = f
        s

    /// Recompute position `i` from the formula given to `of`. O(log n).
    pub fn refresh(mut self, i: D):
        assert source != none  // only a tree built with `of` has a formula
        let f = source!
        self.set(i, f(i))

    /// Read the leaf value at position `i`. O(1).
    pub fn get(self, i: D) -> T = t[size + i.to_idx]

    /// Assign leaf `i` to `x` and pull ancestors. O(log |D|).
    pub fn set(mut self, i: D, x: T):
        t[size + i.to_idx] = x
        self.pull(i)

    /// Leaf `i` as a writable place: `st.at(i) += x`, `st.at(i) max= x`. The ancestors
    /// are pulled when the write ends, so each use is O(log |D|); read with `get`.
    pub fn at(mut self, i: D) lends mut T:
        lend mut t[size + i.to_idx]
        self.pull(i)

    fn pull(mut self, i: D):
        var p = i.to_idx + size
        unchecked:
            while p > 1:
                p /= 2
                t[p] = m.op(t[2 * p], t[2 * p + 1])

    /// Replace the value at `i` with `f` of it, as `LazySegTree.update` does. O(log n).
    pub fn update(mut self, i: D, f: T -> T):
        self.set(i, f(self.get(i)))

    /// The fold of a range, left to right: `st.fold(l..=r)`, `st.fold(..i)`, `st.fold(i..)`.
    /// An empty range gives the identity.
    pub fn fold(self, r: Range<D>) -> T:
        let (lo, hi) = r.ends
        var (l, h) = (lo + size, hi + size)
        var (a, b) = (m.id, m.id)
        unchecked:
            while l < h:
                if l % 2 == 1:
                    a = m.op(a, t[l])
                    l += 1
                if h % 2 == 1:
                    h -= 1
                    b = m.op(t[h], b)
                l /= 2
                h /= 2
        m.op(a, b)

    /// The fold of the whole array. O(1).
    pub @get fn all(self) -> T = t[1]

    /// The operation the tree folds with, as its `(id, op)` pair.
    pub @get fn monoid(self) -> Monoid<T> = m

    /// The furthest boundary `b` right of `from` such that `ok(fold(from..b))`.
    /// `ok` must hold for the identity and switch from true to false at most once.
    pub fn max_right(self, from: Bounds<D>, ok: T -> Bool) -> Bounds<D>:
        var l = from.ord + size
        var acc = m.id
        if from.ord == D.K: return from
        while true:
            while l % 2 == 0: l /= 2
            if not ok(m.op(acc, t[l])):
                while l < size:
                    l *= 2
                    if ok(m.op(acc, t[l])):
                        acc = m.op(acc, t[l])
                        l += 1
                return D.bounds.nth(min(l - size, D.K))
            acc = m.op(acc, t[l])
            l += 1
            if (l & -l) == l: break
        D.bounds.nth(D.K)

    /// The furthest boundary `b` left of `to` such that `ok(fold(b..to))`.
    pub fn min_left(self, to: Bounds<D>, ok: T -> Bool) -> Bounds<D>:
        var h = to.ord + size
        var acc = m.id
        if to.ord == 0: return to
        while true:
            h -= 1
            while h > 1 and h % 2 == 1: h /= 2
            if not ok(m.op(t[h], acc)):
                while h < size:
                    h = 2 * h + 1
                    if ok(m.op(t[h], acc)):
                        acc = m.op(t[h], acc)
                        h -= 1
                return D.bounds.nth(h + 1 - size)
            acc = m.op(t[h], acc)
            if (h & -h) == h: break
        D.bounds.nth(0)

@test
fn folds_match_a_plain_array():
    domain V = 1..=11
    var a = [V] v => rand(-20, 20)
    var st = SegTree<V, Int>(max, a)
    for _ in 0..80:
        let i = V.nth(rand(0, 10))
        a[i] = rand(-20, 20)
        st.set(i, a[i])
        let l = V.nth(rand(0, 10))
        let r = V.nth(rand(l.to_idx, 10))
        assert st.fold(l..=r) == max(a[v] for v in l..=r)
    assert st.all == max(a[v] for v in V)
    assert st.fold(V(3)..V(3)) == Int.MIN
    assert st.fold(..V(4)) == max(a[v] for v in V if v < V(4))
    assert st.fold(V(4)..) == max(a[v] for v in V if v >= V(4))

@test
fn at_is_a_writable_leaf():
    domain V = 1..=9
    var a = [V] v => rand(-20, 20)
    var st = SegTree<V, Int>(+, a)
    for _ in 0..60:
        let i = V.nth(rand(0, 8))
        let x = rand(-9, 9)
        if rand(0, 1) == 0:
            a[i] += x
            st.at(i) += x
        else:
            a[i] = max(a[i], x)
            st.at(i) max= x
        let l = V.nth(rand(0, 8))
        let r = V.nth(rand(l.to_idx, 8))
        assert st.fold(l..=r) == sum(a[v] for v in l..=r)
    assert st.all == sum(a)

@test
fn every_bare_operator_folds():
    domain V = 0..6
    let a = [V] v => [12, 18, 6, 30, 7, 9][v.to_idx]
    assert SegTree<V, Int>(+, a).all == 82
    assert SegTree<V, Int>(min, a).all == 6
    assert SegTree<V, Int>(gcd, a).fold(V(0)..=V(3)) == 6
    assert SegTree<V, Int>(^, a).fold(V(0)..=V(1)) == 12 ^ 18
    assert SegTree<V, Int>(&, a).fold(V(0)..=V(1)) == 12 & 18
    assert SegTree<V, Int>(|, a).fold(V(0)..=V(1)) == 12 | 18
    assert SegTree<V, Int>(*, a).fold(V(4)..=V(5)) == 63
    // over ?Int, `none` is "no value": the identity of max, and what an empty range folds to
    var best = SegTree<V, ?Int>(max)
    assert best.all == none
    best.set(V(2), 5)
    best.set(V(4), -1)
    assert best.all == 5 and best.fold(V(3)..) == -1 and best.fold(..V(2)) == none

@test
fn of_and_refresh_follow_the_formula():
    domain V = 1..=9
    var a = [V] v => rand(-20, 20)
    var st = SegTree<V, Int>.of(min, v => a[v] + v.ord)
    for _ in 0..60:
        let i = V.nth(rand(0, 8))
        a[i] = rand(-20, 20)
        st.refresh(i)
        assert st.all == min(a[v] + v.ord for v in V)
        assert st.get(i) == a[i] + i.ord

@test
fn keeps_the_order_for_non_commutative_monoids():
    domain P = 0..5
    let cat = Monoid<Str>("", (a, b) => a + b)
    var st = SegTree<P, Str>(cat)
    for (i, w) in [(0, "a"), (1, "b"), (2, "c"), (3, "d"), (4, "e")]: st.set(P(i), w)
    assert st.fold(P(1)..=P(3)) == "bcd"
    assert st.all == "abcde"

@test
fn max_right_and_min_left_binary_search():
    domain V = 0..7
    let a = [V] v => v.to_idx + 1
    let st = SegTree<V, Int>(+, a)
    for l in 0..=7:
        for cap in 0..30:
            let from = V.bounds.nth(l)
            var want = l
            var acc = 0
            while want < 7 and acc + (want + 1) <= cap:
                acc += want + 1
                want += 1
            assert st.max_right(from, s => s <= cap).ord == want
    for h in 0..=7:
        for cap in 0..30:
            var want = h
            var acc = 0
            while want > 0 and acc + want <= cap:
                acc += want
                want -= 1
            assert st.min_left(V.bounds.nth(h), s => s <= cap).ord == want

@test
fn update_changes_one_value_in_place():
    domain V = 0..6
    var st = SegTree<V, Int>(+, [V] v => v.to_idx)
    st.update(V(2), x => x + 10)
    st.update(V(5), x => x * 2)
    assert st.get(V(2)) == 12 and st.get(V(5)) == 10 and st.all == 15 + 10 + 5

@test
fn fill_option_initializes_leaves_without_array():
    domain V = 1..=5
    var st = SegTree<V, Int>(max, fill: 0)
    assert st.all == 0 and st.get(V(3)) == 0
    st.set(V(2), -4)
    assert st.fold(V(2)..=V(2)) == -4 and st.all == 0
    let summed = SegTree<V, Int>(+, fill: 7)
    assert summed.all == 35 and summed.fold(V(2)..=V(4)) == 21

/// Static merge-sort tree for orthogonal 2D range counting over `[D]T`.
/// Build in O(n log n) time and memory; answer value-range counts over any `Range<D>` in O(log^2 n).
pub struct MergeSortTree<D, T>:
    data: List<T>
    beg: List<0..=2_000_000_000>
    fin: List<0..=2_000_000_000>
    size: Int

    /// Build from an array `a: [D]T`. O(n log n).
    pub fn new(a: [D]T) -> MergeSortTree<D, T>:
        let size = bit_ceil(D.K)
        var data: List<T> = []
        var beg: List<0..=2_000_000_000> = List.filled(2 * size, 0)
        var fin: List<0..=2_000_000_000> = List.filled(2 * size, 0)
        unchecked:
            for i in D:
                let p = size + i.to_idx
                beg[p] = data.len
                data.push(a[i])
                fin[p] = data.len
            for p in (1..size).rev:
                let (li, le) = (beg[2 * p], fin[2 * p])
                let (ri, re) = (beg[2 * p + 1], fin[2 * p + 1])
                beg[p] = data.len
                var (i, j) = (li, ri)
                while i < le and j < re:
                    if data[i] <= data[j]:
                        data.push(data[i])
                        i += 1
                    else:
                        data.push(data[j])
                        j += 1
                while i < le:
                    data.push(data[i])
                    i += 1
                while j < re:
                    data.push(data[j])
                    j += 1
                fin[p] = data.len
        MergeSortTree(data, beg, fin, size)

    fn lower_idx(self, u: Int, x: T) -> Int:
        unchecked:
            let base = beg[u]
            var (lo, hi) = (base, fin[u])
            while lo < hi:
                let mid = (lo + hi) / 2
                if data[mid] < x: lo = mid + 1
                else: hi = mid
            return lo - base

    fn upper_idx(self, u: Int, x: T) -> Int:
        unchecked:
            let base = beg[u]
            var (lo, hi) = (base, fin[u])
            while lo < hi:
                let mid = (lo + hi) / 2
                if data[mid] <= x: lo = mid + 1
                else: hi = mid
            return lo - base

    /// Count elements in `r` strictly less than `x`. O(log^2 n).
    pub fn count_lt(self, r: Range<D>, x: T) -> Int:
        let (lo, hi) = r.ends
        var (l, h, ans) = (lo + size, hi + size, 0)
        unchecked:
            while l < h:
                if l % 2 == 1:
                    ans += self.lower_idx(l, x)
                    l += 1
                if h % 2 == 1:
                    h -= 1
                    ans += self.lower_idx(h, x)
                l /= 2
                h /= 2
        ans

    /// Count elements in `r` less than or equal to `x`. O(log^2 n).
    pub fn count_le(self, r: Range<D>, x: T) -> Int:
        let (lo, hi) = r.ends
        var (l, h, ans) = (lo + size, hi + size, 0)
        unchecked:
            while l < h:
                if l % 2 == 1:
                    ans += self.upper_idx(l, x)
                    l += 1
                if h % 2 == 1:
                    h -= 1
                    ans += self.upper_idx(h, x)
                l /= 2
                h /= 2
        ans

    /// Count elements in `r` within `[lo_v, hi_v]` inclusive. O(log^2 n).
    pub fn count_in(self, r: Range<D>, lo_v: T, hi_v: T) -> Int:
        if lo_v > hi_v: return 0
        self.count_le(r, hi_v) - self.count_lt(r, lo_v)

/// Dynamic (coordinate-sparse) segment tree over integer coordinates `[lo, hi)`.
/// Allocates nodes on demand in an arena. Point update and range fold run in O(log(hi - lo)).
pub struct SparseSegTree<T>(m: Monoid<T>, lo: Int, hi: Int):
    left: List<-1..=2_000_000_000>
    right: List<-1..=2_000_000_000>
    val: List<T>
    root: Int

    /// Empty sparse segment tree over half-open coordinates `[lo, hi)` (every cell starts at `m.id`). O(1).
    pub fn new<M>(op: M, lo: Int, hi: Int) -> SparseSegTree<T>:
        assert lo < hi, "SparseSegTree needs lo < hi"
        let m = Monoid<T>.of(op)
        SparseSegTree(m, lo, hi, [-1], [-1], [m.id], 0)

    fn new_node(mut self) -> Int:
        let id = val.len
        left.push(-1)
        right.push(-1)
        val.push(m.id)
        id

    fn node_val(self, u: Int) -> T = if u == -1: m.id else: val[u]

    fn set_rec(mut self, u: Int, nl: Int, nr: Int, x: Int, v: T):
        if nr - nl == 1:
            val[u] = v
            return
        let mid = nl + (nr - nl) / 2
        if x < mid:
            if left[u] == -1: left[u] = self.new_node()
            self.set_rec(left[u], nl, mid, x, v)
        else:
            if right[u] == -1: right[u] = self.new_node()
            self.set_rec(right[u], mid, nr, x, v)
        val[u] = m.op(self.node_val(left[u]), self.node_val(right[u]))

    /// Set coordinate `x` (in `[lo, hi)`) to `v`. O(log(hi - lo)).
    pub fn set(mut self, x: Int, v: T):
        assert lo <= x and x < hi, "SparseSegTree coordinate out of range"
        self.set_rec(root, lo, hi, x, v)

    fn update_rec(mut self, u: Int, nl: Int, nr: Int, x: Int, f: T -> T):
        if nr - nl == 1:
            val[u] = f(val[u])
            return
        let mid = nl + (nr - nl) / 2
        if x < mid:
            if left[u] == -1: left[u] = self.new_node()
            self.update_rec(left[u], nl, mid, x, f)
        else:
            if right[u] == -1: right[u] = self.new_node()
            self.update_rec(right[u], mid, nr, x, f)
        val[u] = m.op(self.node_val(left[u]), self.node_val(right[u]))

    /// Replace coordinate `x` with `f(current)`. O(log(hi - lo)).
    pub fn update(mut self, x: Int, f: T -> T):
        assert lo <= x and x < hi, "SparseSegTree coordinate out of range"
        self.update_rec(root, lo, hi, x, f)

    /// Value stored at coordinate `x`. O(log(hi - lo)).
    pub fn get(self, x: Int) -> T:
        assert lo <= x and x < hi, "SparseSegTree coordinate out of range"
        var (u, nl, nr) = (root, lo, hi)
        while u != -1 and nr - nl > 1:
            let mid = nl + (nr - nl) / 2
            if x < mid:
                u = left[u]
                nr = mid
            else:
                u = right[u]
                nl = mid
        self.node_val(u)

    fn fold_rec(self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int) -> T:
        if u == -1 or qr <= nl or nr <= ql: return m.id
        if ql <= nl and nr <= qr: return val[u]
        let mid = nl + (nr - nl) / 2
        m.op(self.fold_rec(left[u], nl, mid, ql, qr), self.fold_rec(right[u], mid, nr, ql, qr))

    /// Fold over half-open coordinate range `[ql, qr)`. O(log(hi - lo)).
    pub fn fold(self, ql: Int, qr: Int) -> T:
        if ql >= qr: return m.id
        self.fold_rec(root, lo, hi, max(lo, ql), min(hi, qr))

    /// Fold over the entire `[lo, hi)` coordinate range. O(1).
    pub @get fn all(self) -> T = val[root]

@test
fn merge_sort_and_sparse_segtree_match_brute():
    domain I = 1..=14
    let a = [I] _ => rand(-10, 10)
    let mst = MergeSortTree<I, Int>(a)
    for l in 1..=14:
        for r in l..=14:
            let rng = I(l)..=I(r)
            for v in [-11, -5, 0, 5, 11]:
                let want_lt = count(a[i] < v for i in rng)
                let want_le = count(a[i] <= v for i in rng)
                assert mst.count_lt(rng, v) == want_lt
                assert mst.count_le(rng, v) == want_le
            assert mst.count_in(rng, -3, 4) == count(-3 <= a[i] and a[i] <= 4 for i in rng)

    var sst = SparseSegTree<Int>(+, -1_000_000_000, 1_000_000_001)
    var ref = Map<Int, Int>()
    let coords = [-1_000_000_000, -500, -1, 0, 1, 42, 999_999_999, 1_000_000_000]
    for _ in 0..80:
        let x = coords[rand(0, coords.len - 1)]
        let d = rand(-20, 20)
        sst.update(x, v => v + d)
        ref[x] += d
        assert sst.get(x) == ref[x]
    for ql in coords:
        for qr in coords:
            let want = sum(v for (k, v) in ref.items if ql <= k and k < qr)
            assert sst.fold(ql, qr) == want
    assert sst.all == sum(v for (_, v) in ref.items)
