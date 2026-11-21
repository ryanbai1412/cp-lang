//! Segment tree with range updates.
use std.algebra.{Monoid, Action}
use std.rand.Rng
use std.seq.{ends}

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

    /// A tree over `a`, filled with `fill`, or with the identity everywhere. O(n).
    pub fn new(act: Action<T, F>, a: ?[D]T = none, fill: ?T = none) -> LazySegTree<D, T, F>:
        let size = bit_ceil(D.K)
        var t = List.filled(2 * size, act.over.id)
        var span = List.filled(2 * size, 0)
        for i in 0..D.K: span[size + i] = 1
        if let init = a:
            assert fill == none, "pass either `a` or `fill`, not both"
            for i in D: t[size + i.to_idx] = init[i]
        elif let v = fill:
            for i in 0..D.K: t[size + i] = v
        for p in (1..size).rev:
            t[p] = act.over.op(t[2 * p], t[2 * p + 1])
            span[p] = span[2 * p] + span[2 * p + 1]
        LazySegTree(act, t, List.filled(size, act.id), span, size, ctz(size))

    fn pull(mut self, p: Int):
        unchecked:
            t[p] = act.over.op(t[2 * p], t[2 * p + 1])

    fn put(mut self, p: Int, f: F):
        unchecked:
            if span[p] == 0: return
            t[p] = act.apply(f, t[p], span[p])
            if p < size: lz[p] = act.compose(f, lz[p])

    fn push(mut self, p: Int):
        unchecked:
            self.put(2 * p, lz[p])
            self.put(2 * p + 1, lz[p])
            lz[p] = act.id

    fn open(mut self, l: Int, h: Int):
        for i in (1..=log).rev:
            if ((l >> i) << i) != l: self.push(l >> i)
            if ((h >> i) << i) != h: self.push((h - 1) >> i)

    /// Assign position `i` to `x`, pushing pending tags on its path. O(log |D|).
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

    /// Read the current value at position `i`, pushing pending tags on its path. O(log |D|).
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
        let (lo, hi) = r.ends
        if lo == hi: return m.id
        var (l, h) = (lo + size, hi + size)
        self.open(l, h)
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

    /// Every position in the range takes the update `f`.
    pub fn apply(mut self, r: Range<D>, f: F):
        let (lo, hi) = r.ends
        if lo == hi: return
        let (l0, h0) = (lo + size, hi + size)
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
pub const AssignSum = Action<Int, ?Int>(
    none,
    (f, g) => f ?? g,
    (f, x, len) => if f == none: x else: f! * len,
    over: +,
)
/// Overwrite every position, fold with `min`.
pub const AssignMin = Action<Int, ?Int>(none, (f, g) => f ?? g, (f, x, len) => f ?? x, over: min)
/// Overwrite every position, fold with `max`.
pub const AssignMax = Action<Int, ?Int>(none, (f, g) => f ?? g, (f, x, len) => f ?? x, over: max)
/// Affine update `(mul, add)` over `+` for `Int`: `x -> mul * x + add`.
pub const AffineSum = Action<Int, (Int, Int)>(
    (1, 0),
    (new_f, old_f) => (new_f.0 * old_f.0, new_f.0 * old_f.1 + new_f.1),
    (f, val, span) => f.0 * val + f.1 * span,
    over: +,
)

/// Generic affine update `(mul, add)` over `+` for any ring `T` (`Int`, `Mod<P>`). O(1).
pub fn affine_sum<T>() -> Action<T, (T, T)>:
    Action<T, (T, T)>(
        (T.one, T.zero),
        (new_f, old_f) => (new_f.0 * old_f.0, new_f.0 * old_f.1 + new_f.1),
        (f, val, span) => f.0 * val + f.1 * T(span),
        over: +,
    )

/// Paired aggregate `(now, flipped)` for range-toggle / complement updates on a `LazySegTree`.
/// Build the toggle action with `flip_action(m)` (tag `Bool`: `true` swaps `now` and `flipped`).
pub struct Flip<T>(now: T, flipped: T):
    /// Leaf value for an element that is currently `on`: `(x, id)` if `on`, else `(id, x)`. O(1).
    pub fn of(on: Bool, x: T, id: T) -> Flip<T>:
        if on: Flip(x, id) else: Flip(id, x)

