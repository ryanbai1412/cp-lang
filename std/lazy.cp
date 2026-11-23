//! Range-update helpers that are not segment trees: `Flip`, `LazyTreap`, `Rope`. Range apply on a segment tree is `SegTree(op, a, act:)` (`segtree.cp`).
use std.algebra.{Action, MaxCount, MinCount, Monoid, add, affine, assign}
use std.rand.Rng
use std.segtree.{SegTree}
use std.seq.{ends}

/// Paired aggregate `(now, flipped)` for range-toggle / complement updates (`SegTree(op, a, act: flip)`).
/// Build the toggle action with `flip_action(m)` (tag `Bool`: `true` swaps `now` and `flipped`).
pub struct Flip<T>(pub now: T, pub flipped: T):
    /// The leaf for one element that is currently `on` or off: `Flip(x, id)` when `on`, else
    /// `Flip(id, x)`. `now` is the value the element contributes as it stands and `flipped` what it
    /// would contribute after one toggle.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Flip.of(true, 5, 0).now == 5`
    /// example: `Flip.of(false, 5, 0).flipped == 5`
    pub fn of(on: Bool, x: T, id: T) -> Flip<T>:
        if on: Flip(x, id) else: Flip(id, x)

    /// The monoid on pairs that folds both halves with `m` independently:
    /// `op(a, b) == Flip(m(a.now, b.now), m(a.flipped, b.flipped))`, identity `Flip(id, id)` with
    /// `id` from `Monoid.of(m)`.
    ///
    /// cost: \(O(1)\)
    /// traps: never (an `m` without a known identity is a compile-time error)
    /// example: `Flip.op(+).op(Flip(1, 0), Flip(0, 1)).now == 1`
    /// example: `Flip.op(+).id.flipped == 0`
    pub fn op<M>(m: M) -> Monoid<Flip<T>>:
        let b = Monoid<T>.of(m)
        Monoid(Flip(b.id, b.id), (x, y) => Flip(b.op(x.now, y.now), b.op(x.flipped, y.flipped)))

/// The range-toggle action on `Flip<T>` values over `m`: the tag is a `Bool`, `true` swaps
/// `now` and `flipped`, `false` is the identity, and tags compose by exclusive or. The swap
/// needs no span, since both halves were folded with the same `m`.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `flip_action(+).apply(true, Flip(3, 1), 4).now == 1`
/// example: `flip_action(+).compose(true, true) == false`
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

fn brute<D>(a: [D]Int, r: Range<D>, pick: Int) -> Int:
    if pick == 0: return sum(a[v] for v in r)
    if pick == 1: return [a[v] for v in r].min() ?? Int.MAX
    [a[v] for v in r].max() ?? Int.MIN

fn act_laws<T, F>(act: Action<T, F>, ts: List<T>, fs: List<F>) -> Bool:
    for x in ts:
        if act.apply(act.id, x, 1) != x: return false
        for y in ts:
            for f in fs:
                if act.apply(f, act.over.op(x, y), 2) != act.over.op(
                    act.apply(f, x, 1),
                    act.apply(f, y, 1),
                ): return false
        for f in fs:
            for g in fs:
                if act.apply(act.compose(f, g), x, 1) != act.apply(
                    f,
                    act.apply(g, x, 1),
                    1,
                ): return false
    true

@test
fn the_stock_actions_match_brute_force():
    domain V = 1..=10
    for k in 0..3:
        var a = [V] v => rand(-9, 9)
        var st = if k == 0: SegTree(+, a.clone(), act: add) elif k == 1: SegTree(
            min,
            a.clone(),
            act: add,
        ) else: SegTree(
            max,
            a.clone(),
            act: add,
        )
        var sa = if k == 0: SegTree(+, a.clone(), act: assign) elif k == 1: SegTree(
            min,
            a.clone(),
            act: assign,
        ) else: SegTree(
            max,
            a.clone(),
            act: assign,
        )
        var b = a.clone()
        for _ in 0..60:
            let l = V.nth(rand(0, 9))
            let r = V.nth(rand(l.ord, 9))
            if rand(0, 1) == 0:
                let x = rand(-5, 5)
                for v in l..=r:
                    a[v] += x
                    b[v] = x
                st.apply(l..=r, x)
                sa.apply(l..=r, Some(x))
            else:
                assert st.fold(l..=r) == brute(a, l..=r, k) and sa.fold(l..=r) == brute(b, l..=r, k)
                assert st.fold(..l) == brute(a, ..l, k) and sa.fold(..l) == brute(b, ..l, k)
                assert st.fold(l..) == brute(a, l.., k) and sa.fold(l..) == brute(b, l.., k)
        for v in V: assert st.get(v) == a[v] and sa.get(v) == b[v]
        assert st.all == brute(a, V.nth(0)..=V.nth(9), k) and sa.all == brute(
            b,
            V.nth(0)..=V.nth(9),
            k,
        )

