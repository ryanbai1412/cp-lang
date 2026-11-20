//! Segment tree with range updates.
use std.algebra.{Monoid, Action, Sum, Max}

/// Range update, range fold. `apply`, `set` and `fold` are O(log n);
/// memory is 2 * bit_ceil(n) values plus bit_ceil(n) pending updates.
/// `act.apply(f, x)` must distribute over `m.op`; for a sum that needs the
/// length, keep the length in the value: `(sum, len)`.
pub struct LazySegTree<D, T, F>(m: Monoid<T>, act: Action<T, F>):
    t: List<T>
    lz: List<F>
    size: Int
    log: Int

    pub fn new(m: Monoid<T>, act: Action<T, F>) -> LazySegTree<D, T, F>:
        let size = bit_ceil(D.K)
        LazySegTree(m, act, List.filled(2 * size, m.id), List.filled(size, act.id), size, ctz(size))

    /// Build from an array in O(n).
    pub fn from(m: Monoid<T>, act: Action<T, F>, a: [D]T) -> LazySegTree<D, T, F>:
        var st = LazySegTree<D, T, F>.new(m, act)
        for i in D: st.t[st.size + i.to_idx] = a[i]
        for p in (1..st.size).rev: st.pull(p)
        st

    fn pull(mut self, p: Int):
        t[p] = m.op(t[2 * p], t[2 * p + 1])

    fn put(mut self, p: Int, f: F):
        t[p] = act.apply(f, t[p])
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

    pub fn get(mut self, i: D) -> T:
        let p = i.to_idx + size
        for k in (1..=log).rev: self.push(p >> k)
        t[p]

    /// The fold of `a[l..=r]`, left to right.
    pub fn fold(mut self, r: Range<D>) -> T:
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

    /// `a[i] = apply(f, a[i])` for every `i` in the range.
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

/// Maximum over `?Int`, where `none` is "no value": the identity, and the
/// fold of an empty range. `?Int` is 8 bytes, the same as `Int`.
pub const MaxOpt = Monoid<?Int>(id: none, op: (a, b) => if a == none: b else: if b == none: a else: max(a!, b!))

/// Range add, range max, over `MaxOpt`. A `none` cell stays `none` under
/// every add, so it works as "minus infinity" or "switched off". The empty
/// value is outside `Int` on purpose: with `Int.MIN` as the marker, a sum
/// that lands exactly on it would be taken for empty and the next add
/// dropped, which breaks `composes` at `Int.MIN + 1`.
pub const AddMax = Action<?Int, Int>(id: 0, compose: (f, g) => f + g, apply: (f, x) => if x == none: none else: x! + f)

/// Range add over `(sum, len)` pairs, for range add with range sum.
pub const AddSum = Action<(Int, Int), Int>(id: 0, compose: (f, g) => f + g, apply: (f, x) => (x.0 + f * x.1, x.1))

/// Range assign over `(sum, len)` pairs, for range assign with range sum. `none` leaves a range alone.
pub const AssignSum = Action<(Int, Int), ?Int>(id: none, compose: (f, g) => if f == none: g else: f, apply: (f, x) => if f == none: x else: (f! * x.1, x.1))

/// The monoid of `(sum, len)` pairs.
pub const SumLen = Monoid<(Int, Int)>(id: (0, 0), op: (a, b) => (a.0 + b.0, a.1 + b.1))

@test
fn range_add_range_max_matches_brute_force():
    domain V = 1..=10
    var a = [V] v => rand(-9, 9)
    var st = LazySegTree<V, ?Int, Int>.from(MaxOpt, AddMax, [V] v => a[v])
    for _ in 0..80:
        let l = V.nth(rand(0, 9))
        let r = V.nth(rand(l.to_idx, 9))
        if rand(0, 1) == 0:
            let x = rand(-5, 5)
            for v in l..=r: a[v] += x
            st.apply(l..=r, x)
        else:
            assert st.fold(l..=r) == max(a[v] for v in l..=r)
    for v in V: assert st.get(v) == a[v]

@test
fn range_add_range_sum_carries_the_length():
    domain V = 0..6
    var st = LazySegTree<V, (Int, Int), Int>.from(SumLen, AddSum, [V] v => (0, 1))
    st.apply(V(1)..=V(4), 3)
    st.apply(V(3)..=V(5), 10)
    assert st.fold(V(0)..=V(5)).0 == 4 * 3 + 3 * 10
    assert st.fold(V(4)..=V(4)) == (13, 1)
    st.set(V(4), (100, 1))
    assert st.fold(V(3)..=V(5)).0 == 13 + 100 + 10
    assert st.fold(V(2)..V(2)) == (0, 0)

@test
fn range_assign_range_sum_matches_brute_force():
    domain V = 0..9
    var a = [V] v => 0
    var st = LazySegTree<V, (Int, Int), ?Int>.from(SumLen, AssignSum, [V] v => (0, 1))
    for _ in 0..80:
        let l = V.nth(rand(0, 8))
        let r = V.nth(rand(l.to_idx, 8))
        if rand(0, 1) == 0:
            let x = rand(-5, 5)
            st.apply(l..=r, x)
            for v in l..=r: a[v] = x
        else:
            assert st.fold(l..=r).0 == sum(a[v] for v in l..=r)

@test
fn stock_actions_obey_their_laws():
    assert laws_hold(AddMax, T: [none, -3, 0, 8], F: [-4, 0, 6])
    assert laws_hold(MaxOpt, [none, Int.MIN, -3, 0, 8])
    assert laws_hold(AddSum, T: [(0, 0), (5, 2), (-1, 3)], F: [-4, 0, 6])
    assert laws_hold(SumLen, [(0, 0), (5, 2), (-1, 3)])

@test
fn add_on_max_composes_at_the_bottom_of_int():
    // A sum may land on Int.MIN: it is a value like any other, not "empty".
    let low: ?Int = Int.MIN + 1
    assert AddMax.apply(-1, low) == Int.MIN
    assert AddMax.apply(1, AddMax.apply(-1, low)) == AddMax.apply(AddMax.compose(1, -1), low)
    assert AddMax.apply(5, none) == none
    domain V = 0..3
    var st = LazySegTree<V, ?Int, Int>.from(MaxOpt, AddMax, [V] v => Int.MIN + 1)
    st.apply(V(0)..=V(2), -1)
    st.apply(V(0)..=V(1), 1)
    assert st.fold(V(0)..=V(2)) == Int.MIN + 1 and st.get(V(2)) == Int.MIN
    assert st.fold(V(1)..V(1)) == none