/// Range-toggle `Action<Flip<T>, Bool>` over monoid `m`: `true` swaps `now` and `flipped`. O(1).
pub fn flip_action<T, M>(m: M) -> Action<Flip<T>, Bool>:
    let k = Monoid<T>.of(m)
    Action<Flip<T>, Bool>(
        id: false,
        compose: (f, g) => f != g,
        apply: (f, x, span) => if f: Flip(x.flipped, x.now) else: x,
        over: Monoid<Flip<T>>(
            Flip(k.id, k.id),
            (x, y) => Flip(k.op(x.now, y.now), k.op(x.flipped, y.flipped)),
        ),
    )

/// Range update, point query: no fold monoid, no pulls, no span array.
/// `compose(new_f, old_f)` applies `old_f` first then `new_f`; `apply(f, x)` updates a single leaf.
/// `apply`, `get`, `set`, and `update` are O(log n); memory is n leaves plus bit_ceil(n) tags.
pub struct DualSegTree<D, T, F>(id: F, compose: (F, F) -> F, apply_fn: (F, T) -> T):
    val: List<T>
    lz: List<F>
    size: Int
    log: Int

    /// A dual tree over `a` or filled with `fill`. O(n).
    pub fn new(
        id: F,
        compose: (F, F) -> F,
        apply: (F, T) -> T,
        a: ?[D]T = none,
        fill: ?T = none,
    ) -> DualSegTree<D, T, F>:
        let size = bit_ceil(D.K)
        var val: List<T> = []
        if let init = a:
            assert fill == none, "pass either `a` or `fill`, not both"
            val = [init[i] for i in D]
        elif let v = fill:
            val = List.filled(D.K, v)
        else:
            assert D.K == 0, "DualSegTree.new needs `a:` or `fill:` for a non-empty domain"
        DualSegTree(id, compose, apply, val, List.filled(size, id), size, ctz(size))

    fn put(mut self, p: Int, f: F):
        unchecked:
            if p < size: lz[p] = compose(f, lz[p])
            else:
                let idx = p - size
                if idx < val.len: val[idx] = apply_fn(f, val[idx])

    fn push(mut self, p: Int):
        unchecked:
            self.put(2 * p, lz[p])
            self.put(2 * p + 1, lz[p])
            lz[p] = id

    fn open(mut self, l: Int, h: Int):
        for i in (1..=log).rev:
            if ((l >> i) << i) != l: self.push(l >> i)
            if ((h >> i) << i) != h: self.push((h - 1) >> i)

    /// Apply tag `f` to every position in `r`. O(log n).
    pub fn apply(mut self, r: Range<D>, f: F):
        let (lo, hi) = r.ends
        if lo == hi: return
        let (l0, h0) = (lo + size, hi + size)
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

    /// Current value at position `i`. O(log n).
    pub fn get(mut self, i: D) -> T:
        let p = i.to_idx + size
        for k in (1..=log).rev: self.push(p >> k)
        val[i.to_idx]

    /// Overwrite position `i` with `x`. O(log n).
    pub fn set(mut self, i: D, x: T):
        let p = i.to_idx + size
        for k in (1..=log).rev: self.push(p >> k)
        val[i.to_idx] = x

    /// Replace position `i` with `f(cur)` in one O(log n) pass.
    pub fn update(mut self, i: D, f: T -> T):
        let p = i.to_idx + size
        for k in (1..=log).rev: self.push(p >> k)
        val[i.to_idx] = f(val[i.to_idx])

fn brute<D>(a: [D]Int, r: Range<D>, pick: Int) -> Int:
    if pick == 0: return sum(a[v] for v in r)
    if pick == 1: return min(a[v] for v in r) ?? Int.MAX
    max(a[v] for v in r) ?? Int.MIN

@test
fn the_stock_actions_match_brute_force():
    domain V = 1..=10
    for (k, act) in [
        (0, AddSum),
        (1, AddMin),
        (2, AddMax),
        (0, AssignSum),
        (1, AssignMin),
        (2, AssignMax),
    ]:
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
    let count_max = Monoid<(Int, Int)>(
        (Int.MIN, 0),
        (a, b) => if a.0 > b.0: a else: if b.0 > a.0: b else: (a.0, a.1 + b.1),
    )
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
    let over = Monoid<(Int, Int, Int)>(
        (Int.MAX, Int.MAX, 0),
        (x, y) => (min(x.0, y.0), min(x.1, y.1), x.2 + y.2),
    )
    let act = Action<(Int, Int, Int), (?Int, ?Int, Int)>(
        (none, none, 0),
        ccc_compose,
        ccc_apply,
        over: over,
    )
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
            let want = (
                min(a[v] for v in l..=r)!,
                min(b[v] for v in l..=r)!,
                sum(a[v] for v in l..=r),
            )
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
        compose: (new_f, old_f) => (
            min(max(old_f.0, new_f.0), new_f.1),
            max(min(old_f.1, new_f.1), new_f.0),
        ),
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

