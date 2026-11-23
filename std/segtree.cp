//! Segment tree over a domain, for any associative operation: point or range update, range query.
use std.algebra.{Action, Monoid}
use std.bag.{SortedBag}
use std.math.{gcd, bit}
use std.seq.{ends, first_true}

struct NoAction
struct NoOp
fn same<A, B>(a: A, b: B) -> Bool = a == b

/// Point update and range fold. The operation is a bare operator (`+`, `*`, `^`, `&`, `|`,
/// `min`, `max`, `gcd`) or your own `Monoid(id, op)`: `SegTree<V, Int>(max, a)`,
/// `SegTree<V, Int>(+)`, `SegTree<V, Run>(Monoid(Run.empty, Run.join), a)`.
/// `set` and `fold` are \(O(\log n)\), and so are `right_while` and `left_while`; memory is
/// 2 * bit_ceil(n) values. The fold keeps the order of the positions, so
/// non-commutative operations work.
pub struct SegTree<D, T, F = Int>(m: Monoid<T>):
    @cache t: List<T>
    @cache lz: List<F>
    @cache has_lz: List<Bool>
    span: List<Int>
    size: Int
    log: Int
    act: ?Action<T, F>
    /// What each position holds, for a tree built with `of`.
    pub var source: ?(D -> T)
    var dual: Bool = false

    /// A tree over `D` folding with `op`: built from `a`, filled with `fill`, or holding the identity
    /// everywhere when neither is given. `op` is a bare operator (`+`, `*`, `^`, `&`, `|`, `min`,
    /// `max`, `gcd`) or a `Monoid(id, op)`.
    ///
    /// `act:` adds range tags (`apply`, `apply_all`); with `act:` and no `op` the tree applies tags
    /// and reads points only. When the action commutes (the stock `add`, or `commutes: true`) `apply`
    /// skips push-down, which changes speed, never answers. `over: lo..hi` covers a huge `Int` range instead, making nodes
    /// on first touch; positions outside the touched ones hold `fill` or the identity.
    ///
    /// cost: \(O(\lvert D \rvert)\); \(O(1)\) with `over:`
    /// empty: an empty `D` gives a tree whose every fold is the identity
    /// order: positions keep the domain `D` of `a`
    /// traps: when both `a` and `fill` are given, or `over:` is given together with `a`
    /// example: `SegTree(+, [1, 2, 3]).all == 6`
    /// example: `SegTree<0..4, Int>(max, fill: 5).fold(1..3) == 5`
    /// example: `SegTree<0..4, Int>(+).all == 0`
    /// example: `var s = SegTree<Int, Int>(+, over: 0..1_000_000_000); s.set(999_999_999, 4); s.all == 4`
    pub fn new<M, A>(
        op: M = NoOp(),
        a: ?[D]T = None,
        fill: ?T = None,
        act: A = NoAction(),
        over: ?Range<Int> = None,
    ) -> SegTree<D, T, F>:
        if over != None: todo
        if same(op, NoOp()):
            // Dual tree: tags only, read one point at a time. Internal nodes hold tags and never
            // values, so `apply` runs on leaves only.
            assert not same(act, NoAction()), "a tree with no fold op needs `act:`"
            let seed = if let init = a: init[D.first!] else: fill!
            let left = Monoid<T>(seed, (x, _) => x)
            var st = SegTree<D, T, F>.build_lazy(left, a, fill, Action<T, F>.of(act, left))
            st.dual = true
            return st
        let m = Monoid<T>.of(op)
        if same(act, NoAction()):
            return SegTree<D, T, F>.build(m, a, fill)
        else:
            let resolved = Action<T, F>.of(act, op)
            return SegTree<D, T, F>.build_lazy(m, a, fill, resolved)

    fn build(m: Monoid<T>, a: ?[D]T, fill: ?T) -> SegTree<D, T, F>:
        let size = bit_ceil(D.count)
        var t = List.filled(2 * size, m.id)
        if let init = a:
            assert fill == None, "pass either `a` or `fill`, not both"
            for i in D: t[size + i.ord] = init[i]
        elif let v = fill:
            for i in 0..D.count: t[size + i] = v
        for p in (1..size).rev: t[p] = m.op(t[2 * p], t[2 * p + 1])
        SegTree(m, t, [], [], [], size, 0, None, None)

    fn build_lazy(m: Monoid<T>, a: ?[D]T, fill: ?T, resolved: Action<T, F>) -> SegTree<D, T, F>:
        let size = bit_ceil(D.count)
        var t = List.filled(2 * size, m.id)
        var span = List.filled(2 * size, 0)
        for i in 0..D.count: span[size + i] = 1
        if let init = a:
            assert fill == None, "pass either `a` or `fill`, not both"
            for i in D: t[size + i.ord] = init[i]
        elif let v = fill:
            for i in 0..D.count: t[size + i] = v
        for p in (1..size).rev:
            t[p] = m.op(t[2 * p], t[2 * p + 1])
            span[p] = span[2 * p] + span[2 * p + 1]
        SegTree(
            m,
            t,
            List.filled(size, resolved.id),
            List.filled(size, false),
            span,
            size,
            ctz(size),
            Some(resolved),
            None,
        )

    fn put(self, p: Int, f: F):
        let a = act!
        unchecked:
            if span[p] == 0: return
            if p >= size or not dual: t[p] = a.apply(f, t[p], span[p])
            if p < size:
                lz[p] = a.compose(f, lz[p])
                has_lz[p] = true

    fn push(self, p: Int):
        let a = act!
        unchecked:
            if not has_lz[p]: return
            self.put(2 * p, lz[p])
            self.put(2 * p + 1, lz[p])
            lz[p] = a.id
            has_lz[p] = false

    fn open(self, l: Int, h: Int):
        for i in (1..=log).rev:
            if ((l >> i) << i) != l: self.push(l >> i)
            if ((h >> i) << i) != h: self.push((h - 1) >> i)

    fn pull_node(self, p: Int):
        unchecked:
            t[p] = m.op(t[2 * p], t[2 * p + 1])
            if has_lz[p]: t[p] = act!.apply(lz[p], t[p], span[p])

    /// Apply tag `f` to every position of `r`: each fold over `r` then sees the values with `f`
    /// applied, through the action given to `act:`.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var s = SegTree(+, [1, 2, 3], act: add); s.apply(0..2, 10); s.all == 26`
    /// example: `var s = SegTree(max, [1, 2, 3], act: add); s.apply(0..1, 5); s.all == 6`
    /// example: `var s = SegTree(+, [1, 2, 3], act: affine); s.apply(0..2, (2, 1)); s.all == 11`
    pub fn apply(mut self, r: Range<D>, f: F):
        assert act != None, "apply requires `act:` on SegTree"
        let (lo, hi) = r.ends
        if lo >= hi: return
        let (l0, h0) = (lo + size, hi + size)
        if dual or not act!.commutes: self.open(l0, h0)
        var (l, h) = (l0, h0)
        unchecked:
            while l < h:
                if l % 2 == 1:
                    self.put(l, f)
                    l += 1
                if h % 2 == 1:
                    h -= 1
                    self.put(h, f)
                l /= 2
                h /= 2
        if dual: return
        for i in 1..=log:
            if ((l0 >> i) << i) != l0: self.pull_node(l0 >> i)
            if ((h0 >> i) << i) != h0: self.pull_node((h0 - 1) >> i)

    /// Apply tag `f` to every position; `apply(D.all, f)`.
    ///
    /// cost: \(O(1)\) amortized when the tags commute, else \(O(\lvert D \rvert)\)
    /// empty: changes nothing for an empty `D`
    /// traps: never
    /// example: `var s = SegTree(+, [1, 2, 3], act: add); s.apply_all(1); s.all == 9`
    pub fn apply_all(mut self, f: F):
        assert act != None, "apply_all requires `act:` on SegTree"
        if D.count > 0: self.put(1, f)

    /// The fold of the values in `r`, left to right: `op(op(a[l], a[l + 1]), ...)`.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// empty: the identity
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var s = SegTree(+, [1, 2, 3], act: add); s.apply(1..3, 1); s.fold(1..3) == 7`
    pub fn fold(self, r: Range<D>) -> T:
        let (lo, hi) = r.ends
        if lo >= hi: return m.id
        var (l, h) = (lo + size, hi + size)
        if act != None: self.open(l, h)
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

    /// The last cut `d` at or right of `from` such that `ok(fold(from..d))` holds. `ok` must be
    /// monotone (once false it stays false as `d` moves right) and true on the identity, so `from`
    /// itself always qualifies. Replaces `max_right`.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// ties: the rightmost qualifying cut
    /// traps: when `ok(identity)` is false
    /// example: `SegTree(+, [1, 2, 3, 4]).right_while((0..0).r, s => s <= 3) == (0..2).r`
    /// example: `SegTree(+, [1, 2, 3, 4]).right_while((0..1).r, s => s <= 5) == (0..3).r`
    /// example: `SegTree(+, [1, 2, 3, 4]).right_while((0..0).r, s => s <= 100) == (0..4).r`
    pub fn right_while(self, from: Cut<D>, ok: T -> Bool) -> Cut<D>:
        assert ok(m.id), "`ok` must hold on the identity"
        let n = D.count
        if from.ord == n: return from
        var l = from.ord + size
        if act != None: self.open(l, l + 1)
        var acc = m.id
        while true:
            while l % 2 == 0: l /= 2
            if not ok(m.op(acc, t[l])):
                while l < size:
                    if act != None: self.push(l)
                    l *= 2
                    if ok(m.op(acc, t[l])):
                        acc = m.op(acc, t[l])
                        l += 1
                return D.cuts.nth(l - size)
            acc = m.op(acc, t[l])
            l += 1
            if l & -l == l: break
        D.cuts.nth(n)

    /// The first cut `d` at or left of `to` such that `ok(fold(d..to))` holds. `ok` must be
    /// monotone (once false it stays false as `d` moves left) and true on the identity, so `to`
    /// itself always qualifies. Replaces `min_left`.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// ties: the leftmost qualifying cut
    /// traps: when `ok(identity)` is false
    /// example: `SegTree(+, [1, 2, 3, 4]).left_while((0..4).r, s => s <= 7) == (0..2).r`
    /// example: `SegTree(+, [1, 2, 3, 4]).left_while((0..4).r, s => s <= 3) == (0..4).r`
    pub fn left_while(self, to: Cut<D>, ok: T -> Bool) -> Cut<D>:
        assert ok(m.id), "`ok` must hold on the identity"
        if to.ord == 0: return to
        var r = to.ord + size
        if act != None: self.open(r - 1, r)
        var acc = m.id
        while true:
            r -= 1
            while r > 1 and r % 2 == 1: r /= 2
            if not ok(m.op(t[r], acc)):
                while r < size:
                    if act != None: self.push(r)
                    r = 2 * r + 1
                    if ok(m.op(t[r], acc)):
                        acc = m.op(t[r], acc)
                        r -= 1
                return D.cuts.nth(r + 1 - size)
            acc = m.op(t[r], acc)
            if r & -r == r: break
        D.cuts.nth(0)

    /// A tree whose position `i` holds `f(i)`. `f` is kept: when something it reads changes,
    /// `refresh(i)` recomputes position `i`, so the formula is written once instead of at every
    /// update site.
    ///
    /// cost: \(O(\lvert D \rvert)\), calling `f` once per position in key order
    /// empty: an empty `D` gives a tree whose every fold is the identity
    /// order: position `i` holds `f(i)`
    /// traps: never
    ///
    /// ```cp
    /// domain D = 0..3
    /// let w: [D]Int = [1, 2, 3]
    /// SegTree<D, Int>.of(+, i => w[i] * 2).all == 12
    /// ```
    pub fn of<M>(op: M, f: D -> T) -> SegTree<D, T, F>:
        var s = SegTree<D, T, F>.build(Monoid<T>.of(op), [D] i => f(i), None)
        s.source = f
        s

    /// Recompute position `i` as `f(i)` with the formula given to `of`, and update its ancestors.
    ///
    /// cost: \(O(\log \lvert D \rvert)\) plus one call of `f`
    /// traps: when `i` is outside `D`, or the tree was not built with `of`
    ///
    /// ```cp
    /// domain D = 0..3
    /// var w: [D]Int = [1, 2, 3]
    /// var s = SegTree<D, Int>.of(+, i => w[i])
    /// w[D(1)] = 10
    /// s.refresh(D(1))
    /// s.all == 14
    /// ```
    pub fn refresh(mut self, i: D):
        assert source != None  // only a tree built with `of` has a formula
        let f = source!
        self.set(i, f(i))

    /// The value at position `i`, with every tag applied.
    ///
    /// cost: \(O(1)\); \(O(\log \lvert D \rvert)\) on a tree with `act:`
    /// traps: when `i` is outside `D`
    /// example: `SegTree(+, [4, 5, 6]).get(1) == 5`
    /// example: `var s = SegTree(act: assign, a: [1, 2, 3]); s.apply(0..2, Some(9)); s.get(1) == 9`
    pub fn get(self, i: D) -> T:
        let p = size + i.ord
        if act != None:
            for k in (1..=log).rev: self.push(p >> k)
        t[p]

    /// Make position `i` hold `x`, replacing any tags applied to it, and update its ancestors.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// traps: when `i` is outside `D`
    /// example: `var s = SegTree(+, [1, 2, 3]); s.set(0, 10); s.all == 15`
    pub fn set(mut self, i: D, x: T):
        let p = size + i.ord
        if act != None:
            for k in (1..=log).rev: self.push(p >> k)
        t[p] = x
        self.pull(i)

    /// Position `i` as a writable place: `st.at(i) += x`, `st.at(i) max= x`. The ancestors are
    /// updated when the write ends, so each use is one `set`; read with `get` instead.
    ///
    /// cost: \(O(\log \lvert D \rvert)\) per write
    /// traps: when `i` is outside `D`
    /// example: `var s = SegTree(+, [1, 2, 3]); s.at(0) += 5; s.all == 11`
    pub fn at(mut self, i: D) -> lends mut T:
        let p = size + i.ord
        if act != None:
            for k in (1..=log).rev: self.push(p >> k)
        lend mut t[p]
        self.pull(i)

    fn pull(mut self, i: D):
        var p = i.ord + size
        unchecked:
            while p > 1:
                p /= 2
                t[p] = m.op(t[2 * p], t[2 * p + 1])

    /// Replace the value `v` at `i` with `f(v)`; `set(i, f(get(i)))` in one walk.
    ///
    /// cost: \(O(\log \lvert D \rvert)\) plus one call of `f`
    /// traps: when `i` is outside `D`
    /// example: `var s = SegTree(+, [1, 2, 3]); s.update(2, x => x * 10); s.all == 33`
    pub fn update(mut self, i: D, f: T -> T):
        self.set(i, f(self.get(i)))

    /// The fold of every position, left to right.
    ///
    /// cost: \(O(1)\)
    /// empty: the identity for an empty `D`
    /// traps: never
    /// example: `SegTree(max, [3, 9, 4]).all == 9`
    @get pub fn all(self) -> T = t[1]

    /// The operation the tree folds with, as its `(id, op)` pair.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `SegTree(+, [1]).monoid.id == 0`
    @get pub fn monoid(self) -> Monoid<T> = m

    /// The fold of `r`, or `None` when `r` is empty; for operations whose identity is only a
    /// sentinel.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// empty: `None`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `SegTree(+, [1, 2]).try_fold(0..2) == Some(3)`
    /// example: `SegTree(+, [1, 2]).try_fold(1..1) == None`
    pub fn try_fold(self, r: Range<D>) -> ?T:
        let (lo, hi) = r.ends
        if lo >= hi: return None
        Some(self.fold(r))

    /// Make every position hold `fill`, or the identity when `fill` is `None`, dropping all pending
    /// tags; the storage is reused. A tree built with `of` keeps its formula.
    ///
    /// cost: \(O(\lvert D \rvert)\)
    /// empty: nothing to reset for an empty `D`
    /// traps: never
    /// example: `var s = SegTree(+, [1, 2, 3]); s.clear(); s.all == 0`
    /// example: `var s = SegTree(+, [1, 2, 3]); s.clear(fill: 2); s.all == 6`
    pub fn clear(mut self, fill: ?T = None):
        let v = fill ?? m.id
        for i in 0..size: t[size + i] = if i < D.count: v else: m.id
        for p in (1..size).rev: t[p] = m.op(t[2 * p], t[2 * p + 1])

    /// The values of the canonical nodes that exactly cover `r`: the maximal aligned blocks of the
    /// tree's `bit_ceil(|D|)` leaves inside `r`. Their fold, left to right, is `fold(r)`.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// empty: yields nothing
    /// order: left to right; at most `2 * log2(bit_ceil(|D|))` values
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `SegTree(+, [1, 2, 3, 4]).walk(0..3).collect() == [3, 3]`
    /// example: `SegTree(+, [1, 2, 3, 4]).walk(1..4).collect() == [2, 7]`
    pub fn walk(self, r: Range<D>) -> yields T:
        let (lo, hi) = r.ends
        var (l, h) = (lo + size, hi + size)
        var right: List<T> = []
        while l < h:
            if l % 2 == 1:
                yield t[l]
                l += 1
            if h % 2 == 1:
                h -= 1
                right.push(t[h])
            l /= 2
            h /= 2
        var k = right.len
        while k > 0:
            k -= 1
            yield right[k]

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
        let r = V.nth(rand(l.ord, 10))
        assert st.fold(l..=r) == [a[v] for v in l..=r].max()!
    assert st.all == [a[v] for v in V].max()!
    assert st.fold(V(3)..V(3)) == Int.MIN
    assert st.fold(..V(4)) == [a[v] for v in V if v < V(4)].max()!
    assert st.fold(V(4)..) == [a[v] for v in V if v >= V(4)].max()!

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
        let r = V.nth(rand(l.ord, 8))
        assert st.fold(l..=r) == sum(a[v] for v in l..=r)
    assert st.all == a.sum()

