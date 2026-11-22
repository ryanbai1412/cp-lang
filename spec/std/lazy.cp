//! Range-update helpers that are not segment trees: `Flip`, `LazyTreap`, `Rope`. Range apply on a segment tree is `SegTree(op, a, act:)` (`segtree.cp`).
use std.algebra.{Action, MaxCount, MinCount, Monoid}
use std.rand.Rng
use std.seq.{ends}


/// Paired aggregate `(now, flipped)` for range-toggle / complement updates (`SegTree(op, a, act: flip)`).
/// Build the toggle action with `flip_action(m)` (tag `Bool`: `true` swaps `now` and `flipped`).
pub struct Flip<T>(now: T, flipped: T):
    /// The leaf for one element that is currently `on` or off: `Flip(x, id)` when `on`, else
    /// `Flip(id, x)`. `now` is the value the element contributes as it stands and `flipped` what it
    /// would contribute after one toggle.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Flip.of(true, 5, 0).now == 5`
    /// example: `Flip.of(false, 5, 0).flipped == 5`
    pub fn of(on: Bool, x: T, id: T) -> Flip<T>:
        if on: Flip(x, id) else: Flip(id, x)

    /// The monoid on pairs that folds both halves with `m` independently:
    /// `op(a, b) == Flip(m(a.now, b.now), m(a.flipped, b.flipped))`, identity `Flip(id, id)` with
    /// `id` from `Monoid.of(m)`.
    ///
    /// cost: O(1)
    /// traps: never (an `m` without a known identity is a compile-time error)
    /// example: `Flip.op(+).op(Flip(1, 0), Flip(0, 1)).now == 1`
    /// example: `Flip.op(+).id.flipped == 0`
    pub fn op<M>(m: M) -> Monoid<Flip<T>> = todo

/// The range-toggle action on `Flip<T>` values over `m`: the tag is a `Bool`, `true` swaps
/// `now` and `flipped`, `false` is the identity, and tags compose by exclusive or. The swap
/// needs no span, since both halves were folded with the same `m`.
///
/// cost: O(1)
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
    if pick == 1: return min(a[v] for v in r) ?? Int.MAX
    max(a[v] for v in r) ?? Int.MIN

@test
fn the_stock_actions_match_brute_force() = todo

@test
fn at_writes_through_pending_tags() = todo

@test
fn stock_actions_obey_their_laws() = todo

@test
fn set_and_an_empty_tree() = todo

@test
fn padding_never_takes_an_update() = todo

@test
fn an_action_of_your_own() = todo

@test
fn right_while_and_left_while_see_pending_updates() = todo

fn ccc_compose(f: (?Int, ?Int, Int), g: (?Int, ?Int, Int)) -> (?Int, ?Int, Int):
    let nb = f.0 ?? g.0
    if f.1 != None: return (nb, f.1, 0)
    if g.1 != None: return (nb, g.1! + f.2, 0)
    (nb, None, g.2 + f.2)

fn ccc_apply(f: (?Int, ?Int, Int), x: (Int, Int, Int), len: Int) -> (Int, Int, Int):
    let b_min = f.0 ?? x.1
    if f.1 != None:
        let a_val = b_min + f.1!
        return (a_val, b_min, a_val * len)
    (x.0 + f.2, b_min, x.2 + f.2 * len)

@test
fn ccc02s4hard_dual_field_cross_overwrite_and_interleaved_point_set() = todo

@test
fn update_pushes_modifies_and_pulls_in_one_pass() = todo

@test
fn custom_clamp_action_with_update() = todo

@test
fn fill_and_affine_sum_match_brute() = todo

@test
fn dual_segtree_non_commutative_tags_match_brute() = todo

struct TreapNode<T, F>(
    l: ?Int,
    r: ?Int,
    pri: 0..2_147_483_648,
    sz: 0..2_147_483_648,
    val: T,
    tag: ?F,
)