@test
fn fill_and_affine_sum_match_brute():
    domain V = 0..7
    var dp = LazySegTree<V, Int>(AddMax, fill: 0)
    assert dp.all == 0
    dp.apply(V(1)..=V(4), 5)
    assert dp.all == 5 and dp.get(V(0)) == 0 and dp.get(V(3)) == 5

    var a = [V] _ => rand(-5, 5)
    var st = LazySegTree<V, Int>(AffineSum, a)
    for _ in 0..50:
        let l = V.nth(rand(0, 6))
        let r = V.nth(rand(l.to_idx, 6))
        let mul = rand(-2, 3)
        let add = rand(-4, 4)
        for v in l..=r: a[v] = a[v] * mul + add
        st.apply(l..=r, (mul, add))
        assert st.fold(l..=r) == sum(a[v] for v in l..=r)

@test
fn dual_segtree_non_commutative_tags_match_brute():
    // Non-commutative affine tag (mul, add) applied to ranges, point queries and updates.
    domain V = 1..=11
    var a = [V] _ => rand(-5, 5)
    var st = DualSegTree<V, Int, (Int, Int)>(
        id: (1, 0),
        compose: (nf, of) => (nf.0 * of.0, nf.0 * of.1 + nf.1),
        apply: (f, x) => f.0 * x + f.1,
        a: a,
    )
    for _ in 0..100:
        let kind = rand(0, 3)
        if kind <= 1:
            let l = V.nth(rand(0, 10))
            let r = V.nth(rand(l.to_idx, 10))
            let (m, b) = (rand(-2, 2), rand(-6, 6))
            for v in l..=r: a[v] = m * a[v] + b
            st.apply(l..=r, (m, b))
        elif kind == 2:
            let i = V.nth(rand(0, 10))
            let x = rand(-10, 10)
            a[i] = x
            st.set(i, x)
        else:
            let i = V.nth(rand(0, 10))
            a[i] = a[i] * 3 - 1
            st.update(i, x => x * 3 - 1)
        let q = V.nth(rand(0, 10))
        assert st.get(q) == a[q]
    for v in V: assert st.get(v) == a[v]

struct TreapNode<T, F>(l: ?Int, r: ?Int, pri: U64, sz: Int, val: T, tag: ?F)

