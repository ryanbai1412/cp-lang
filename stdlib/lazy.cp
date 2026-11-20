//! Segment tree with range updates.
use std.algebra.{Monoid, Action}

/// Range update, range fold: `LazySegTree<V, Int>(AddMax, a)` (the tag type `F` is inferred
/// from the `Action`). The stock actions are `AddSum`, `AddMin`, `AddMax`, `AssignSum`,
/// `AssignMin`, and `AssignMax`; a custom action is
/// `Action(id: ..., compose: (new_f, old_f) => ..., apply: (f, val, span) => ..., over: op)`
/// where `compose` applies `old_f` first then `new_f` and `apply` updates a node fold
/// covering `span` leaves.
/// `apply`, `set`, `update`, `get`, `fold`, `max_right`, and `min_left` are O(log n); `all` is O(1).
/// Memory is 2 * bit_ceil(n) values and spans plus bit_ceil(n) pending tags.
pub struct LazySegTree<D, T, F>(act: Action<T, F>):
    t: List<T>
    lz: List<F>
    /// How many positions each node covers; the padding past the domain covers none.
    span: List<Int>
    size: Int
    log: Int

    /// A tree over `a`, or with the identity everywhere when no array is given. O(n).
    pub fn new(act: Action<T, F>, a: ?[D]T = none) -> LazySegTree<D, T, F>:
        let size = bit_ceil(D.K)
        var t = List.filled(2 * size, act.over.id)
        var span = List.filled(2 * size, 0)
        for i in 0..D.K: span[size + i] = 1
        if let init = a:
            for i in D: t[size + i.to_idx] = init[i]
        for p in (1..size).rev:
            t[p] = act.over.op(t[2 * p], t[2 * p + 1])
            span[p] = span[2 * p] + span[2 * p + 1]
        LazySegTree(act, t, List.filled(size, act.id), span, size, ctz(size))

    fn pull(mut self, p: Int):
        t[p] = act.over.op(t[2 * p], t[2 * p + 1])

    fn put(mut self, p: Int, f: F):
        if span[p] == 0: return
        t[p] = act.apply(f, t[p], span[p])
        if p < size: lz[p] = act.compose(f, lz[p])

    fn push(mut self, p: Int):
        self.put(2 * p, lz[p])
        self.put(2 * p + 1, lz[p])
        lz[p] = act.id

    fn open(mut self, l: Int, h: Int):
        for i in (1..=log).rev:
            if ((l >> i) << i) != l: self.push(l >> i)
            if ((h >> i) << i) != h: self.push((h - 1) >> i)

    pub fn set(mut self, i: D, x: T):
        let p = i.to_idx + size
        for k in (1..=log).rev: self.push(p >> k)
        t[p] = x
        for k in 1..=log: self.pull(p >> k)

    /// Push down to leaf `i`, replace its value with `f(cur)`, and pull back up in one O(log n) traversal.
    pub fn update(mut self, i: D, f: T -> T):
        let p = i.to_idx + size
        for k in (1..=log).rev: self.push(p >> k)
        t[p] = f(t[p])
        for k in 1..=log: self.pull(p >> k)

    pub fn get(mut self, i: D) -> T:
        let p = i.to_idx + size
        for k in (1..=log).rev: self.push(p >> k)
        t[p]

    /// The fold of every position. O(1).
    pub @get fn all(self) -> T = t[1]

    /// The operation the tree folds with, as its `(id, op)` pair.
    pub @get fn monoid(self) -> Monoid<T> = act.over

    /// The fold of a range, left to right: `st.fold(l..=r)`, `st.fold(..i)`, `st.fold(i..)`.
    /// An empty range gives the identity.
    pub fn fold(mut self, r: Range<D>) -> T:
        let m = act.over
        if r.is_empty: return m.id
        var (l, h) = (r.lo.ord + size, r.hi.ord + size)
        self.open(l, h)
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

    /// Every position in the range takes the update `f`.
    pub fn apply(mut self, r: Range<D>, f: F):
        if r.is_empty: return
        let (l0, h0) = (r.lo.ord + size, r.hi.ord + size)
        self.open(l0, h0)
        var (l, h) = (l0, h0)
        while l < h:
            if l % 2 == 1:
                self.put(l, f)
                l += 1
            if h % 2 == 1:
                h -= 1
                self.put(h, f)
            l /= 2
            h /= 2
        for i in 1..=log:
            if ((l0 >> i) << i) != l0: self.pull(l0 >> i)
            if ((h0 >> i) << i) != h0: self.pull((h0 - 1) >> i)

    /// The furthest boundary `b` right of `from` such that `ok(fold(from..b))`.
    /// `ok` must hold for the identity and switch from true to false at most once.
    pub fn max_right(mut self, from: Bounds<D>, ok: T -> Bool) -> Bounds<D>:
        let m = act.over
        if from.ord == D.K: return from
        var l = from.ord + size
        for k in (1..=log).rev: self.push(l >> k)
        var acc = m.id
        while true:
            while l % 2 == 0: l /= 2
            if not ok(m.op(acc, t[l])):
                while l < size:
                    self.push(l)
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
    pub fn min_left(mut self, to: Bounds<D>, ok: T -> Bool) -> Bounds<D>:
        let m = act.over
        if to.ord == 0: return to
        var h = to.ord + size
        for k in (1..=log).rev: self.push((h - 1) >> k)
        var acc = m.id
        while true:
            h -= 1
            while h > 1 and h % 2 == 1: h /= 2
            if not ok(m.op(t[h], acc)):
                while h < size:
                    self.push(h)
                    h = 2 * h + 1
                    if ok(m.op(t[h], acc)):
                        acc = m.op(t[h], acc)
                        h -= 1
                return D.bounds.nth(h + 1 - size)
            acc = m.op(t[h], acc)
            if (h & -h) == h: break
        D.bounds.nth(0)