/// Arena-backed randomized treap with subtree-size-aware lazy tag propagation.
/// Supports value-ordered insertion (`insert`, using `before`) for slope-trick / priority
/// structures with lazy updates, and 0-based implicit sequence operations (`push`, `insert_at`,
/// `apply_range`, `get`).
/// `apply(v, left_sz, f)` updates a node's value given its left subtree size;
/// `split(f, left_sz)` splits a tag `f` for the left and right subtrees (default `(f, f)`);
/// `compose(new_f, old_f)` applies `old_f` first then `new_f`.
/// `insert`, `insert_at`, `push`, `apply_range`, `get` are O(log n) expected; `apply_all` is O(1);
/// `take(k)` is O(k + log n); `to_list` is O(n). Memory is O(n) in a contiguous node arena;
/// sizes and priorities are `0..2^31`, so a treap holds fewer than 2^31 elements.
pub struct LazyTreap<T, F>(
    apply_fn: (T, Int, F) -> T,
    split_fn: (F, Int) -> (F, F),
    compose_fn: (F, F) -> F,
    before_fn: (T, T) -> Bool,
):
    nodes: List<TreapNode<T, F>>
    root: ?Int
    rng: Rng

    /// An empty treap. `apply(v, k, f)` is element `v` after tag `f`, where `k` is the number of
    /// elements before it in the run the tag was applied to (so tags may depend on rank);
    /// `compose(new_f, old_f)` is `old_f` then `new_f`; `split(f, k)` gives the tags for the left
    /// child and for the right child of a node with `k` elements on its left (default `(f, f)`, for
    /// tags that ignore rank; a rank-dependent tag needs a `split` that shifts the right child's ranks
    /// by `k + 1`); `before(x, y)` orders value insertion (default never, so `insert` appends).
    ///
    /// cost: O(1)
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
    /// cost: O(1)
    /// traps: never
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(4); t.len == 1`
    @get pub fn len(self) -> Int = self.sz_of(root)

    /// Make room for `n` elements in total, so pushes up to that size never regrow the node arena.
    /// A smaller `n` than the current capacity changes nothing.
    ///
    /// cost: O(n) when it grows, else O(1)
    /// traps: when `n >= 2^31` (the size bound)
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.reserve(10); t.len == 0`
    pub fn reserve(mut self, n: Int) = root == None

    /// Whether the treap holds no elements.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f).is_empty == true`
    @get pub fn is_empty(self) -> Bool = root == None

    fn put(mut self, u: ?Int, f: F):
        if let i = u:
            unchecked:
                nodes[i].val = apply_fn(nodes[i].val, self.sz_of(nodes[i].l), f)
                if let old = nodes[i].tag:
                    nodes[i].tag = compose_fn(f, old)
                else:
                    nodes[i].tag = f

    fn down(mut self, i: Int):
        unchecked:
            if let f = nodes[i].tag:
                let (fl, fr) = split_fn(f, self.sz_of(nodes[i].l))
                self.put(nodes[i].l, fl)
                self.put(nodes[i].r, fr)
                nodes[i].tag = None

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
    /// cost: O(log n) expected
    /// ties: after every equal element
    /// traps: when the treap already holds `2^31 - 1` elements
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f, before: (x, y) => x < y); t.insert(3); t.insert(1); t.insert(2); t.to_list() == [1, 2, 3]`
    pub fn insert(mut self, x: T):
        let idx = nodes.len
        let p = self.priority()
        nodes.push(TreapNode(None, None, p, 1, x, None))
        root = self.ins(root, idx, x, p)

    /// Append `x` at the end of the sequence.
    ///
    /// cost: O(log n) expected
    /// traps: when the treap already holds `2^31 - 1` elements
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(2); t.to_list() == [1, 2]`
    pub fn push(mut self, x: T):
        let idx = nodes.len
        nodes.push(TreapNode(None, None, self.priority(), 1, x, None))
        root = self.merge(root, idx)

    /// Insert `x` so that it becomes the element at 0-based position `pos`; `pos == len` appends.
    ///
    /// cost: O(log n) expected
    /// traps: when `pos < 0` or `pos > len`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(3); t.insert_at(1, 2); t.to_list() == [1, 2, 3]`
    pub fn insert_at(mut self, pos: Int, x: T):
        assert 0 <= pos and pos <= self.len
        let idx = nodes.len
        nodes.push(TreapNode(None, None, self.priority(), 1, x, None))
        let (a, b) = self.split_pos(root, pos)
        root = self.merge(self.merge(a, idx), b)

    /// Apply tag `f` to every element, as one tag on the root; ranks `k` count from the first element.
    ///
    /// cost: O(1)
    /// empty: changes nothing
    /// traps: never
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(2); t.apply_all(10); t.to_list() == [11, 12]`
    pub fn apply_all(mut self, f: F):
        self.put(root, f)

    /// Apply tag `f` to the elements at positions `lo..hi`; ranks `k` count from position `lo`.
    ///
    /// cost: O(log n) expected
    /// empty: `lo == hi` changes nothing
    /// traps: when `lo < 0`, `hi > len` or `lo > hi`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(0); t.push(0); t.push(0); t.apply_range(1, 3, 5); t.to_list() == [0, 5, 5]`
    pub fn apply_range(mut self, lo: Int, hi: Int, f: F):
        assert 0 <= lo and lo <= hi and hi <= self.len
        if lo == hi: return
        let (a, bc) = self.split_pos(root, lo)
        let (b, c) = self.split_pos(bc, hi - lo)
        self.put(b, f)
        root = self.merge(self.merge(a, b), c)

    /// Apply tag `f` to the elements at the positions of `r`; `apply_range` with a range.
    ///
    /// cost: O(log n) expected
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `0..len`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(2); t.apply(1..2, 10); t.to_list() == [1, 12]`
    pub fn apply(mut self, r: Range<Int>, f: F):
        self.apply_range(r.start, r.start + r.len, f)

    /// Reverse the order of the elements at positions `r`. Tags applied earlier keep their effect
    /// on the elements they reached; a later tag sees the new order.
    ///
    /// cost: O(log n) expected
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `0..len`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(2); t.push(3); t.reverse(0..2); t.to_list() == [2, 1, 3]`
    pub fn reverse(mut self, r: Range<Int>) = todo

    /// Remove the element at 0-based position `pos` and return it with every pending tag applied.
    ///
    /// cost: O(log n) expected
    /// traps: when `pos < 0` or `pos >= len`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(2); t.apply_all(5); t.erase_at(0) == 6 and t.to_list() == [7]`
    pub fn erase_at(mut self, pos: Int) -> T:
        assert 0 <= pos and pos < self.len, "`erase_at` position out of range"
        let (a, bc) = self.split_pos(root, pos)
        let (b, c) = self.split_pos(bc, 1)
        root = self.merge(a, c)
        unchecked: return nodes[b!].val

    /// Remove the elements at positions `r`; later elements move down by `r.len`.
    ///
    /// cost: O(log n) expected
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `0..len`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(2); t.push(3); t.erase_range(0..2); t.to_list() == [3]`
    pub fn erase_range(mut self, r: Range<Int>):
        let (lo, hi) = (r.start, r.start + r.len)
        assert 0 <= lo and lo <= hi and hi <= self.len, "`erase_range` out of range"
        if lo == hi: return
        let (a, bc) = self.split_pos(root, lo)
        let (_, c) = self.split_pos(bc, hi - lo)
        root = self.merge(a, c)

    /// Split off the elements at positions `k..len` into a new treap with the same `apply`,
    /// `compose`, `split` and `before`, and return it; `self` keeps positions `0..k`. Pending
    /// tags travel with the elements they cover.
    ///
    /// cost: O(log n) expected
    /// traps: when `k < 0` or `k > len`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(2); t.push(3); let u = t.split_at(1); t.to_list() == [1] and u.to_list() == [2, 3]`
    pub fn split_at(mut self, k: Int) -> LazyTreap<T, F> = todo

    /// Move every element of `other` onto the end of `self`, in order, leaving `other` empty. Both
    /// treaps must use the same functions; `self`'s are kept.
    ///
    /// cost: O(log n) expected
    /// empty: an empty `other` changes nothing
    /// traps: when the total reaches `2^31` elements
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); var u = t.split_at(1); u.push(5); t.append(mut u); t.to_list() == [1, 5] and u.is_empty`
    pub fn append(mut self, mut other: LazyTreap<T, F>) = todo

    /// The element at 0-based position `pos` with every pending tag on its path applied. Tags are
    /// pushed down along the path, which is why it takes `mut self`; the sequence does not change.
    ///
    /// cost: O(log n) expected
    /// traps: when `pos < 0` or `pos >= len`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(2); t.apply(0..1, 4); t.get(0) == 5`
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

    // A read leaves the tree as it is: the tags still pending above a node
    // ride along as `pend` instead of being pushed into it.
    fn walk(self, u: ?Int, pend: ?F, lim: Int, mut out: List<T>):
        if out.len >= lim: return
        let i = u ?? return
        unchecked:
            let ls = self.sz_of(nodes[i].l)
            var down = nodes[i].tag
            var v = nodes[i].val
            if let p = pend:
                v = apply_fn(v, ls, p)
                if let t = down: down = compose_fn(p, t) else: down = p
            if let d = down:
                let (fl, fr) = split_fn(d, ls)
                self.walk(nodes[i].l, fl, lim, mut out)
                if out.len < lim:
                    out.push(v)
                    self.walk(nodes[i].r, fr, lim, mut out)
            else:
                self.walk(nodes[i].l, None, lim, mut out)
                if out.len < lim:
                    out.push(v)
                    self.walk(nodes[i].r, None, lim, mut out)

    /// The first `min(k, len)` elements in order, with pending tags applied. Reads without pushing
    /// tags, so the tree is unchanged.
    ///
    /// cost: O(k + log n)
    /// empty: `[]` when `k == 0` or the treap is empty
    /// order: sequence order
    /// traps: when `k < 0`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(2); t.push(3); t.take(2) == [1, 2]`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.take(5) == [1]`
    pub fn take(self, k: Int) -> List<T>:
        var out: List<T> = []
        if k > 0: self.walk(root, None, k, mut out)
        out

    /// Every element in order, with pending tags applied; `take(len)`.
    ///
    /// cost: O(n)
    /// empty: `[]`
    /// order: sequence order
    /// traps: never
    /// example: `LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f).to_list() == []`
    pub fn to_list(self) -> List<T> = self.take(self.len)