@test
fn at_writes_through_pending_tags():
    domain V = 1..=10
    var a = [V] v => rand(-9, 9)
    var st = SegTree(+, a.clone(), act: add)
    for _ in 0..60:
        let l = V.nth(rand(0, 9))
        let r = V.nth(rand(l.ord, 9))
        let x = rand(-5, 5)
        if rand(0, 1) == 0:
            for v in l..=r: a[v] += x
            st.apply(l..=r, x)
        else:
            a[l] *= 2
            st.at(l) *= 2
        assert st.fold(l..=r) == sum(a[v] for v in l..=r)
    for v in V: assert st.get(v) == a[v]

@test
fn stock_actions_obey_their_laws():
    for _ in 0..1:
        let ts = [-3, 0, 8]
        let fs = [-4, 0, 6]
        let sum_add = Action<Int, Int>.of(add, +)
        assert act_laws(sum_add, ts, fs)
        assert act_laws(Action<Int, Int>.of(add, min), ts, fs)
        assert act_laws(Action<Int, Int>.of(add, max), ts, fs)
        let gs = [None, Some(-4), Some(0), Some(6)]
        assert act_laws(Action<Int, ?Int>.of(assign, +), ts, gs)
        assert act_laws(Action<Int, ?Int>.of(assign, min), ts, gs)
        assert act_laws(Action<Int, ?Int>.of(assign, max), ts, gs)

@test
fn set_and_an_empty_tree():
    domain V = 0..6
    var st = SegTree<V, Int>(+, act: add)
    st.apply(V(1)..=V(4), 3)
    st.apply(V(3)..=V(5), 10)
    assert st.fold(V(0)..=V(5)) == 4 * 3 + 3 * 10
    assert st.get(V(4)) == 13
    st.set(V(4), 100)
    assert st.fold(V(3)..=V(5)) == 13 + 100 + 10
    assert st.fold(V(2)..V(2)) == 0

@test
fn padding_never_takes_an_update():
    domain V = 0..3
    var st = SegTree(max, [V] v => 0, act: add)
    st.apply(V(0)..=V(2), -1)
    st.apply(V(0)..=V(1), 1)
    assert st.fold(V(0)..=V(2)) == 0 and st.get(V(2)) == -1
    assert st.fold(V(1)..V(1)) == Int.MIN
    var mn = SegTree(min, [V] v => 0, act: add)
    mn.apply(V(0)..=V(2), 1)
    mn.apply(V(0)..=V(1), -1)
    assert mn.fold(V(0)..=V(2)) == 0 and mn.get(V(2)) == 1
    assert mn.fold(V(1)..V(1)) == Int.MAX and mn.fold(V(2).r..) == Int.MAX
    assert mn.fold(V(1)..) == 0 and mn.all == 0

@test
fn an_action_of_your_own():
    let count_max = Monoid<(Int, Int)>(
        (Int.MIN, 0),
        (a, b) => if a.0 > b.0: a else: if b.0 > a.0: b else: (a.0, a.1 + b.1),
    )
    let shift = Action<(Int, Int), Int>(
        0,
        (f, g) => f + g,
        (f, x, len) => (x.0 + f, x.1),
        count_max,
    )
    domain V = 0..8
    var a = [V] v => 0
    var st = SegTree(count_max, [V] v => (0, 1), act: shift)
    for _ in 0..60:
        let l = V.nth(rand(0, 7))
        let r = V.nth(rand(l.ord, 7))
        let x = rand(-3, 3)
        for v in l..=r: a[v] += x
        st.apply(l..=r, x)
        let top = [a[v] for v in V].max()!
        assert st.all == (top, [a[v] for v in V].count(x => x == top))

@test
fn right_while_and_left_while_see_pending_updates():
    domain V = 0..9
    var a = [V] v => rand(0, 5)
    var st = SegTree(+, a.clone(), act: add)
    for _ in 0..80:
        let l = V.nth(rand(0, 8))
        let r = V.nth(rand(l.ord, 8))
        let x = rand(0, 3)
        for v in l..=r: a[v] += x
        st.apply(l..=r, x)
        let lim = rand(0, 40)
        let c = rand(0, 9)
        var right = c
        var acc = 0
        while right < 9 and acc + a[V.nth(right)] <= lim:
            acc += a[V.nth(right)]
            right += 1
        assert st.right_while(V.cuts.nth(c), s => s <= lim) == V.cuts.nth(right)
        var left = c
        acc = 0
        while left > 0 and acc + a[V.nth(left - 1)] <= lim:
            acc += a[V.nth(left - 1)]
            left -= 1
        assert st.left_while(V.cuts.nth(c), s => s <= lim) == V.cuts.nth(left)

fn ccc_compose(f: (?Int, ?Int, Int), g: (?Int, ?Int, Int)) -> (?Int, ?Int, Int):
    let nb = f.0 ?? g.0
    if f.1 != None: return (nb, f.1, 0)
    if g.1 != None: return (nb, Some(g.1! + f.2), 0)
    (nb, None, g.2 + f.2)