/// Arena-backed randomized treap with subtree-size-aware lazy tag propagation.
/// Supports value-ordered insertion (`insert`, using `before`) for slope-trick / priority
/// structures with lazy updates, and 0-based implicit sequence operations (`push`, `insert_at`,
/// `apply_range`, `get`).
/// `apply(v, left_sz, f)` updates a node's value given its left subtree size;
/// `split(f, left_sz)` splits a tag `f` for the left and right subtrees (default `(f, f)`);
/// `compose(new_f, old_f)` applies `old_f` first then `new_f`.
/// `insert`, `insert_at`, `push`, `apply_range`, `get` are O(log n) expected; `apply_all` is O(1);
/// `take(k)` is O(k + log n); `to_list` is O(n). Memory is O(n) in a contiguous node arena.
pub struct LazyTreap<T, F>(
    apply_fn: (T, Int, F) -> T,
    split_fn: (F, Int) -> (F, F),
    compose_fn: (F, F) -> F,
    before_fn: (T, T) -> Bool,
):
    nodes: List<TreapNode<T, F>>
    root: ?Int
    rng: Rng

    /// Create an empty `LazyTreap` with node updater `apply`, tag composition `compose`,
    /// optional subtree tag splitter `split`, and optional order predicate `before(x, y)`
    /// (true when `x` belongs before `y`). O(1).
    pub fn new(
        apply: (T, Int, F) -> T,
        compose: (F, F) -> F,
        split: (F, Int) -> (F, F) = (f, _) => (f, f),
        before: (T, T) -> Bool = (_, _) => false,
    ) -> LazyTreap<T, F>:
        LazyTreap(apply, split, compose, before, [], none, Rng(48))

    fn sz_of(self, u: ?Int) -> Int:
        if let i = u:
            unchecked: return nodes[i].sz
        0

    /// Number of elements currently stored. O(1).
    @get
    pub fn len(self) -> Int = self.sz_of(root)

    /// Whether the treap is empty. O(1).
    @get
    pub fn is_empty(self) -> Bool = root == none

    fn put(mut self, u: ?Int, f: F):
        if let i = u:
            unchecked:
                nodes[i].val = apply_fn(nodes[i].val, self.sz_of(nodes[i].l), f)
                if let old = nodes[i].tag:
                    nodes[i].tag = compose_fn(f, old)
                else:
                    nodes[i].tag = f

    fn push_tag(mut self, i: Int, f: F):
        unchecked:
            let (fl, fr) = split_fn(f, self.sz_of(nodes[i].l))
            self.put(nodes[i].l, fl)
            self.put(nodes[i].r, fr)
            nodes[i].tag = none

    fn down(mut self, i: Int):
        unchecked:
            if let f = nodes[i].tag: self.push_tag(i, f)

    fn pull(mut self, i: Int):
        unchecked: nodes[i].sz = 1 + self.sz_of(nodes[i].l) + self.sz_of(nodes[i].r)

    fn split_val(mut self, u: ?Int, x: T) -> (?Int, ?Int):
        let i = u ?? return (none, none)
        unchecked:
            if let f = nodes[i].tag: self.push_tag(i, f)
            if before_fn(x, nodes[i].val):
                let (a, b) = self.split_val(nodes[i].l, x)
                nodes[i].l = b
                self.pull(i)
                return (a, i)
            else:
                let (a, b) = self.split_val(nodes[i].r, x)
                nodes[i].r = a
                self.pull(i)
                return (i, b)

    fn ins(mut self, u: ?Int, idx: Int, x: T, p: U64) -> Int:
        let i = u ?? return idx
        unchecked:
            if p < nodes[i].pri:
                let (a, b) = self.split_val(i, x)
                nodes[idx].l = a
                nodes[idx].r = b
                self.pull(idx)
                return idx
            if let f = nodes[i].tag: self.push_tag(i, f)
            nodes[i].sz += 1
            if before_fn(x, nodes[i].val):
                nodes[i].l = self.ins(nodes[i].l, idx, x, p)
            else:
                nodes[i].r = self.ins(nodes[i].r, idx, x, p)
            return i

    fn split_pos(mut self, u: ?Int, k: Int) -> (?Int, ?Int):
        let i = u ?? return (none, none)
        self.down(i)
        unchecked:
            let ls = self.sz_of(nodes[i].l)
            if k <= ls:
                let (a, b) = self.split_pos(nodes[i].l, k)
                nodes[i].l = b
                self.pull(i)
                return (a, i)
            else:
                let (a, b) = self.split_pos(nodes[i].r, k - ls - 1)
                nodes[i].r = a
                self.pull(i)
                return (i, b)

    fn merge(mut self, a: ?Int, b: ?Int) -> ?Int:
        let ia = a ?? return b
        let ib = b ?? return a
        unchecked:
            if nodes[ia].pri < nodes[ib].pri:
                self.down(ia)
                nodes[ia].r = self.merge(nodes[ia].r, ib)
                self.pull(ia)
                return ia
            else:
                self.down(ib)
                nodes[ib].l = self.merge(ia, nodes[ib].l)
                self.pull(ib)
                return ib

    /// Insert `x` at the first position where `before(x, elem)` holds. O(log n) expected.
    pub fn insert(mut self, x: T):
        let idx = nodes.len
        let p = rng.next()
        nodes.push(TreapNode(none, none, p, 1, x, none))
        root = self.ins(root, idx, x, p)

    /// Append `x` at the end of the sequence. O(log n) expected.
    pub fn push(mut self, x: T):
        let idx = nodes.len
        nodes.push(TreapNode(none, none, rng.next(), 1, x, none))
        root = self.merge(root, idx)

    /// Insert `x` at 0-based position `pos` (`0 <= pos <= len`). O(log n) expected.
    pub fn insert_at(mut self, pos: Int, x: T):
        assert 0 <= pos and pos <= self.len
        let idx = nodes.len
        nodes.push(TreapNode(none, none, rng.next(), 1, x, none))
        let (a, b) = self.split_pos(root, pos)
        root = self.merge(self.merge(a, idx), b)

    /// Apply lazy tag `f` to every element in the treap. O(1).
    pub fn apply_all(mut self, f: F):
        self.put(root, f)

    /// Apply lazy tag `f` to elements at 0-based positions `lo..hi`. O(log n) expected.
    pub fn apply_range(mut self, lo: Int, hi: Int, f: F):
        assert 0 <= lo and lo <= hi and hi <= self.len
        if lo == hi: return
        let (a, bc) = self.split_pos(root, lo)
        let (b, c) = self.split_pos(bc, hi - lo)
        self.put(b, f)
        root = self.merge(self.merge(a, b), c)

    /// Read the element at 0-based position `pos`. O(log n) expected.
    pub fn get(mut self, pos: Int) -> T:
        assert 0 <= pos and pos < self.len
        var cur = root
        var rem = pos
        unchecked:
            while let i = cur:
                self.down(i)
                let ls = self.sz_of(nodes[i].l)
                if rem < ls:
                    cur = nodes[i].l
                elif rem == ls:
                    return nodes[i].val
                else:
                    rem -= ls + 1
                    cur = nodes[i].r
            assert false
            return nodes[0].val

    fn walk(mut self, u: ?Int, lim: Int, mut out: List<T>):
        if out.len >= lim: return
        if let i = u:
            self.down(i)
            unchecked:
                self.walk(nodes[i].l, lim, mut out)
                if out.len < lim:
                    out.push(nodes[i].val)
                    self.walk(nodes[i].r, lim, mut out)

    /// Return the first `min(k, len)` elements in order after pushing lazy tags. O(k + log n).
    pub fn take(mut self, k: Int) -> List<T>:
        var out: List<T> = []
        if k > 0: self.walk(root, k, mut out)
        out

    /// Return all elements in order after pushing lazy tags. O(n).
    pub fn to_list(mut self) -> List<T> = self.take(self.len)