/// Add to every position, fold with `+`.
pub const AddSum = Action<Int, Int>(0, +, (f, x, len) => x + f * len, over: +)
/// Add to every position, fold with `min`.
pub const AddMin = Action<Int, Int>(0, +, (f, x, len) => x + f, over: min)
/// Add to every position, fold with `max`.
pub const AddMax = Action<Int, Int>(0, +, (f, x, len) => x + f, over: max)
/// Overwrite every position, fold with `+`. The pending update is a `?Int`: `none` is "nothing pending".
pub const AssignSum = Action<Int, ?Int>(none, (f, g) => f ?? g, (f, x, len) => if f == none: x else: f! * len, over: +)
/// Overwrite every position, fold with `min`.
pub const AssignMin = Action<Int, ?Int>(none, (f, g) => f ?? g, (f, x, len) => f ?? x, over: min)
/// Overwrite every position, fold with `max`.
pub const AssignMax = Action<Int, ?Int>(none, (f, g) => f ?? g, (f, x, len) => f ?? x, over: max)

fn brute<D>(a: [D]Int, r: Range<D>, pick: Int) -> Int:
    if pick == 0: return sum(a[v] for v in r)
    if pick == 1: return min(a[v] for v in r) ?? Int.MAX
    max(a[v] for v in r) ?? Int.MIN

@test
fn the_stock_actions_match_brute_force():
    domain V = 1..=10
    for (k, act) in [(0, AddSum), (1, AddMin), (2, AddMax), (0, AssignSum), (1, AssignMin), (2, AssignMax)]:
        let assign = act.id == none
        var a = [V] v => rand(-9, 9)
        var st = LazySegTree<V, Int>(act, a)
        for _ in 0..60:
            let l = V.nth(rand(0, 9))
            let r = V.nth(rand(l.to_idx, 9))
            if rand(0, 1) == 0:
                let x = rand(-5, 5)
                for v in l..=r: a[v] = if assign: x else: a[v] + x
                st.apply(l..=r, x)
            else:
                assert st.fold(l..=r) == brute(a, l..=r, k)
                assert st.fold(..l) == brute(a, ..l, k)
                assert st.fold(l..) == brute(a, l.., k)
        for v in V: assert st.get(v) == a[v]
        assert st.all == brute(a, V.first..=V.last, k)

@test
fn stock_actions_obey_their_laws():
    assert laws_hold(AddSum, T: [-3, 0, 8], F: [-4, 0, 6])
    assert laws_hold(AddMin, T: [-3, 0, 8], F: [-4, 0, 6])
    assert laws_hold(AddMax, T: [-3, 0, 8], F: [-4, 0, 6])
    assert laws_hold(AssignSum, T: [-3, 0, 8], F: [none, -4, 0, 6])
    assert laws_hold(AssignMin, T: [-3, 0, 8], F: [none, -4, 0, 6])
    assert laws_hold(AssignMax, T: [-3, 0, 8], F: [none, -4, 0, 6])