fn ccc_apply(f: (?Int, ?Int, Int), x: (Int, Int, Int), len: Int) -> (Int, Int, Int):
    let b_min = f.0 ?? x.1
    if f.1 != None:
        let a_val = b_min + f.1!
        return (a_val, b_min, a_val * len)
    (x.0 + f.2, b_min, x.2 + f.2 * len)

@test
fn ccc02s4hard_dual_field_cross_overwrite_and_interleaved_point_set():
    let over = Monoid<(Int, Int, Int)>(
        (Int.MAX, Int.MAX, 0),
        (x, y) => (min(x.0, y.0), min(x.1, y.1), x.2 + y.2),
    )
    let act = Action<(Int, Int, Int), (?Int, ?Int, Int)>(
        (None, None, 0),
        ccc_compose,
        ccc_apply,
        over,
    )
    domain V = 0..11
    var a = [V] v => rand(-10, 10)
    var b = [V] v => rand(-10, 10)
    var st = SegTree(over, [V] v => (a[v], b[v], a[v]), act:)
    for _ in 0..120:
        let kind = rand(0, 3)
        let l = V.nth(rand(0, 10))
        let r = V.nth(rand(l.ord, 10))
        if kind == 0:
            let d = rand(-5, 5)
            for v in l..=r: a[v] += d
            st.apply(l..=r, (None, None, d))
        elif kind == 1:
            let vb = rand(-8, 8)
            let d = rand(-4, 4)
            for v in l..=r:
                b[v] = vb
                a[v] = vb + d
            st.apply(l..=r, (Some(vb), Some(d), 0))
        elif kind == 2:
            let na = rand(-15, 15)
            let nb = rand(-15, 15)
            a[l] = na
            b[l] = nb
            st.set(l, (na, nb, na))
        else:
            let want = (
                [a[v] for v in l..=r].min()!,
                [b[v] for v in l..=r].min()!,
                sum(a[v] for v in l..=r),
            )
            assert st.fold(l..=r) == want
    for v in V: assert st.get(v) == (a[v], b[v], a[v])

@test
fn update_pushes_modifies_and_pulls_in_one_pass():
    domain V = 1..=9
    var a = [V] v => rand(-10, 10)
    var st = SegTree(+, a.clone(), act: add)
    for _ in 0..80:
        let kind = rand(0, 3)
        let l = V.nth(rand(0, 8))
        let r = V.nth(rand(l.ord, 8))
        if kind == 0:
            let d = rand(-7, 7)
            for v in l..=r: a[v] += d
            st.apply(l..=r, d)
        elif kind == 1:
            let k = rand(-5, 5)
            a[l] = a[l] * 2 + k
            st.update(l, cur => cur * 2 + k)
        elif kind == 2:
            let x = rand(-20, 20)
            a[l] = x
            st.set(l, x)
        else:
            assert st.fold(l..=r) == sum(a[v] for v in l..=r)
            assert st.all == sum(a[v] for v in V)
    for v in V: assert st.get(v) == a[v]

@test
fn custom_clamp_action_with_update():
    let clamp = Action<Int, (Int, Int)>(
        (Int.MIN, Int.MAX),
        (nf, of) => (min(max(of.0, nf.0), nf.1), max(min(of.1, nf.1), nf.0)),
        (f, val, span) => min(max(val, f.0), f.1),
        Monoid<Int>.of(max),
    )
    domain V = 0..6
    let init = [3, -4, 9, 0, 5, -2]
    var a = [V] v => init[v.ord]
    var st = SegTree(max, a.clone(), act: clamp)
    for _ in 0..50:
        let l = V.nth(rand(0, 5))
        let r = V.nth(rand(l.ord, 5))
        let lo = rand(-6, 4)
        let hi = rand(lo, 8)
        for v in l..=r: a[v] = min(max(a[v], lo), hi)
        st.apply(l..=r, (lo, hi))
        let i = V.nth(rand(0, 5))
        a[i] += 1
        st.update(i, x => x + 1)
        assert st.fold(l..=r) == [a[v] for v in l..=r].max()!
        assert st.all == [a[v] for v in V].max()!
    for v in V: assert st.get(v) == a[v]

@test
fn fill_and_affine_sum_match_brute():
    domain V = 0..7
    var dp = SegTree<V, Int>(max, fill: 0, act: add)
    assert dp.all == 0
    dp.apply(V(1)..=V(4), 5)
    assert dp.all == 5 and dp.get(V(0)) == 0 and dp.get(V(3)) == 5
    var a = [V] _ => rand(-5, 5)
    var st = SegTree<V, Int, (Int, Int)>(+, a.clone(), act: affine)
    for _ in 0..50:
        let l = V.nth(rand(0, 6))
        let r = V.nth(rand(l.ord, 6))
        let mul = rand(-2, 3)
        let k = rand(-4, 4)
        for v in l..=r: a[v] = a[v] * mul + k
        st.apply(l..=r, (mul, k))
        assert st.fold(l..=r) == sum(a[v] for v in l..=r)

