//! Segment tree over a domain, for any associative operation.
use std.algebra.{Monoid}
use std.math.{gcd}

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

    /// A tree over `a`, or with the identity everywhere when no array is given. O(n).
    pub fn new<M>(op: M, a: ?[D]T = none) -> SegTree<D, T>:
        let m = Monoid<T>.of(op)
        let size = bit_ceil(D.K)
        var t = List.filled(2 * size, m.id)
        if let init = a:
            for i in D: t[size + i.to_idx] = init[i]
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

    pub fn get(self, i: D) -> T = t[size + i.to_idx]

    pub fn set(mut self, i: D, x: T):
        var p = i.to_idx + size
        t[p] = x
        while p > 1:
            p /= 2
            t[p] = m.op(t[2 * p], t[2 * p + 1])

    /// Replace the value at `i` with `f` of it, as `LazySegTree.update` does. O(log n).
    pub fn update(mut self, i: D, f: T -> T):
        self.set(i, f(self.get(i)))

    /// The fold of a range, left to right: `st.fold(l..=r)`, `st.fold(..i)`, `st.fold(i..)`.
    /// An empty range gives the identity.
    pub fn fold(self, r: Range<D>) -> T:
        var (l, h) = (r.lo.ord + size, r.hi.ord + size)
        var (a, b) = (m.id, m.id)
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