@test
fn lazy_treap_slope_trick_and_implicit_ops_match_brute():
    // 1. Descending ordered treap with position-dependent affine tag (slope trick).
    var tr = LazyTreap<Int, (Int, Int)>(
        apply: (v, ls, f) => v + f.0 + ls * f.1,
        compose: (nf, of) => (nf.0 + of.0, nf.1 + of.1),
        split: (f, ls) => (f, (f.0 + (ls + 1) * f.1, f.1)),
        before: (a, b) => a >= b,
    )
    var ref: List<Int> = []
    for _ in 0..40:
        let base = rand(-5, 10)
        let step = rand(-4, 0)
        tr.apply_all((base, step))
        for j in 0..ref.len: ref[j] += base + j * step
        let x = rand(-10, 10)
        tr.insert(x)
        var pos = ref.len
        for j in 0..ref.len:
            if x >= ref[j]:
                pos = j
                break
        ref.insert(pos, x)
        assert tr.len == ref.len
        assert tr.take(5) == ref[0..min(5, ref.len)]
    assert tr.to_list() == ref

    // 2. Implicit sequence treap with position-independent range add + insert_at + get.
    var seq = LazyTreap<Int, Int>(apply: (v, _, f) => v + f, compose: (nf, of) => nf + of)
    var arr: List<Int> = []
    for _ in 0..40:
        let pos = rand(0, arr.len)
        let x = rand(-10, 10)
        seq.insert_at(pos, x)
        arr.insert(pos, x)
        let lo = rand(0, arr.len - 1)
        let hi = rand(lo + 1, arr.len)
        let d = rand(-5, 5)
        seq.apply_range(lo, hi, d)
        for j in lo..hi: arr[j] += d
        let q = rand(0, arr.len - 1)
        assert seq.get(q) == arr[q]
    assert seq.to_list() == arr

@test
fn flip_action_toggles_ranges():
    domain I = 0..6
    let init = [I] i => Flip<Int>.of(i.ord % 2 == 0, 1, 0)
    var st = LazySegTree<I, Flip<Int>>(flip_action<Int>(+), init)
    assert st.all.now == 3
    assert st.all.flipped == 3
    st.apply(I(1)..I(5), true)
    assert st.all.now == 3
    assert st.fold(I(1)..I(5)).now == 2
    st.apply(I(0)..I(3), true)
    assert st.all.now == 2