@test
fn dual_segtree_non_commutative_tags_match_brute():
    domain V = 1..=11
    var a = [V] _ => rand(-5, 5)
    let aff = Action<Int, (Int, Int)>(
        (1, 0),
        (nf, of) => (nf.0 * of.0, nf.0 * of.1 + nf.1),
        (f, x, n) => f.0 * x + f.1 * n,
        Monoid<Int>.of(+),
    )
    var st = SegTree(+, a.clone(), act: aff)
    for _ in 0..100:
        let kind = rand(0, 3)
        let l = V.nth(rand(0, 10))
        let r = V.nth(rand(l.ord, 10))
        if kind <= 1:
            let (m, b) = (rand(-2, 2), rand(-6, 6))
            for v in l..=r: a[v] = m * a[v] + b
            st.apply(l..=r, (m, b))
        elif kind == 2:
            let x = rand(-10, 10)
            a[l] = x
            st.set(l, x)
        else:
            a[l] = a[l] * 3 - 1
            st.update(l, x => x * 3 - 1)
        let q = V.nth(rand(0, 10))
        assert st.get(q) == a[q]
    for v in V: assert st.get(v) == a[v]

struct TreapNode<T, F>:
    var l: ?Int
    var r: ?Int
    pri: 0..2_147_483_648
    var sz: 0..2_147_483_648
    var val: T
    var tag: ?F
    var rev: Bool