/// O(n) space. Implicit-key rope over a domain `D` supporting O(log n) expected cut, paste,
/// reverse, range tag `apply`, path queries, and point `get`.
pub struct Rope<D, T>:
    n: Int
    mg: ?((T, T) -> T) = None
    base: ?[D]T = None
    val: List<T> = []
    tg: List<T> = []
    has: List<Bool> = []
    rv: List<Bool> = []
    pri: List<Int> = []
    ls: List<Int> = []
    rs: List<Int> = []
    sz: List<Int> = []
    root: Int = -1
    seed: Int = 1

    fn size(self, t: Int) -> Int = if t < 0: 0 else: sz[t]

    fn pull(mut self, t: Int):
        sz[t] = self.size(ls[t]) + 1 + self.size(rs[t])

    fn tag_node(mut self, t: Int, x: T):
        if t < 0: return
        if let m = mg:
            val[t] = m(x, val[t])
            tg[t] = if has[t]: m(x, tg[t]) else: x
        else:
            val[t] = x
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
            self.tag_node(ls[t], tg[t])
            self.tag_node(rs[t], tg[t])
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
    /// cost: O(|D|)
    /// empty: an empty `D` builds an empty rope
    /// order: positions keep the domain `D` (of `a`, when given)
    /// traps: when neither or both of `a` and `fill` are given
    /// example: `Rope([1, 2, 3]).to_array() == [1, 2, 3]`
    /// example: `Rope<0..2, Int>(fill: 7).get(1) == 7`
    pub fn new(a: ?[D]T = None, fill: ?T = None, merge: ?((T, T) -> T) = None) -> Rope<D, T>:
        var r = Rope<D, T>(n: 0, mg: merge)
        if let x = a:
            assert fill == None
            r.base = Some(x)
        else:
            var arr: [D]T = fill!
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
            while stack.len > 0 and r.pri[stack[stack.len - 1]] < r.seed:
                last = stack.pop()!
                r.pull(last)
            r.ls[t] = last
            if stack.len > 0: r.rs[stack[stack.len - 1]] = t
            stack.push(t)
        while stack.len > 0:
            let t = stack.pop()!
            r.pull(t)
            r.root = t
        r.n = r.val.len
        r

    /// Apply `tag` to every value at positions `r`: each becomes `merge(tag, x)`, or `tag` when
    /// no `merge` was given. Stored lazily on the covering nodes.
    ///
    /// cost: O(log n) expected
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
    /// cost: O(log n) expected
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
    /// cost: O(log n) expected
    /// traps: when `i` is outside `D`
    /// example: `var r = Rope([1, 2, 3]); r.reverse(1..3); r.get(1) == 3`
    pub fn get(mut self, i: D) -> T = val[self.find(i.ord)]

    /// Overwrite the value currently at position `i` with `x`, replacing any pending tags on it.
    ///
    /// cost: O(log n) expected
    /// traps: when `i` is outside `D`
    /// example: `var r = Rope([1, 2, 3]); r.set(1, 9); r.to_array() == [1, 9, 3]`
    pub fn set(mut self, i: D, x: T):
        let t = self.find(i.ord)
        val[t] = x

    /// The current values in position order, with every pending tag merged in.
    ///
    /// cost: O(|D|)
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
fn flip_action_toggles_ranges() = todo