@test
fn set_and_an_empty_tree():
    domain V = 0..6
    var st = LazySegTree<V, Int>(AddSum)
    st.apply(V(1)..=V(4), 3)
    st.apply(V(3)..=V(5), 10)
    assert st.fold(V(0)..=V(5)) == 4 * 3 + 3 * 10
    assert st.get(V(4)) == 13
    st.set(V(4), 100)
    assert st.fold(V(3)..=V(5)) == 13 + 100 + 10
    assert st.fold(V(2)..V(2)) == 0

@test
fn padding_never_takes_an_update():
    // 3 positions in a tree of 4 leaves: the spare leaf holds the identity of max (Int.MIN)
    // or min (Int.MAX) and must stay out of every add, or `Int.MIN - 1` / `Int.MAX + 1` would overflow.
    domain V = 0..3
    var st = LazySegTree<V, Int>(AddMax, [V] v => 0)
    st.apply(V(0)..=V(2), -1)
    st.apply(V(0)..=V(1), 1)
    assert st.fold(V(0)..=V(2)) == 0 and st.get(V(2)) == -1
    assert st.fold(V(1)..V(1)) == Int.MIN

    var mn = LazySegTree<V, Int>(AddMin, [V] v => 0)
    mn.apply(V(0)..=V(2), 1)
    mn.apply(V(0)..=V(1), -1)
    assert mn.fold(V(0)..=V(2)) == 0 and mn.get(V(2)) == 1
    assert mn.fold(V(1)..V(1)) == Int.MAX and mn.fold(V(2).r..) == Int.MAX
    assert mn.fold(V(1)..) == 0 and mn.all == 0

@test
fn an_action_of_your_own():
    // values are (best, how many positions hold it); the update adds to every position
    let count_max = Monoid<(Int, Int)>((Int.MIN, 0), (a, b) => if a.0 > b.0: a else: if b.0 > a.0: b else: (a.0, a.1 + b.1))
    let add = Action<(Int, Int), Int>(0, +, (f, x, len) => (x.0 + f, x.1), over: count_max)
    domain V = 0..8
    var a = [V] v => 0
    var st = LazySegTree<V, (Int, Int)>(add, [V] v => (0, 1))
    for _ in 0..60:
        let l = V.nth(rand(0, 7))
        let r = V.nth(rand(l.to_idx, 7))
        let x = rand(-3, 3)
        for v in l..=r: a[v] += x
        st.apply(l..=r, x)
        let top = max(a[v] for v in V)!
        assert st.all == (top, count(a[v] == top for v in V))

@test
fn max_right_and_min_left_see_pending_updates():
    domain V = 0..7
    var a = [V] v => v.to_idx + 1
    var st = LazySegTree<V, Int>(AddSum, a)
    for round in 0..6:
        let l = V.nth(rand(0, 6))
        let r = V.nth(rand(l.to_idx, 6))
        let x = rand(0, 4)
        for v in l..=r: a[v] += x
        st.apply(l..=r, x)
        for from in 0..=7:
            for cap in [0, 3, 9, 20, 60]:
                var (want, acc) = (from, 0)
                while want < 7 and acc + a[V.nth(want)] <= cap:
                    acc += a[V.nth(want)]
                    want += 1
                assert st.max_right(V.bounds.nth(from), s => s <= cap).ord == want
                var (low, back) = (from, 0)
                while low > 0 and back + a[V.nth(low - 1)] <= cap:
                    back += a[V.nth(low - 1)]
                    low -= 1
                assert st.min_left(V.bounds.nth(from), s => s <= cap).ord == low

fn ccc_compose(f: (?Int, ?Int, Int), g: (?Int, ?Int, Int)) -> (?Int, ?Int, Int):
    let nb = f.0 ?? g.0
    if f.1 != none: return (nb, f.1, 0)
    if g.1 != none: return (nb, g.1! + f.2, 0)
    (nb, none, g.2 + f.2)

fn ccc_apply(f: (?Int, ?Int, Int), x: (Int, Int, Int), len: Int) -> (Int, Int, Int):
    let b_min = f.0 ?? x.1
    if f.1 != none:
        let a_val = b_min + f.1!
        return (a_val, b_min, a_val * len)
    (x.0 + f.2, b_min, x.2 + f.2 * len)