/// Arena-backed randomized treap with subtree-size-aware lazy tag propagation.
/// Supports value-ordered insertion (`insert`, using `before`) for slope-trick / priority
/// structures with lazy updates, and 0-based implicit sequence operations (`push`, `insert_at`,
/// `apply_range`, `get`).
/// `apply(v, left_sz, f)` updates a node's value given its left subtree size;
/// `split(f, left_sz)` splits a tag `f` for the left and right subtrees (default `(f, f)`);
/// `compose(new_f, old_f)` applies `old_f` first then `new_f`.
/// `insert`, `insert_at`, `push`, `apply_range`, `get` are \(O(\log n)\) expected; `apply_all` is \(O(1)\);
/// `take(k)` is \(O(k + \log n)\); `to_list` is \(O(n)\). Memory is \(O(n)\) in a contiguous node arena;
/// sizes and priorities are `0..2^31`, so a treap holds fewer than 2^31 elements.
pub struct LazyTreap<T, F>(
    apply_fn: (T, Int, F) -> T,
    split_fn: (F, Int) -> (F, F),
    compose_fn: (F, F) -> F,
    before_fn: (T, T) -> Bool,
):
    var nodes: List<TreapNode<T, F>>
    var root: ?Int
    rng: Rng

    /// An empty treap. `apply(v, k, f)` is element `v` after tag `f`, where `k` is the number of
    /// elements before it in the run the tag was applied to (so tags may depend on rank);
    /// `compose(new_f, old_f)` is `old_f` then `new_f`; `split(f, k)` gives the tags for the left
    /// child and for the right child of a node with `k` elements on its left (default `(f, f)`, for
    /// tags that ignore rank; a rank-dependent tag needs a `split` that shifts the right child's ranks
    /// by `k + 1`); `before(x, y)` orders value insertion (default never, so `insert` appends).
    ///
    /// cost: \(O(1)\)
    /// empty: the new treap is empty
    /// traps: never
    /// example: `LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f).len == 0`
    pub fn new(
        apply: (T, Int, F) -> T,
        compose: (F, F) -> F,
        split: (F, Int) -> (F, F) = (f, _) => (f, f),
        before: (T, T) -> Bool = (_, _) => false,
    ) -> LazyTreap<T, F>:
        LazyTreap(apply, split, compose, before, [], None, Rng(48))

    fn priority(mut self) -> Int = Int(rng.next() >> 33)

    fn sz_of(self, u: ?Int) -> Int:
        if let i = u:
            unchecked: return nodes[i].sz
        0

    /// The number of elements stored.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(4); t.len == 1`
    @get pub fn len(self) -> Int = self.sz_of(root)

    /// Make room for `n` elements in total, so pushes up to that size never regrow the node arena.
    /// A smaller `n` than the current capacity changes nothing.
    ///
    /// cost: \(O(n)\) when it grows, else \(O(1)\)
    /// traps: when `n >= 2^31` (the size bound)
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.reserve(10); t.len == 0`
    pub fn reserve(mut self, n: Int) = root == None

    /// Whether the treap holds no elements.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f).is_empty == true`
    @get pub fn is_empty(self) -> Bool = root == None

    // A node's subtree reads as its children in stored order, reversed when `rev` is set, then
    // `tag` applied over that order. `tag` is always the newer of the two.
    fn put(mut self, u: ?Int, f: F):
        if let i = u:
            unchecked:
                let ls = self.sz_of(if nodes[i].rev: nodes[i].r else: nodes[i].l)
                nodes[i].val = apply_fn(nodes[i].val, ls, f)
                if let old = nodes[i].tag:
                    nodes[i].tag = compose_fn(f, old)
                else:
                    nodes[i].tag = f

    fn push_tag(mut self, i: Int):
        unchecked:
            if let f = nodes[i].tag:
                let (a, b) = if nodes[i].rev: (nodes[i].r, nodes[i].l) else: (
                    nodes[i].l,
                    nodes[i].r,
                )
                let (fa, fb) = split_fn(f, self.sz_of(a))
                self.put(a, fa)
                self.put(b, fb)
                nodes[i].tag = None

    fn toggle(mut self, u: ?Int):
        if let i = u:
            self.push_tag(i)
            unchecked: nodes[i].rev = not nodes[i].rev

    fn down(mut self, i: Int):
        self.push_tag(i)
        unchecked:
            if nodes[i].rev:
                let l = nodes[i].l
                nodes[i].l = nodes[i].r
                nodes[i].r = l
                nodes[i].rev = false
                self.toggle(nodes[i].l)
                self.toggle(nodes[i].r)

    fn pull(mut self, i: Int):
        unchecked: nodes[i].sz = 1 + self.sz_of(nodes[i].l) + self.sz_of(nodes[i].r)

    fn split_val(mut self, u: ?Int, x: T) -> (?Int, ?Int):
        let i = u ?? return (None, None)
        unchecked:
            self.down(i)
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

    fn ins(mut self, u: ?Int, idx: Int, x: T, p: Int) -> Int:
        let i = u ?? return idx
        unchecked:
            if p < nodes[i].pri:
                let (a, b) = self.split_val(i, x)
                nodes[idx].l = a
                nodes[idx].r = b
                self.pull(idx)
                return idx
            self.down(i)
            nodes[i].sz += 1
            if before_fn(x, nodes[i].val):
                nodes[i].l = self.ins(nodes[i].l, idx, x, p)
            else:
                nodes[i].r = self.ins(nodes[i].r, idx, x, p)
            return i

    fn split_pos(mut self, u: ?Int, k: Int) -> (?Int, ?Int):
        let i = u ?? return (None, None)
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

    /// Insert `x` before the first element `e` with `before(x, e)`, or at the end when there is
    /// None. With a strict order for `before`, a sorted treap stays sorted and `x` goes after the
    /// elements equal to it.
    ///
    /// cost: \(O(\log n)\) expected
    /// ties: after every equal element
    /// traps: when the treap already holds `2^31 - 1` elements
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f, before: (x, y) => x < y); t.insert(3); t.insert(1); t.insert(2); t.to_list() == [1, 2, 3]`
    pub fn insert(mut self, x: T):
        let idx = nodes.len
        let p = self.priority()
        nodes.push(TreapNode(None, None, p, 1, x, None, false))
        root = self.ins(root, idx, x, p)

    /// Append `x` at the end of the sequence.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: when the treap already holds `2^31 - 1` elements
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(2); t.to_list() == [1, 2]`
    pub fn push(mut self, x: T):
        let idx = nodes.len
        nodes.push(TreapNode(None, None, self.priority(), 1, x, None, false))
        root = self.merge(root, idx)

    /// Insert `x` so that it becomes the element at 0-based position `pos`; `pos == len` appends.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: when `pos < 0` or `pos > len`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(3); t.insert_at(1, 2); t.to_list() == [1, 2, 3]`
    pub fn insert_at(mut self, pos: Int, x: T):
        assert 0 <= pos <= self.len
        let idx = nodes.len
        nodes.push(TreapNode(None, None, self.priority(), 1, x, None, false))
        let (a, b) = self.split_pos(root, pos)
        root = self.merge(self.merge(a, idx), b)

    /// Apply tag `f` to every element, as one tag on the root; ranks `k` count from the first element.
    ///
    /// cost: \(O(1)\)
    /// empty: changes nothing
    /// traps: never
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(2); t.apply_all(10); t.to_list() == [11, 12]`
    pub fn apply_all(mut self, f: F):
        self.put(root, f)

    /// Apply tag `f` to the elements at positions `lo..hi`; ranks `k` count from position `lo`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: `lo == hi` changes nothing
    /// traps: when `lo < 0`, `hi > len` or `lo > hi`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(0); t.push(0); t.push(0); t.apply_range(1, 3, 5); t.to_list() == [0, 5, 5]`
    pub fn apply_range(mut self, lo: Int, hi: Int, f: F):
        assert 0 <= lo <= hi <= self.len
        if lo == hi: return
        let (a, bc) = self.split_pos(root, lo)
        let (b, c) = self.split_pos(bc, hi - lo)
        self.put(b, f)
        root = self.merge(self.merge(a, b), c)

    /// Apply tag `f` to the elements at the positions of `r`; `apply_range` with a range.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `0..len`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(2); t.apply(1..2, 10); t.to_list() == [1, 12]`
    pub fn apply(mut self, r: Range<Int>, f: F):
        self.apply_range(r.start, r.start + r.len, f)

    /// Reverse the order of the elements at positions `r`. Tags applied earlier keep their effect
    /// on the elements they reached; a later tag sees the new order.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `0..len`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(2); t.push(3); t.reverse(0..2); t.to_list() == [2, 1, 3]`
    pub fn reverse(mut self, r: Range<Int>):
        let (lo, hi) = (r.start, r.start + r.len)
        assert 0 <= lo <= hi <= self.len, "`reverse` out of range"
        if lo == hi: return
        let (a, bc) = self.split_pos(root, lo)
        let (b, c) = self.split_pos(bc, hi - lo)
        self.toggle(b)
        root = self.merge(self.merge(a, b), c)

    /// Remove the element at 0-based position `pos` and return it with every pending tag applied.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: when `pos < 0` or `pos >= len`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(2); t.apply_all(5); t.erase_at(0) == 6 and t.to_list() == [7]`
    pub fn erase_at(mut self, pos: Int) -> T:
        assert 0 <= pos < self.len, "`erase_at` position out of range"
        let (a, bc) = self.split_pos(root, pos)
        let (b, c) = self.split_pos(bc, 1)
        root = self.merge(a, c)
        unchecked: return nodes[b!].val

    /// Remove the elements at positions `r`; later elements move down by `r.len`.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `0..len`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(2); t.push(3); t.erase_range(0..2); t.to_list() == [3]`
    pub fn erase_range(mut self, r: Range<Int>):
        let (lo, hi) = (r.start, r.start + r.len)
        assert 0 <= lo <= hi <= self.len, "`erase_range` out of range"
        if lo == hi: return
        let (a, bc) = self.split_pos(root, lo)
        let (_, c) = self.split_pos(bc, hi - lo)
        root = self.merge(a, c)

    /// Split off the elements at positions `k..len` into a new treap with the same `apply`,
    /// `compose`, `split` and `before`, and return it; `self` keeps positions `0..k`. Pending
    /// tags travel with the elements they cover.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: when `k < 0` or `k > len`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(2); t.push(3); let u = t.split_at(1); t.to_list() == [1] and u.to_list() == [2, 3]`
    pub fn split_at(mut self, k: Int) -> LazyTreap<T, F>:
        assert 0 <= k <= self.len, "`split_at` position out of range"
        let (a, b) = self.split_pos(root, k)
        root = a
        var out = LazyTreap(apply_fn, split_fn, compose_fn, before_fn, [], None, Rng(48))
        out.root = out.copy_from(self, b)
        out

    fn copy_from(mut self, src: LazyTreap<T, F>, u: ?Int) -> ?Int:
        let i = u ?? return None
        let n = src.nodes[i]
        let l = self.copy_from(src, n.l)
        let r = self.copy_from(src, n.r)
        nodes.push(TreapNode(l, r, n.pri, n.sz, n.val, n.tag, n.rev))
        nodes.len - 1

    /// Move every element of `other` onto the end of `self`, in order, leaving `other` empty. Both
    /// treaps must use the same functions; `self`'s are kept.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: an empty `other` changes nothing
    /// traps: when the total reaches `2^31` elements
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); var u = t.split_at(1); u.push(5); t.append(mut u); t.to_list() == [1, 5] and u.is_empty`
    pub fn append(mut self, mut other: LazyTreap<T, F>):
        assert self.len + other.len < 2_147_483_648, "a LazyTreap holds fewer than `2^31` elements"
        let moved = self.copy_from(other, other.root)
        root = self.merge(root, moved)
        other.root = None
        other.nodes = []

    /// The element at 0-based position `pos` with every pending tag on its path applied. Tags are
    /// pushed down along the path, which is why it takes `mut self`; the sequence does not change.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: when `pos < 0` or `pos >= len`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(2); t.apply(0..1, 4); t.get(0) == 5`
    pub fn get(mut self, pos: Int) -> T:
        assert 0 <= pos < self.len
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

    // A read leaves the tree as it is: what `down` would push into a node rides along instead,
    // as the tag `pend` put on it and whether it is toggled (`tog`) after that.
    fn walk(self, u: ?Int, pend: ?F, tog: Bool, lim: Int, mut out: List<T>):
        if out.len >= lim: return
        let i = u ?? return
        unchecked:
            let n = nodes[i]
            let (a, b) = if n.rev: (n.r, n.l) else: (n.l, n.r)
            let ls = self.sz_of(a)
            var v = n.val
            var t = n.tag
            if let p = pend:
                v = apply_fn(v, ls, p)
                if let old = t: t = compose_fn(p, old) else: t = p
            var (fa, fb): (?F, ?F) = (None, None)
            if let f = t:
                let (x, y) = split_fn(f, ls)
                fa = x
                fb = y
            let flip = n.rev != tog
            let (fl, fr) = if n.rev: (fb, fa) else: (fa, fb)
            let (first, f1, last, f2) = if flip: (n.r, fr, n.l, fl) else: (n.l, fl, n.r, fr)
            self.walk(first, f1, flip, lim, mut out)
            if out.len < lim:
                out.push(v)
                self.walk(last, f2, flip, lim, mut out)

    /// The first `min(k, len)` elements in order, with pending tags applied. Reads without pushing
    /// tags, so the tree is unchanged.
    ///
    /// cost: \(O(k + \log n)\)
    /// empty: `[]` when `k == 0` or the treap is empty
    /// order: sequence order
    /// traps: when `k < 0`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(2); t.push(3); t.take(2) == [1, 2]`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.take(5) == [1]`
    pub fn take(self, k: Int) -> List<T>:
        var out: List<T> = []
        if k > 0: self.walk(root, None, false, k, mut out)
        out

    /// Every element in order, with pending tags applied; `take(len)`.
    ///
    /// cost: \(O(n)\)
    /// empty: `[]`
    /// order: sequence order
    /// traps: never
    /// example: `LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f).to_list() == []`
    pub fn to_list(self) -> List<T> = self.take(self.len)