@test
fn every_bare_operator_folds():
    domain V = 0..6
    let a = [V] v => [12, 18, 6, 30, 7, 9][v.ord]
    assert SegTree<V, Int>(+, a).all == 82
    assert SegTree<V, Int>(min, a).all == 6
    assert SegTree<V, Int>(gcd, a).fold(V(0)..=V(3)) == 6
    assert SegTree<V, Int>(^, a).fold(V(0)..=V(1)) == 12 ^ 18
    assert SegTree<V, Int>(&, a).fold(V(0)..=V(1)) == 12 & 18
    assert SegTree<V, Int>(|, a).fold(V(0)..=V(1)) == 12 | 18
    assert SegTree<V, Int>(*, a).fold(V(4)..=V(5)) == 63
    // over ?Int, `None` is "no value": the identity of max, and what an empty range folds to
    var best = SegTree<V, ?Int>(max)
    assert best.all == None
    best.set(V(2), 5)
    best.set(V(4), -1)
    assert best.all == 5 and best.fold(V(3)..) == -1 and best.fold(..V(2)) == None

@test
fn of_and_refresh_follow_the_formula():
    domain V = 1..=9
    var a = [V] v => rand(-20, 20)
    var st = SegTree<V, Int>.of(min, v => a[v] + v.ord)
    for _ in 0..60:
        let i = V.nth(rand(0, 8))
        a[i] = rand(-20, 20)
        st.refresh(i)
        assert st.all == [a[v] + v.ord for v in V].min()!
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
fn right_while_and_left_while_binary_search():
    for n in [1, 2, 3, 5, 8, 13]:
        domain P = 0..n
        for _ in 0..40:
            var a: [P]Int = 0
            for i in P: a[i] = rand(0, 5)
            let s = SegTree<P, Int>.of(+, i => a[i])
            let lim = rand(0, 12)
            for f in 0..=n:
                var d = f
                var sum = 0
                while d < n and sum + a[P.nth(d)] <= lim:
                    sum += a[P.nth(d)]
                    d += 1
                assert s.right_while(P.cuts.nth(f), x => x <= lim).ord == d
                var e = f
                var sm = 0
                while e > 0 and sm + a[P.nth(e - 1)] <= lim:
                    sm += a[P.nth(e - 1)]
                    e -= 1
                assert s.left_while(P.cuts.nth(f), x => x <= lim).ord == e