@test
fn ccc02s4hard_dual_field_cross_overwrite_and_interleaved_point_set():
    // Conformance test pinned in v2.1: each position holds (a, b) where a range update can
    // overwrite `b` and cross-overwrite `a = b + d` or add to `a`, interleaved with point
    // `.set` calls that must push pending tags root-to-leaf first.
    let over = Monoid<(Int, Int, Int)>((Int.MAX, Int.MAX, 0), (x, y) => (min(x.0, y.0), min(x.1, y.1), x.2 + y.2))
    let act = Action<(Int, Int, Int), (?Int, ?Int, Int)>((none, none, 0), ccc_compose, ccc_apply, over: over)
    domain V = 0..11
    var a = [V] v => rand(-10, 10)
    var b = [V] v => rand(-10, 10)
    var st = LazySegTree<V, (Int, Int, Int)>(act, [V] v => (a[v], b[v], a[v]))
    for _ in 0..120:
        let kind = rand(0, 3)
        if kind == 0:
            // range add to a
            let l = V.nth(rand(0, 10))
            let r = V.nth(rand(l.to_idx, 10))
            let d = rand(-5, 5)
            for v in l..=r: a[v] += d
            st.apply(l..=r, (none, none, d))
        elif kind == 1:
            // range cross-overwrite: set b = vb and a = b + d
            let l = V.nth(rand(0, 10))
            let r = V.nth(rand(l.to_idx, 10))
            let vb = rand(-8, 8)
            let d = rand(-4, 4)
            for v in l..=r:
                b[v] = vb
                a[v] = vb + d
            st.apply(l..=r, (vb, d, 0))
        elif kind == 2:
            // interleaved point set while lazy tags are still pending above the leaf
            let p = V.nth(rand(0, 10))
            let na = rand(-15, 15)
            let nb = rand(-15, 15)
            a[p] = na
            b[p] = nb
            st.set(p, (na, nb, na))
        else:
            let l = V.nth(rand(0, 10))
            let r = V.nth(rand(l.to_idx, 10))
            let want = (min(a[v] for v in l..=r)!, min(b[v] for v in l..=r)!, sum(a[v] for v in l..=r))
            assert st.fold(l..=r) == want
    for v in V:
        assert st.get(v) == (a[v], b[v], a[v])

@test
fn update_pushes_modifies_and_pulls_in_one_pass():
    // Single-leaf callback `update(i, f)` must see all pending ancestor tags before modifying
    // the leaf and must never double-apply those tags afterward.
    domain V = 1..=9
    var a = [V] v => rand(-10, 10)
    var st = LazySegTree<V, Int>(AddSum, a)
    for _ in 0..80:
        let kind = rand(0, 3)
        if kind == 0:
            let l = V.nth(rand(0, 8))
            let r = V.nth(rand(l.to_idx, 8))
            let d = rand(-7, 7)
            for v in l..=r: a[v] += d
            st.apply(l..=r, d)
        elif kind == 1:
            let i = V.nth(rand(0, 8))
            let add = rand(-5, 5)
            a[i] = a[i] * 2 + add
            st.update(i, cur => cur * 2 + add)
        elif kind == 2:
            let i = V.nth(rand(0, 8))
            let x = rand(-20, 20)
            a[i] = x
            st.set(i, x)
        else:
            let l = V.nth(rand(0, 8))
            let r = V.nth(rand(l.to_idx, 8))
            assert st.fold(l..=r) == sum(a[v] for v in l..=r)
            assert st.all == sum(a[v] for v in V)
    for v in V: assert st.get(v) == a[v]

@test
fn custom_clamp_action_with_update():
    // Non-commutative clamp tag `(lo, hi)`: clamps values into `[lo, hi]` over `max`.
    let clamp = Action<Int, (Int, Int)>(
        id: (Int.MIN, Int.MAX),
        compose: (new_f, old_f) => (min(max(old_f.0, new_f.0), new_f.1), max(min(old_f.1, new_f.1), new_f.0)),
        apply: (f, val, span) => min(max(val, f.0), f.1),
        over: max,
    )
    domain V = 0..6
    var a = [V] v => [3, -4, 9, 0, 5, -2][v.to_idx]
    var st = LazySegTree<V, Int>(clamp, a)
    for _ in 0..50:
        let l = V.nth(rand(0, 5))
        let r = V.nth(rand(l.to_idx, 5))
        let lo = rand(-6, 4)
        let hi = rand(lo, 8)
        for v in l..=r: a[v] = min(max(a[v], lo), hi)
        st.apply(l..=r, (lo, hi))
        let i = V.nth(rand(0, 5))
        a[i] = a[i] + 1
        st.update(i, x => x + 1)
        assert st.fold(l..=r) == max(a[v] for v in l..=r)!
        assert st.all == max(a[v] for v in V)!
    for v in V: assert st.get(v) == a[v]