/// \(O(n)\) space. Implicit-key rope over a domain `D` supporting \(O(\log n)\) expected cut, paste,
/// reverse, range tag `apply`, path queries, and point `get`.
pub struct Rope<D, T>:
    var n: Int
    mg: ?((T, T) -> T) = None
    var base: ?[D]T = None
    val: List<T> = []
    tg: List<T> = []
    has: List<Bool> = []
    rv: List<Bool> = []
    pri: List<Int> = []
    ls: List<Int> = []
    rs: List<Int> = []
    sz: List<Int> = []
    var root: Int = -1
    var seed: Int = 1

    fn size(self, t: Int) -> Int = if t < 0: 0 else: sz[t]

    fn pull(mut self, t: Int):
        sz[t] = self.size(ls[t]) + 1 + self.size(rs[t])

    // `val[t]` leaves out `tg[t]`, which reaches it only when `down` pushes the tag.
    fn tag_node(mut self, t: Int, x: T):
        if t < 0: return
        if has[t]:
            if let m = mg: tg[t] = m(x, tg[t])
            else: tg[t] = x
        else:
            tg[t] = x
        has[t] = true

    fn down(mut self, t: Int):
        if rv[t]:
            let c = ls[t]
            ls[t] = rs[t]
            rs[t] = c
            if ls[t] >= 0: rv[ls[t]] = not rv[ls[t]]
            if rs[t] >= 0: rv[rs[t]] = not rv[rs[t]]
            rv[t] = false
        if has[t]:
            let x = tg[t]
            self.tag_node(ls[t], x)
            self.tag_node(rs[t], x)
            if let m = mg: val[t] = m(x, val[t])
            else: val[t] = x
            has[t] = false

    // (first k values, the rest)
    fn split(mut self, t: Int, k: Int) -> (Int, Int):
        if t < 0: return (-1, -1)
        self.down(t)
        let s = self.size(ls[t])
        if s < k:
            let (a, b) = self.split(rs[t], k - s - 1)
            rs[t] = a
            self.pull(t)
            (t, b)
        else:
            let (a, b) = self.split(ls[t], k)
            ls[t] = b
            self.pull(t)
            (a, t)

    fn join(mut self, a: Int, b: Int) -> Int:
        if a < 0: return b
        if b < 0: return a
        if pri[a] > pri[b]:
            self.down(a)
            rs[a] = self.join(rs[a], b)
            self.pull(a)
            a
        else:
            self.down(b)
            ls[b] = self.join(a, ls[b])
            self.pull(b)
            b

    fn find(mut self, k0: Int) -> Int:
        var (t, k) = (root, k0)
        while true:
            self.down(t)
            let s = self.size(ls[t])
            if k < s: t = ls[t]
            elif k == s: return t
            else:
                k -= s + 1
                t = rs[t]
        -1

    fn emit(mut self, t: Int, mut out: [D]T, mut k: Int):
        if t < 0: return
        self.down(t)
        self.emit(ls[t], mut out, mut k)
        out[D.nth(k)] = val[t]
        k += 1
        self.emit(rs[t], mut out, mut k)

    /// A rope over `D` holding the values of `a`, or `fill` at every position. `merge(tag, x)` is
    /// how a tag from `apply` combines into a value; without it, a tag overwrites the values it covers.
    /// Positions keep their domain `D`; `reverse` moves values, not positions.
    ///
    /// cost: \(O(\lvert D \rvert)\)
    /// empty: an empty `D` builds an empty rope
    /// order: positions keep the domain `D` (of `a`, when given)
    /// traps: when neither or both of `a` and `fill` are given
    /// example: `var t = Rope([1, 2, 3]); t.to_array() == [1, 2, 3]`
    /// example: `var t = Rope<0..2, Int>(fill: 7); t.get(1) == 7`
    pub fn new(a: ?[D]T = None, fill: ?T = None, merge: ?((T, T) -> T) = None) -> Rope<D, T>:
        var r = Rope<D, T>(n: 0, mg: merge)
        if let x = a:
            assert fill == None
            r.base = Some(x)
        else:
            let arr: [D]T = fill!
            r.base = Some(arr)
        let src = r.base!
        var stack: List<Int> = []
        for i in D:
            let t = r.val.len
            r.seed = (r.seed * 1103515245 + 12345) % 2147483648
            r.val.push(src[i])
            r.tg.push(src[i])
            r.has.push(false)
            r.rv.push(false)
            r.pri.push(r.seed)
            r.ls.push(-1)
            r.rs.push(-1)
            r.sz.push(1)
            var last = -1
            while stack.len > 0 and r.pri[stack.last!] < r.seed:
                last = stack.pop()!
                r.pull(last)
            r.ls[t] = last
            if stack.len > 0: r.rs[stack.last!] = t
            stack.push(t)
        while let t = stack.pop():
            r.pull(t)
            r.root = t
        r.n = r.val.len
        r

    /// Apply `tag` to every value at positions `r`: each becomes `merge(tag, x)`, or `tag` when
    /// no `merge` was given. Stored lazily on the covering nodes.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var r = Rope([1, 2, 3], merge: (t, x) => t + x); r.apply(1..3, 10); r.to_array() == [1, 12, 13]`
    /// example: `var r = Rope([1, 2, 3]); r.apply(0..2, 7); r.to_array() == [7, 7, 3]`
    pub fn apply(mut self, r: Range<D>, tag: T):
        if r.is_empty: return
        let (lo, hi) = (r.first!.ord, r.last!.ord + 1)
        let (a, bc) = self.split(root, lo)
        let (b, c) = self.split(bc, hi - lo)
        self.tag_node(b, tag)
        root = self.join(self.join(a, b), c)

    /// Reverse the order of the values at positions `r`; pending tags stay with their values.
    ///
    /// cost: \(O(\log n)\) expected
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var r = Rope([1, 2, 3]); r.reverse(0..3); r.get(0) == 3`
    pub fn reverse(mut self, r: Range<D>):
        if r.is_empty: return
        let (lo, hi) = (r.first!.ord, r.last!.ord + 1)
        let (a, bc) = self.split(root, lo)
        let (b, c) = self.split(bc, hi - lo)
        rv[b] = not rv[b]
        root = self.join(self.join(a, b), c)

    /// The value currently at position `i`, with pending tags merged in.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: when `i` is outside `D`
    /// example: `var r = Rope([1, 2, 3]); r.reverse(1..3); r.get(1) == 3`
    pub fn get(mut self, i: D) -> T:
        let t = self.find(i.ord)
        val[t]

    // The first `Some` of `f` on node `t`'s path to position `k`: the value first, then each
    // ancestor's pending tag, deepest first. `flip` is the reversal pending from above `t`.
    fn path_first<R>(mut self, t: Int, k: Int, flip: Bool, f: (mut T) -> ?R) -> ?R:
        let sw = flip != rv[t]
        let (l, r) = if sw: (rs[t], ls[t]) else: (ls[t], rs[t])
        let s = self.size(l)
        if k == s:
            if let x = f(mut val[t]): return Some(x)
            return if has[t]: f(mut tg[t]) else: None
        let below = if k < s: self.path_first(l, k, sw, f) else: self.path_first(r, k - s - 1, sw, f)
        if let x = below: return Some(x)
        if has[t]: f(mut tg[t]) else: None

    /// The first `Some` that `f` gives on the pieces holding position `i`'s value, oldest first:
    /// the value stored at `i`, then each tag still pending above it, without pushing any tag down.
    /// `f` may change a piece in place (`(mut seg) => seg.first_alive(ok)` prunes dead entries for
    /// good), so a query that would otherwise merge tags costs only the walk.
    ///
    /// cost: \(O(\log n)\) expected calls of `f`
    /// empty: `None` when `f` gives `None` on every piece
    /// order: the value at `i` first, then pending tags from the deepest up to the root
    /// traps: when `i` is outside `D`
    /// example: `var r = Rope([1, 2], merge: (t, x) => t + x); r.apply(0..2, 10); r.query_path(1, (mut x) => if x > 5: Some(x) else: None) == Some(10)`
    pub fn query_path<R>(mut self, i: D, f: (mut T) -> ?R) -> ?R = self.path_first(root, i.ord, false, f)

    /// Overwrite the value currently at position `i` with `x`, replacing any pending tags on it.
    ///
    /// cost: \(O(\log n)\) expected
    /// traps: when `i` is outside `D`
    /// example: `var r = Rope([1, 2, 3]); r.set(1, 9); r.to_array() == [1, 9, 3]`
    pub fn set(mut self, i: D, x: T):
        let t = self.find(i.ord)
        val[t] = x

    /// The current values in position order, with every pending tag merged in.
    ///
    /// cost: \(O(\lvert D \rvert)\)
    /// empty: an empty array for an empty `D`
    /// order: position order of `D`
    /// traps: never
    /// example: `var r = Rope([1, 2, 3]); r.reverse(0..2); r.to_array() == [2, 1, 3]`
    pub fn to_array(mut self) -> [D]T:
        var out = base!
        var k = 0
        self.emit(root, mut out, mut k)
        out

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
    var st = SegTree(Flip<Int>.op(+), init, act: flip_action<Int>(+))
    assert st.all.now == 3
    assert st.all.flipped == 3
    st.apply(I(1)..I(5), true)
    assert st.all.now == 3
    assert st.fold(I(1)..I(5)).now == 2
    st.apply(I(0)..I(3), true)
    assert st.all.now == 2