@test
fn update_changes_one_value_in_place():
    domain V = 0..6
    var st = SegTree<V, Int>(+, [V] v => v.ord)
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
/// Build in \(O(n \log n)\) time and memory; answer value-range counts over any `Range<D>` in \(O(\log^2 n)\).
pub struct MergeSortTree<D, T>:
    data: List<T>
    beg: List<0..=2_000_000_000>
    fin: List<0..=2_000_000_000>
    size: Int

    /// A static tree over `a` whose nodes hold their values sorted, for counting values in a range.
    ///
    /// cost: \(O(\lvert D \rvert \log \lvert D \rvert)\) time and memory
    /// empty: an empty `a` gives a tree where every count is 0
    /// order: positions keep the domain `D` of `a`
    /// traps: never
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).count_lt(0..5, 3) == 2`
    pub fn new(a: [D]T) -> MergeSortTree<D, T>:
        let size = bit_ceil(D.count)
        var data: List<T> = []
        var beg: List<0..=2_000_000_000> = List.filled(2 * size, 0)
        var fin: List<0..=2_000_000_000> = List.filled(2 * size, 0)
        unchecked:
            for i in D:
                let p = size + i.ord
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

    /// The number of positions in `r` whose value is `< x`.
    ///
    /// cost: \(O(\log^2 \lvert D \rvert)\)
    /// empty: 0
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).count_lt(0..5, 3) == 2`
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

    /// The number of positions in `r` whose value is `<= x`.
    ///
    /// cost: \(O(\log^2 \lvert D \rvert)\)
    /// empty: 0
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).count_le(0..5, 3) == 3`
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

    /// The number of positions in `r` whose value lies in `lo_v..=hi_v`; 0 when `lo_v > hi_v`.
    ///
    /// cost: \(O(\log^2 \lvert D \rvert)\)
    /// empty: 0
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).count_in(1..4, 1, 4) == 3`
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).count_in(0..5, 4, 3) == 0`
    pub fn count_in(self, r: Range<D>, lo_v: T, hi_v: T) -> Int:
        if lo_v > hi_v: return 0
        self.count_le(r, hi_v) - self.count_lt(r, lo_v)

    /// The number of positions in `r` whose value is `> x`.
    ///
    /// cost: \(O(\log^2 \lvert D \rvert)\)
    /// empty: 0
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).count_gt(0..5, 3) == 2`
    pub fn count_gt(self, r: Range<D>, x: T) -> Int:
        let (lo, hi) = r.ends
        (hi - lo) - self.count_le(r, x)

    /// The number of positions in `r` whose value is `>= x`.
    ///
    /// cost: \(O(\log^2 \lvert D \rvert)\)
    /// empty: 0
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).count_ge(0..5, 4) == 2`
    pub fn count_ge(self, r: Range<D>, x: T) -> Int:
        let (lo, hi) = r.ends
        (hi - lo) - self.count_lt(r, x)

    /// The smallest value in `r` that is `>= x`, or `None`.
    ///
    /// cost: \(O(\log^2 \lvert D \rvert)\)
    /// empty: `None`
    /// ties: equal values are interchangeable; the value is returned, not a position
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).first_ge(0..3, 2) == Some(3)`
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).first_ge(0..3, 5) == None`
    pub fn first_ge(self, r: Range<D>, x: T) -> ?T:
        let (lo, hi) = r.ends
        var (l, h) = (lo + size, hi + size)
        var best: ?T = None
        unchecked:
            while l < h:
                if l % 2 == 1:
                    let j = beg[l] + self.lower_idx(l, x)
                    if j < fin[l] and (best == None or data[j] < best!): best = Some(data[j])
                    l += 1
                if h % 2 == 1:
                    h -= 1
                    let j = beg[h] + self.lower_idx(h, x)
                    if j < fin[h] and (best == None or data[j] < best!): best = Some(data[j])
                l /= 2
                h /= 2
        best

    /// The largest value in `r` that is `<= x`, or `None`.
    ///
    /// cost: \(O(\log^2 \lvert D \rvert)\)
    /// empty: `None`
    /// ties: equal values are interchangeable; the value is returned, not a position
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).last_le(2..5, 3) == Some(1)`
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).last_le(2..5, 0) == None`
    pub fn last_le(self, r: Range<D>, x: T) -> ?T:
        let (lo, hi) = r.ends
        var (l, h) = (lo + size, hi + size)
        var best: ?T = None
        unchecked:
            while l < h:
                if l % 2 == 1:
                    let j = beg[l] + self.upper_idx(l, x) - 1
                    if j >= beg[l] and (best == None or data[j] > best!): best = Some(data[j])
                    l += 1
                if h % 2 == 1:
                    h -= 1
                    let j = beg[h] + self.upper_idx(h, x) - 1
                    if j >= beg[h] and (best == None or data[j] > best!): best = Some(data[j])
                l /= 2
                h /= 2
        best

/// \(O(\lvert R \rvert \cdot \lvert C \rvert)\) space. Iterative 2D segment tree: point update and rectangle fold in \(O(\log \lvert R \rvert \cdot \log \lvert C \rvert)\).
pub struct SegTree2D<R, C, T>(m: Monoid<T>):
    t: List<T>

    fn fold_row(self, i: Int, cl: Int, ch: Int) -> T:
        let w = C.count
        var (l, h) = (cl + w, ch + w)
        var (left, right) = (m.id, m.id)
        while l < h:
            if l % 2 == 1:
                left = m.op(left, t[i * 2 * w + l])
                l += 1
            if h % 2 == 1:
                h -= 1
                right = m.op(t[i * 2 * w + h], right)
            l /= 2
            h /= 2
        m.op(left, right)

    /// A grid tree over `(R, C)` folding with `op`: built from `a`, filled with `fill`, or holding
    /// the identity everywhere. `op` must be `Commutative`, since a rectangle is folded in
    /// row-blocks and column-blocks rather than in one order.
    ///
    /// cost: \(O(\lvert R \rvert \cdot \lvert C \rvert)\)
    /// empty: an empty `R` or `C` gives a tree whose every fold is the identity
    /// order: cells keep the domains `R` and `C` of `a`
    /// traps: when both `a` and `fill` are given
    /// example: `SegTree2D<0..3, 0..3, Int>(+, fill: 1).all == 9`
    pub fn new<M>(op: M, a: ?[R, C]T = None, fill: ?T = None) -> SegTree2D<R, C, T>:
        let m = Monoid<T>.of(op)
        let (h, w) = (R.count, C.count)
        var g = SegTree2D<R, C, T>(m, List.filled(4 * h * w, m.id))
        if let init = a:
            assert fill == None, "pass either `a:` or `fill:`, not both"
            for r in R:
                for c in C: g.t[(h + r.ord) * 2 * w + w + c.ord] = init[r, c]
        elif let v = fill:
            for i in h..2 * h:
                for j in w..2 * w: g.t[i * 2 * w + j] = v
        for i in h..2 * h:
            for j in (1..w).rev: g.t[i * 2 * w + j] = m.op(
                g.t[i * 2 * w + 2 * j],
                g.t[i * 2 * w + 2 * j + 1],
            )
        for i in (1..h).rev:
            for j in 1..2 * w: g.t[i * 2 * w + j] = m.op(
                g.t[2 * i * 2 * w + j],
                g.t[(2 * i + 1) * 2 * w + j],
            )
        g

    /// The value of cell `(r, c)`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `r` is outside `R` or `c` is outside `C`
    /// example: `var g = SegTree2D<0..3, 0..3, Int>(+, fill: 1); g.set(1, 2, 7); g.get(1, 2) == 7`
    pub fn get(self, r: R, c: C) -> T:
        t[(R.count + r.ord) * 2 * C.count + C.count + c.ord]

    /// Make cell `(r, c)` hold `x`.
    ///
    /// cost: \(O(\log \lvert R \rvert \cdot \log \lvert C \rvert)\)
    /// traps: when `r` is outside `R` or `c` is outside `C`
    /// example: `var g = SegTree2D<0..3, 0..3, Int>(+, fill: 1); g.set(0, 0, 5); g.all == 13`
    pub fn set(mut self, r: R, c: C, x: T):
        let (h, w) = (R.count, C.count)
        var i = h + r.ord
        let j0 = w + c.ord
        t[i * 2 * w + j0] = x
        var j = j0 / 2
        while j >= 1:
            t[i * 2 * w + j] = m.op(t[i * 2 * w + 2 * j], t[i * 2 * w + 2 * j + 1])
            j /= 2
        i /= 2
        while i >= 1:
            j = j0
            while j >= 1:
                t[i * 2 * w + j] = m.op(t[2 * i * 2 * w + j], t[(2 * i + 1) * 2 * w + j])
                j /= 2
            i /= 2

    /// Replace the value `v` of cell `(r, c)` with `f(v)`.
    ///
    /// cost: \(O(\log \lvert R \rvert \cdot \log \lvert C \rvert)\) plus one call of `f`
    /// traps: when `r` is outside `R` or `c` is outside `C`
    /// example: `var g = SegTree2D<0..3, 0..3, Int>(+, fill: 1); g.update(2, 2, x => x + 1); g.all == 10`
    pub fn update(mut self, r: R, c: C, f: T -> T):
        self.set(r, c, f(self.get(r, c)))

    /// The fold of every cell in rows `rows` and columns `cols`.
    ///
    /// cost: \(O(\log \lvert R \rvert \cdot \log \lvert C \rvert)\)
    /// empty: the identity when `rows` or `cols` is empty
    /// traps: when a non-empty range reaches outside its domain
    /// example: `SegTree2D<0..3, 0..3, Int>(+, fill: 1).fold(0..2, 1..3) == 4`
    /// example: `SegTree2D<0..3, 0..3, Int>(+, fill: 1).fold(0..2, 1..1) == 0`
    pub fn fold(self, rows: Range<R>, cols: Range<C>) -> T:
        let (rl, rh) = rows.ends
        let (cl, ch) = cols.ends
        var (l, hh) = (rl + R.count, rh + R.count)
        var (left, right) = (m.id, m.id)
        while l < hh:
            if l % 2 == 1:
                left = m.op(left, self.fold_row(l, cl, ch))
                l += 1
            if hh % 2 == 1:
                hh -= 1
                right = m.op(self.fold_row(hh, cl, ch), right)
            l /= 2
            hh /= 2
        m.op(left, right)

    /// The fold of every cell.
    ///
    /// cost: \(O(1)\)
    /// empty: the identity for an empty grid
    /// traps: never
    /// example: `var g = SegTree2D<0..2, 0..2, Int>(max, fill: 0); g.set(1, 0, 8); g.all == 8`
    @get pub fn all(self) -> T:
        if R.count == 0 or C.count == 0: m.id else: t[2 * C.count + 1]

/// \(O(\lvert D \rvert \log V)\) build and memory. Wavelet matrix over non-negative integer values:
/// \(O(\log V)\) range k-th smallest, frequency, value-range count, predecessor/successor, and quantile sum.
pub struct WaveletMatrix<D>:
    bits: Int
    n: Int
    zs: List<Int>
    cnt: List<Int>
    zsum: List<Int>

    fn ones(self, l: Int, i: Int) -> Int = cnt[l * (n + 1) + i]

    fn below(self, lo0: Int, hi0: Int, x: Int) -> Int:
        if x <= 0: return 0
        if bits < 62 and x >= 1 << bits: return hi0 - lo0
        var (lo, hi, res) = (lo0, hi0, 0)
        for l in 0..bits:
            let (zl, zh) = (lo - self.ones(l, lo), hi - self.ones(l, hi))
            if x.bit((bits - 1 - l)):
                res += zh - zl
                (lo, hi) = (zs[l] + self.ones(l, lo), zs[l] + self.ones(l, hi))
            else:
                (lo, hi) = (zl, zh)
        res

    /// A wavelet matrix over the non-negative values of `a`, with one bit level per bit of the
    /// largest value (`V` below is that value plus one).
    ///
    /// cost: \(O(\lvert D \rvert \log V)\) time and memory
    /// empty: an empty `a` gives a matrix where every count is 0
    /// order: positions keep the domain `D` of `a`
    /// traps: when some value is negative
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).kth(0..5, 2) == Some(3)`
    pub fn new(a: [D]Int) -> WaveletMatrix<D>:
        var cur = [a[i] for i in D]
        let n = cur.len
        var top = 0
        for v in cur:
            assert v >= 0, "`WaveletMatrix` needs non-negative values"
            top max= v
        var bits = 0
        while (top >> bits) > 0: bits += 1
        var w = WaveletMatrix<D>(
            bits,
            n,
            [],
            List.filled(bits * (n + 1), 0),
            List.filled(bits * (n + 1), 0),
        )
        for l in 0..bits:
            let b = bits - 1 - l
            let base = l * (n + 1)
            var (zero, one): (List<Int>, List<Int>) = ([], [])
            for i in 0..n:
                let v = cur[i]
                if v.bit(b):
                    w.cnt[base + i + 1] = w.cnt[base + i] + 1
                    w.zsum[base + i + 1] = w.zsum[base + i]
                    one.push(v)
                else:
                    w.cnt[base + i + 1] = w.cnt[base + i]
                    w.zsum[base + i + 1] = w.zsum[base + i] + v
                    zero.push(v)
            w.zs.push(zero.len)
            for v in one: zero.push(v)
            cur = zero
        w

    /// The `k`-th smallest value in `r`, counting from 0 and with repeats, or `None` when
    /// `k >= r.len`.
    ///
    /// cost: \(O(\log V)\)
    /// empty: `None`
    /// ties: equal values are counted once each
    /// traps: when `k < 0`, or a non-empty `r` reaches outside `D`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).kth(0..5, 0) == Some(1)`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).kth(0..5, 2) == Some(3)`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).kth(0..5, 5) == None`
    pub fn kth(self, r: Range<D>, k: Int) -> ?Int:
        assert k >= 0, "`kth` needs `k >= 0`"
        var (lo, hi) = r.ends
        if k >= hi - lo: return None
        var rem = k
        var res = 0
        for l in 0..bits:
            let (zl, zh) = (lo - self.ones(l, lo), hi - self.ones(l, hi))
            if rem < zh - zl:
                (lo, hi) = (zl, zh)
            else:
                rem -= zh - zl
                res |= 1 << (bits - 1 - l)
                (lo, hi) = (zs[l] + self.ones(l, lo), zs[l] + self.ones(l, hi))
        Some(res)

    /// The number of positions in `r` holding exactly `x`; 0 for any `x` outside `0..V`.
    ///
    /// cost: \(O(\log V)\)
    /// empty: 0
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).rank(0..5, 1) == 2`
    pub fn rank(self, r: Range<D>, x: Int) -> Int:
        let (lo, hi) = r.ends
        if x < 0: return 0
        self.below(lo, hi, x + 1) - self.below(lo, hi, x)

    /// The number of positions in `r` whose value is `< x`.
    ///
    /// cost: \(O(\log V)\)
    /// empty: 0
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).count_lt(0..5, 4) == 3`
    pub fn count_lt(self, r: Range<D>, x: Int) -> Int:
        let (lo, hi) = r.ends
        self.below(lo, hi, x)

    /// The number of positions in `r` whose value lies in `lo_v..=hi_v`; 0 when `lo_v > hi_v`.
    ///
    /// cost: \(O(\log V)\)
    /// empty: 0
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).count_in(0..5, 1, 3) == 3`
    pub fn count_in(self, r: Range<D>, lo_v: Int, hi_v: Int) -> Int:
        let (lo, hi) = r.ends
        if lo_v > hi_v: return 0
        self.below(lo, hi, hi_v + 1) - self.below(lo, hi, lo_v)

    /// The largest value in `r` that is `< x`, or `None`.
    ///
    /// cost: \(O(\log V)\)
    /// empty: `None`
    /// ties: equal values are interchangeable; the value is returned, not a position
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).prev_val(0..5, 3) == Some(1)`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).prev_val(0..5, 1) == None`
    pub fn prev_val(self, r: Range<D>, x: Int) -> ?Int:
        let (lo, hi) = r.ends
        let c = self.below(lo, hi, x)
        if c == 0: None else: self.kth(r, c - 1)

    /// The smallest value in `r` that is `> x`, or `None`.
    ///
    /// cost: \(O(\log V)\)
    /// empty: `None`
    /// ties: equal values are interchangeable; the value is returned, not a position
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).next_val(0..5, 3) == Some(4)`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).next_val(0..5, 5) == None`
    pub fn next_val(self, r: Range<D>, x: Int) -> ?Int:
        let (lo, hi) = r.ends
        let c = self.below(lo, hi, x + 1)
        if c >= hi - lo: None else: self.kth(r, c)

    /// The sum of the `k` smallest values in `r`, with repeats.
    ///
    /// cost: \(O(\log V)\)
    /// empty: 0 (only `k == 0` is allowed)
    /// ties: equal values are counted once each
    /// traps: when `k` is outside `0..=r.len`, or a non-empty `r` reaches outside `D`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).quantile_sum(0..5, 3) == 5`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).quantile_sum(1..3, 0) == 0`
    pub fn quantile_sum(self, r: Range<D>, k: Int) -> Int:
        var (lo, hi) = r.ends
        assert 0 <= k <= hi - lo, "`quantile_sum` needs `k` in `0..=r.len`"
        var rem = k
        var (res, total) = (0, 0)
        for l in 0..bits:
            if rem == 0: return total
            let base = l * (n + 1)
            let (zl, zh) = (lo - self.ones(l, lo), hi - self.ones(l, hi))
            if rem < zh - zl:
                (lo, hi) = (zl, zh)
            else:
                rem -= zh - zl
                total += zsum[base + hi] - zsum[base + lo]
                res |= 1 << (bits - 1 - l)
                (lo, hi) = (zs[l] + self.ones(l, lo), zs[l] + self.ones(l, hi))
        total + rem * res

/// \(O(\lvert D \rvert \log^2 \lvert D \rvert)\) space. Dynamic point-update + 2D range value-count and k-th structure
/// (Fenwick of order trees) over `[D]T`.
pub struct DynamicOrderTree<D, T>:
    n: Int
    val: List<T> = []
    // `bit[j]` for `j >= 1` holds the values at positions `j - (j & -j)..j`; `bit[0]` holds
    // every value, the candidates `kth` searches.
    bit: List<SortedBag<T>> = []

    fn bump(mut self, i: Int, x: T, put: Bool):
        if put: bit[0].add(x)
        else: bit[0].remove(x)
        var j = i + 1
        while j <= n:
            if put: bit[j].add(x)
            else: bit[j].remove(x)
            j += j & -j

    fn below(self, end: Int, x: T, le: Bool) -> Int:
        var acc = 0
        var j = end
        while j > 0:
            acc += bit[j].rank(x)
            if le: acc += bit[j].count(x)
            j -= j & -j
        acc

    fn span_count(self, r: Range<D>, x: T, le: Bool) -> Int:
        if r.is_empty: return 0
        let (lo, hi) = (r.first!.ord, r.last!.ord + 1)
        self.below(hi, x, le) - self.below(lo, x, le)

    /// A structure over `a` answering value counts and order statistics on ranges while positions
    /// are reassigned.
    ///
    /// cost: \(O(\lvert D \rvert \log^2 \lvert D \rvert)\) time and memory
    /// empty: an empty `a` gives a tree where every count is 0
    /// order: positions keep the domain `D` of `a`
    /// traps: never
    /// example: `DynamicOrderTree([3, 1, 4]).count_lt(0..3, 4) == 2`
    pub fn new(a: [D]T) -> DynamicOrderTree<D, T>:
        var t = DynamicOrderTree<D, T>(
            D.count,
            [a[i] for i in D],
            [SortedBag<T>.new() for _ in 0..D.count + 1],
        )
        for i in 0..D.count: t.bump(i, t.val[i], true)
        t

    /// Make position `i` hold `x`.
    ///
    /// cost: \(O(\log^2 \lvert D \rvert)\)
    /// traps: when `i` is outside `D`
    /// example: `var t = DynamicOrderTree([3, 1, 4]); t.set(0, 0); t.kth(0..3, 0) == Some(0)`
    pub fn set(mut self, i: D, x: T):
        let k = i.ord
        self.bump(k, val[k], false)
        val[k] = x
        self.bump(k, x, true)

    /// The value at position `i`.
    ///
    /// cost: \(O(1)\)
    /// traps: when `i` is outside `D`
    /// example: `var t = DynamicOrderTree([3, 1, 4]); t.set(2, 9); t.get(2) == 9`
    pub fn get(self, i: D) -> T = val[i.ord]

    /// The number of positions in `r` whose value is `< x`.
    ///
    /// cost: \(O(\log^2 \lvert D \rvert)\)
    /// empty: 0
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `DynamicOrderTree([3, 1, 4]).count_lt(1..3, 4) == 1`
    pub fn count_lt(self, r: Range<D>, x: T) -> Int = self.span_count(r, x, false)

    /// The number of positions in `r` whose value lies in `lo_v..=hi_v`; 0 when `lo_v > hi_v`.
    ///
    /// cost: \(O(\log^2 \lvert D \rvert)\)
    /// empty: 0
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `DynamicOrderTree([3, 1, 4]).count_in(0..3, 1, 3) == 2`
    pub fn count_in(self, r: Range<D>, lo_v: T, hi_v: T) -> Int:
        if hi_v < lo_v: return 0
        self.span_count(r, hi_v, true) - self.span_count(r, lo_v, false)

    /// The `k`-th smallest value in `r`, counting from 0 and with repeats, or `None` when
    /// `k >= r.len`.
    ///
    /// cost: \(O(\log^3 \lvert D \rvert)\): a binary search over all stored values, counting each in `r`
    /// empty: `None`
    /// ties: equal values are counted once each
    /// traps: when `k < 0`, or a non-empty `r` reaches outside `D`
    /// example: `DynamicOrderTree([3, 1, 4]).kth(0..3, 1) == Some(3)`
    /// example: `DynamicOrderTree([3, 1, 4]).kth(0..3, 3) == None`
    pub fn kth(self, r: Range<D>, k: Int) -> ?T:
        assert k >= 0, "`kth` takes `k >= 0`"
        if k >= r.len: return None
        let lo = (0..bit[0].len - 1).first_true(mid => self.span_count(r, bit[0].nth(mid)!, true) > k) ?? (bit[0].len - 1)
        bit[0].nth(lo)

/// \(O(q \log \lvert D \rvert)\) space. Mergeable min-ordered set over domain `D` for lazily pruned
/// minimum-alive queries (`MergeSeg.singleton`, `merge`, `first_alive`).
pub struct MergeSeg<D>:
    heap: List<D>

    fn push_raw(mut self, x: D):
        with lint(unproven_list_index: allow)
        heap.push(x)
        var i = heap.len - 1
        while i > 0:
            let p = (i - 1)  // 2
            if heap[i] < heap[p]:
                let tmp = heap[p]
                heap[p] = heap[i]
                heap[i] = tmp
                i = p
            else:
                break

    fn pop_min(mut self):
        with lint(unproven_list_index: allow)
        let n = heap.len
        if n <= 1:
            heap.clear()
            return
        heap[0] = heap[n - 1]
        heap.pop()
        let m = heap.len
        var i = 0
        while true:
            let l = 2 * i + 1
            let r = l + 1
            var best = i
            if l < m and heap[l] < heap[best]: best = l
            if r < m and heap[r] < heap[best]: best = r
            if best == i: break
            let tmp = heap[i]
            heap[i] = heap[best]
            heap[best] = tmp
            i = best

    /// A set of positions of `D` holding only `x`, stored as a sparse binary tree over `D`.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// traps: when `x` is outside `D`
    /// example: `var t = MergeSeg<0..10>.singleton(3); t.first_alive(x => true) == Some(3)`
    pub fn singleton(x: D) -> MergeSeg<D> = MergeSeg([x])

    /// The union of `self` and `other`. The result reuses the nodes of both, so neither input may be
    /// used afterwards.
    ///
    /// cost: \(O(\log \lvert D \rvert)\) amortized per node absorbed
    /// traps: never
    /// example: `var m = MergeSeg<0..10>.singleton(7).merge(MergeSeg<0..10>.singleton(3)); m.first_alive(x => true) == Some(3)`
    pub fn merge(self, other: MergeSeg<D>) -> MergeSeg<D>:
        if heap.len >= other.heap.len:
            var out = MergeSeg(heap)
            for x in other.heap: out.push_raw(x)
            out
        else:
            var out = MergeSeg(other.heap)
            for x in heap: out.push_raw(x)
            out

    /// The smallest position `x` in the set with `alive(x)`, or `None`. Every position found dead on
    /// the way is removed for good, so `alive` must be monotone over time: a position never comes
    /// back to life.
    ///
    /// cost: \(O(\log \lvert D \rvert)\) amortized
    /// ties: the smallest position in key order
    /// traps: never
    /// example: `var m = MergeSeg<0..10>.singleton(3).merge(MergeSeg<0..10>.singleton(7)); m.first_alive(x => x > 5) == Some(7)`
    /// example: `var m = MergeSeg<0..10>.singleton(3).merge(MergeSeg<0..10>.singleton(7)); m.first_alive(x => x > 5); m.first_alive(x => true) == Some(7)`
    pub fn first_alive(mut self, alive: D -> Bool) -> ?D:
        with lint(unproven_list_index: allow)
        while heap.len > 0:
            let x = heap[0]
            if alive(x): return Some(x)
            while heap.len > 0 and heap[0] == x:
                self.pop_min()
        None

@test
fn merge_sort_and_sparse_segtree_match_brute():
    domain I = 1..=14
    let a = [I] _ => rand(-10, 10)
    let mst = MergeSortTree<I, Int>(a)
    fn cnt(r: Range<I>, ok: Int -> Bool) -> Int:
        let c = count(ok(a[i]) for i in r)
        c
    for l in 1..=14:
        for r in l..=14:
            let rng = I(l)..=I(r)
            for v in [-11, -5, 0, 5, 11]:
                assert mst.count_lt(rng, v) == cnt(rng, x => x < v)
                assert mst.count_le(rng, v) == cnt(rng, x => x <= v)
            assert mst.count_in(rng, -3, 4) == cnt(rng, x => -3 <= x <= 4)
