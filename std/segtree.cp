//! Segment tree over a domain, for any associative operation: point or range update, range query.
//!
//! start: `SegTree`, `MergeSortTree`, `WaveletMatrix`, `SegTree2D`
//! example: `SegTree(+, [1, 2, 3]).all == 6`
use std.algebra.{Action, Acts, Monoid}
use std.bag.{SortedBag}
use std.math.{gcd, bit, popcount, bit_ceil, bit_width, ctz}
use std.range.{Len, ends, first_true}
use std.domains.{Cut, Domain}
use std.list.{List, Str}
use std.ops.{Add, Mul, One, Ord, Zero, max, min}
use std.optional.{Some}
use std.ranges.{Range}
use std.show.{Show}
use std.stream.{rand}

type Pos = 0..=2_000_000_000

struct NoAction

impl<T> Acts<T, Int> for NoAction:
    fn action(self, over: Monoid<T>, sums: Bool) -> Action<T, Int>:
        assert false, "a tree without `act:` has no action"
        Action(0, (f, g) => f, (f: Int, v: T, span: Int) => v, over)
struct NoOp
struct NoOver
fn same<A, B>(a: A, b: B) -> Bool = a == b

/// The positions a `SegTree` runs over. A domain brings its own; `Int` has none, so a
/// `SegTree<Int, T>` takes them from `over: lo..hi`.
pub trait Positions:
    fn count() -> Int
    fn at(k: Int) -> Self
    fn cut(k: Int) -> Cut<Self>

impl<D: Domain> Positions for D:
    fn count() -> Int = D.count
    fn at(k: Int) -> D = D.nth(k)
    fn cut(k: Int) -> Cut<D> = D.cuts.nth(k)

impl Positions for Int:
    fn count() -> Int = 0
    fn at(k: Int) -> Int = k
    fn cut(k: Int) -> Int = k

/// Point update and range fold. The operation is a bare operator (`+`, `*`, `^`, `&`, `|`,
/// `min`, `max`, `gcd`) or your own `Monoid(id, op)`: `SegTree<V, Int>(max, a)`,
/// `SegTree<V, Int>(+)`, `SegTree<V, Run>(Monoid(Run.empty, Run.join), a)`.
/// `set` and `fold` are \(O(\log n)\), and so are `right_while` and `left_while`; memory is
/// 2 * bit_ceil(n) values. The fold keeps the order of the positions, so
/// non-commutative operations work. `SegTree<Int, T>(op, over: lo..hi)` covers a huge range
/// instead, making nodes on first touch: every call is \(O(\log(hi - lo))\) and makes at most
/// \(2 \log_2(hi - lo)\) nodes, so memory grows with the positions touched, not with `hi - lo`.
pub struct SegTree<D: Positions, T, F = Int>(m: Monoid<T>):
    var t: List<T>
    var lz: List<F>
    var has_lz: List<Bool>
    size: Len
    log: Int
    act: ?Action<T, F>
    /// What each position holds, for a tree built with `of`.
    pub var source: ?(D -> T)
    var dual: Bool = false
    // The `over:` layout: node 0 stands for every untouched node, node 1 is the root over
    // `0..size`, and `lc`/`rc` hold each node's children, made on first touch.
    var lc: List<Int> = []
    var rc: List<Int> = []
    var blank: List<T> = []
    var part: List<T> = []
    base: Int = 0
    n: Len = 0

    /// A tree over `D` folding with `op`: built from `a`, filled with `fill`, or holding the identity
    /// everywhere when neither is given. `op` is a bare operator (`+`, `*`, `^`, `&`, `|`, `min`,
    /// `max`, `gcd`) or a `Monoid(id, op)`.
    /// For an op with no identity, build over `op.opt()` and `Some` cells; each fold is then `None` when empty.
    ///
    /// `act:` adds range tags (`apply`, `apply_all`); with `act:` and no `op` the tree applies tags
    /// and reads points only. When the action commutes (the stock `add`, or `commutes: true`) `apply`
    /// skips push-down, which changes speed, never answers. `over: lo..hi` covers a huge `Int` range instead, making nodes
    /// on first touch; positions outside the touched ones hold `fill` or the identity.
    ///
    /// cost: \(O(\lvert D \rvert)\); \(O(\log(hi - lo))\) with `over: lo..hi`, which then makes at
    /// most \(2 \log_2(hi - lo)\) nodes per call
    /// empty: `domain E = 0..0; SegTree<E, Int>(+).all == 0`
    /// order: positions keep the domain `D` of `a`
    /// traps: when both `a` and `fill` are given, or `over:` is given together with `a`; a tree
    /// built `over: lo..hi` traps on any position outside `lo..hi`
    /// example: `SegTree(+, [1, 2, 3]).all == 6`
    /// example: `SegTree<0..4, Int>(max, fill: 5).fold(1..3) == 5`
    /// example: `SegTree<0..4, Int>(+).all == 0`
    /// example: `var s = SegTree<Int, Int>(+, over: 0..1_000_000_000); s.set(999_999_999, 4); s.all == 4`
    /// trap: `SegTree<Int, Int>(fill: 0, over: 0..10)`
    pub fn new<M, A: Acts<T, F>, O>(
        op: M = NoOp(),
        a: ?[D]T = None,
        fill: ?T = None,
        act: A = NoAction(),
        over: O = NoOver(),
    ) -> SegTree<D, T, F>:
        if same(over, NoOver()): return SegTree<D, T, F>.on_domain(op, a, fill, act)
        assert a == None, "pass either `a` or `over:`, not both"
        if same(op, NoOp()):
            assert not same(act, NoAction()), "a tree with no fold op needs `act:`"
            let left = Monoid<T>(fill!, (x, _) => x)
            let resolved = Action<T, F>.of(act, left)
            var st = SegTree<D, T, F>.sparse(left, over, fill, Some(resolved))
            st.dual = true
            return st
        let m = Monoid<T>.of(op)
        if same(act, NoAction()): return SegTree<D, T, F>.sparse(m, over, fill, None)
        SegTree<D, T, F>.sparse(m, over, fill, Some(Action<T, F>.of(act, op)))

    fn on_domain<M, A: Acts<T, F>>(op: M, a: ?[D]T, fill: ?T, act: A) -> SegTree<D, T, F>:
        if same(op, NoOp()):
            // Dual tree: tags only, read one point at a time. Internal nodes hold tags and never
            // values, so `apply` runs on leaves only.
            assert not same(act, NoAction()), "a tree with no fold op needs `act:`"
            let seed = if let init = a: init[D.at(0)] else: fill!
            let left = Monoid<T>(seed, (x, _) => x)
            let resolved = Action<T, F>.of(act, left)
            var st = SegTree<D, T, F>.build_lazy(left, a, fill, resolved)
            st.dual = true
            return st
        let m = Monoid<T>.of(op)
        if same(act, NoAction()): return SegTree<D, T, F>.build(m, a, fill)
        SegTree<D, T, F>.build_lazy(m, a, fill, Action<T, F>.of(act, op))

    fn sparse(m: Monoid<T>, over: Range<Int>, fill: ?T, act: ?Action<T, F>) -> SegTree<D, T, F>:
        let (lo, hi) = over.ends
        let size = bit_ceil(max(hi - lo, 1))
        var st = SegTree<D, T, F>(m, [], [], [], size, ctz(size), act, None) with base: lo, n: hi - lo
        st.seed(fill)
        st

    // Reset the `over:` layout to one untouched root. `blank[k]` is the fold of `2^k` fill values
    // and `part[k]` the fold of the `n mod 2^k` fill values under the node of width `2^k` that
    // straddles `n`, so an untouched node's value is known without making it.
    fn seed(mut self, fill: ?T):
        let v = fill ?? m.id
        blank = [v]
        part = [m.id]
        for k in 0..log:
            blank.push(m.op(blank[k], blank[k]))
            part.push(if n.bit(k): m.op(blank[k], part[k]) else: part[k])
        t = [m.id]
        lc = [0]
        rc = [0]
        if let f = act:
            lz = [f.id]
            has_lz = [false]
        let _ = self.grow(0, size)

    fn blank_at(self, nl: Int, nr: Int) -> T:
        if nr <= n: return blank[ctz(nr - nl)]
        if nl >= n: return m.id
        part[ctz(nr - nl)]

    fn grow(mut self, nl: Int, nr: Int) -> Int:
        t.push(self.blank_at(nl, nr))
        lc.push(0)
        rc.push(0)
        if let f = act:
            lz.push(f.id)
            has_lz.push(false)
        t.len - 1

    // The positions of `D` under node `p`, which covers `nl..nr`.
    fn width(self, p: Int, nl: Int, nr: Int) -> Int:
        if lc.len == 0: return self.dense_width(p)
        max(0, min(nr, n) - nl)

    // The positions of `D` under dense node `p`: its leaves less the padding past `n`.
    fn dense_width(self, p: Int) -> Int:
        unchecked:
            let k = log + 1 - bit_width(p)
            return min(max(n - ((p << k) - size), 0), 1 << k)

    // The positions of `D` under dense node `p`, one of the nodes with `w` leaves.
    fn level_width(self, p: Int, w: Int) -> Int:
        unchecked:
            return min(max(size + n - p * w, 0), w)

    fn kid(self, p: Int, right: Bool) -> Int:
        if lc.len == 0: return if right: 2 * p + 1 else: 2 * p
        if right: rc[p] else: lc[p]

    @get fn count(self) -> Int = if lc.len > 0: n else: D.count()

    fn slot(self, x: Int) -> Int:
        let y = x - base
        assert y in 0..n, "the position is outside the tree's `over:` range"
        y

    // Make node `p`'s children if they are untouched, then hand them its pending tag.
    fn open_node(mut self, p: Int, nl: Int, nr: Int):
        let mid = (nl + nr) / 2
        if lc[p] == 0:
            let c = self.grow(nl, mid)
            lc[p] = c
        if rc[p] == 0:
            let c = self.grow(mid, nr)
            rc[p] = c
        if act != None: self.push(p, lc[p], rc[p], self.width(0, nl, mid), self.width(0, mid, nr), nr - nl == 2)

    fn apply_at(mut self, p: Int, nl: Int, nr: Int, lo: Int, hi: Int, f: F):
        if hi <= nl or nr <= lo: return
        if lo <= nl and nr <= hi:
            self.put(p, f, self.width(p, nl, nr), nr - nl == 1)
            return
        self.open_node(p, nl, nr)
        let mid = (nl + nr) / 2
        self.apply_at(lc[p], nl, mid, lo, hi, f)
        self.apply_at(rc[p], mid, nr, lo, hi, f)
        self.pull_node(p, lc[p], rc[p], self.width(p, nl, nr))

    fn fold_at(self, p: Int, nl: Int, nr: Int, lo: Int, hi: Int, pend: ?F) -> T:
        if hi <= nl or nr <= lo: return m.id
        if lo <= nl and nr <= hi: return self.seen(pend, p, nl, nr)
        let down = self.below(pend, p)
        let mid = (nl + nr) / 2
        m.op(self.fold_at(self.kid(p, false), nl, mid, lo, hi, down), self.fold_at(self.kid(p, true), mid, nr, lo, hi, down))

    fn pull_path(mut self, p: Int, nl: Int, nr: Int, x: Int):
        if nr - nl == 1: return
        let mid = (nl + nr) / 2
        if x < mid: self.pull_path(lc[p], nl, mid, x) else: self.pull_path(rc[p], mid, nr, x)
        self.pull_node(p, lc[p], rc[p], self.width(p, nl, nr))

    fn build(m: Monoid<T>, a: ?[D]T, fill: ?T) -> SegTree<D, T, F>:
        let size = bit_ceil(D.count())
        var t = List.filled(2 * size, m.id)
        if let init = a:
            assert fill == None, "pass either `a` or `fill`, not both"
            for i in 0..D.count(): t[size + i] = init[D.at(i)]
        elif let v = fill:
            for i in 0..D.count(): t[size + i] = v
        for p in (1..size).rev: t[p] = m.op(t[2 * p], t[2 * p + 1])
        SegTree(m, t, [], [], size, 0, None, None)

    fn build_lazy(m: Monoid<T>, a: ?[D]T, fill: ?T, resolved: Action<T, F>) -> SegTree<D, T, F>:
        let size = bit_ceil(D.count())
        var t = List.filled(2 * size, m.id)
        if let init = a:
            assert fill == None, "pass either `a` or `fill`, not both"
            for i in 0..D.count(): t[size + i] = init[D.at(i)]
        elif let v = fill:
            for i in 0..D.count(): t[size + i] = v
        for p in (1..size).rev: t[p] = m.op(t[2 * p], t[2 * p + 1])
        SegTree(
            m,
            t,
            List.filled(size, resolved.id),
            List.filled(size, false),
            size,
            ctz(size),
            Some(resolved),
            None,
            n: D.count(),
        )

    fn put(mut self, p: Int, f: F, w: Int, leaf: Bool):
        unchecked:
            if w == 0: return
            if leaf or not dual: t[p] = act!.apply(f, t[p], w)
            if not leaf:
                if has_lz[p]: act!.compose_onto(mut lz[p], f)
                else:
                    lz[p] = f
                    has_lz[p] = true

    // Hand node `p`'s pending tag to its children `l` and `r`, holding `wl` and `wr` positions.
    fn push(mut self, p: Int, l: Int, r: Int, wl: Int, wr: Int, leaf: Bool):
        unchecked:
            if not has_lz[p]: return
            has_lz[p] = false
            let f = lz[p]
            self.put(l, f, wl, leaf)
            self.put(r, f, wr, leaf)

    // Push dense node `p`'s tag onto its children, each with `w` leaves.
    fn push_at(mut self, p: Int, w: Int):
        unchecked:
            if not has_lz[p]: return
        let (l, r) = (2 * p, 2 * p + 1)
        self.push(p, l, r, self.level_width(l, w), self.level_width(r, w), l >= size)

    // Push down the tags above the ends of `l..h`: node `l >> i` sits above a cut inside it
    // exactly when `i > ctz(l)`.
    fn open(mut self, l: Int, h: Int):
        let (tl, th) = (ctz(l), ctz(h))
        var w = size
        for i in (1..=log).rev:
            w /= 2
            if i > tl: self.push_at(l >> i, w)
            if i > th: self.push_at((h - 1) >> i, w)

    // Rejoin dense node `p`, one of the nodes with `w` leaves, from its children.
    fn pull_at(mut self, p: Int, w: Int):
        unchecked:
            t[p] = m.op(t[2 * p], t[2 * p + 1])
            if act != None and has_lz[p]: t[p] = act!.apply(lz[p], t[p], self.level_width(p, w))

    // Node `p`'s value with `pend`, the tags its ancestors hold, applied.
    fn seen(self, pend: ?F, p: Int, nl: Int, nr: Int) -> T:
        let v = if p == 0: self.blank_at(nl, nr) else: t[p]
        if let f = pend:
            let w = self.width(p, nl, nr)
            if w > 0: return act!.apply(f, v, w)
        v

    // The tags pending above `p`'s children: `pend`, after `p`'s own.
    fn below(self, pend: ?F, p: Int) -> ?F:
        if act == None or not has_lz[p]: return pend
        if let f = pend: return Some(act!.compose(f, lz[p]))
        Some(lz[p])

    // Walk right from `from` under node `p` (covering `nl..nr`), folding the leaves `ok` holds on
    // into `acc` (None before the first): the first leaf where `ok` fails, or -1. Reads pending
    // tags on the way down instead of pushing them.
    fn right_from(self, p: Int, nl: Int, nr: Int, from: Int, pend: ?F, mut acc: ?T, ok: T -> Bool) -> Int:
        if nr <= from: return -1
        if from <= nl:
            let w = if acc == None: self.seen(pend, p, nl, nr) else: m.op(acc!, self.seen(pend, p, nl, nr))
            if ok(w):
                acc = Some(w)
                return -1
            if nr - nl == 1: return nl
        let down = self.below(pend, p)
        let mid = (nl + nr) / 2
        let at = self.right_from(self.kid(p, false), nl, mid, from, down, mut acc, ok)
        if at >= 0: return at
        self.right_from(self.kid(p, true), mid, nr, from, down, mut acc, ok)

    fn left_from(self, p: Int, nl: Int, nr: Int, to: Int, pend: ?F, mut acc: ?T, ok: T -> Bool) -> Int:
        if nl >= to: return -1
        if nr <= to:
            let w = if acc == None: self.seen(pend, p, nl, nr) else: m.op(self.seen(pend, p, nl, nr), acc!)
            if ok(w):
                acc = Some(w)
                return -1
            if nr - nl == 1: return nr
        let down = self.below(pend, p)
        let mid = (nl + nr) / 2
        let at = self.left_from(self.kid(p, true), mid, nr, to, down, mut acc, ok)
        if at >= 0: return at
        self.left_from(self.kid(p, false), nl, mid, to, down, mut acc, ok)

    // `x`, folded over `n` leaves under node `p`, with the tags of `p`'s ancestors applied.
    fn lift(self, x: T, n: Int, p: Int) -> T:
        if n == 0: return x
        var (v, q) = (x, p)
        unchecked:
            while q > 1:
                q /= 2
                if has_lz[q]: v = act!.apply(lz[q], v, n)
        v

    fn pull_node(mut self, p: Int, l: Int, r: Int, w: Int):
        unchecked:
            t[p] = m.op(t[l], t[r])
            if act != None and has_lz[p]: t[p] = act!.apply(lz[p], t[p], w)

    /// Apply tag `f` to every position of `r`: each fold over `r` then sees the values with `f`
    /// applied, through the action given to `act:`.
    ///
    /// cost: \(O(\log \lvert D \rvert)\) tag puts; a put is one `compose_in` (else `compose`) on a
    /// node that holds a tag and one copy of the tag onto a node that holds none
    /// empty: `var s = SegTree(+, [1, 2, 3], act: add); s.apply(1..1, 10); s.all == 6`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var s = SegTree(+, [1, 2, 3], act: add); s.apply(0..2, 10); s.all == 26`
    /// example: `var s = SegTree(max, [1, 2, 3], act: add); s.apply(0..1, 5); s.all == 6`
    /// example: `var s = SegTree(+, [1, 2, 3], act: affine); s.apply(0..2, (2, 1)); s.all == 11`
    pub fn apply(mut self, r: Range<D>, f: F):
        assert act != None, "apply requires `act:` on SegTree"
        let (lo, hi) = r.ends
        if lo >= hi: return
        if lc.len > 0:
            self.apply_at(1, 0, size, self.slot(lo), self.slot(hi - 1) + 1, f)
            return
        let (l0, h0) = (lo + size, hi + size)
        if dual or not act!.commutes: self.open(l0, h0)
        // Every node the loop tags lies inside `r`, so it holds all `w` of its leaves.
        var (l, h, w) = (l0, h0, 1)
        unchecked:
            while l < h:
                if l % 2 == 1:
                    self.put(l, f, w, l >= size)
                    l += 1
                if h % 2 == 1:
                    h -= 1
                    self.put(h, f, w, h >= size)
                l /= 2
                h /= 2
                w += w
        if dual: return
        let (tl, th) = (ctz(l0), ctz(h0))
        w = 1
        for i in 1..=log:
            w += w
            if i > tl: self.pull_at(l0 >> i, w)
            if i > th: self.pull_at((h0 - 1) >> i, w)

    /// Apply tag `f` to every position; `apply(D.all, f)`.
    ///
    /// cost: \(O(1)\) amortized when the tags commute, else \(O(\lvert D \rvert)\)
    /// empty: `var s = SegTree<0..0, Int, Int>(+, act: add); s.apply_all(5); s.all == 0`
    /// traps: never
    /// example: `var s = SegTree(+, [1, 2, 3], act: add); s.apply_all(1); s.all == 9`
    pub fn apply_all(mut self, f: F):
        assert act != None, "apply_all requires `act:` on SegTree"
        if self.count > 0: self.put(1, f, self.width(1, 0, size), size == 1)

    /// The fold of the values in `r`, left to right: `op(op(a[l], a[l + 1]), ...)`. With an
    /// action that declares `replaces:` (as `assign` does), the fold stops at the first node whose
    /// pending tag sets its span and answers from that tag, as a recursive lazy tree does.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// empty: `SegTree(min, [1, 2]).fold(1..1) == Int.MAX`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var s = SegTree(+, [1, 2, 3], act: add); s.apply(1..3, 1); s.fold(1..3) == 7`
    pub fn fold(self, r: Range<D>) -> T:
        let (lo, hi) = r.ends
        if lc.len > 0 or act != None: return self.fold_tagged(lo, hi)
        self.fold_leaves(lo + size, hi + size)

    // Without tags or a dynamic tree, a fold is this one loop, short enough for g++ to inline
    // into the caller's query loop the way a hand-written segment tree's is.
    fn fold_leaves(self, lo: Int, hi: Int) -> T:
        var (l, h) = (lo, hi)
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

    fn fold_tagged(self, lo: Int, hi: Int) -> T:
        if lc.len > 0: return if lo < hi: self.fold_at(1, 0, size, self.slot(lo), self.slot(hi - 1) + 1, None) else: m.id
        var (l, h) = (lo + size, hi + size)
        var (a, b) = (m.id, m.id)
        let f = act!
        if let set = f.replaces: return if lo < hi: self.fold_set(1, 0, size, lo, hi, set) else: m.id
        // `a` folds `na` leaves under node `l - 1`, `b` folds `nb` under node `h`; each takes the
        // tags of the nodes it climbs through, so nothing is pushed.
        var (na, nb, w) = (0, 0, 1)
        unchecked:
            while l < h:
                if l % 2 == 1:
                    a = m.op(a, t[l])
                    na += w
                    l += 1
                if h % 2 == 1:
                    h -= 1
                    b = m.op(t[h], b)
                    nb += w
                l /= 2
                h /= 2
                if na > 0 and has_lz[l - 1]: a = f.apply(lz[l - 1], a, na)
                if nb > 0 and has_lz[h]: b = f.apply(lz[h], b, nb)
                w += w
        m.op(self.lift(a, na, l - 1), self.lift(b, nb, h))

    // Top-down fold that answers from the first tag that sets its whole span.
    fn fold_set(self, p: Int, nl: Int, nr: Int, lo: Int, hi: Int, set: F -> Bool) -> T:
        unchecked:
            if lo <= nl and nr <= hi: return t[p]
            let w = min(nr, hi) - max(nl, lo)
            let tagged = has_lz[p]
            if tagged and set(lz[p]): return act!.apply(lz[p], t[p], w)
            let mid = (nl + nr) / 2
            var x = m.id
            if hi <= mid: x = self.fold_set(2 * p, nl, mid, lo, hi, set)
            elif mid <= lo: x = self.fold_set(2 * p + 1, mid, nr, lo, hi, set)
            else: x = m.op(self.fold_set(2 * p, nl, mid, lo, hi, set), self.fold_set(2 * p + 1, mid, nr, lo, hi, set))
            if tagged: return act!.apply(lz[p], x, w)
            return x

    /// The last cut `d` at or right of `from` such that `ok(fold(from..d))` holds. `ok` must be
    /// monotone (once false it stays false as `d` moves right) and true on the identity, so `from`
    /// itself always qualifies. Replaces `max_right`.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// ties: `SegTree(+, [1, 0, 0, 5]).right_while((0..0).r, s => s <= 1) == (0..3).r`
    /// traps: when `ok(identity)` is false
    /// example: `SegTree(+, [1, 2, 3, 4]).right_while((0..0).r, s => s <= 3) == (0..2).r`
    /// example: `SegTree(+, [1, 2, 3, 4]).right_while((0..1).r, s => s <= 5) == (0..3).r`
    /// example: `SegTree(+, [1, 2, 3, 4]).right_while((0..0).r, s => s <= 100) == (0..4).r`
    pub fn right_while(self, from: Cut<D>, ok: T -> Bool) -> Cut<D>:
        assert ok(m.id), "`ok` must hold on the identity"
        let k = self.count
        let x = from.ord - base
        if x == k: return from
        var acc: ?T = None
        let at = self.right_from(1, 0, size, x, None, mut acc, ok)
        D.cut(base + (if at < 0: k else: at))

    /// The first cut `d` at or left of `to` such that `ok(fold(d..to))` holds. `ok` must be
    /// monotone (once false it stays false as `d` moves left) and true on the identity, so `to`
    /// itself always qualifies. Replaces `min_left`.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// ties: `SegTree(+, [5, 0, 0, 1]).left_while((0..4).r, s => s <= 1) == (0..1).r`
    /// traps: when `ok(identity)` is false
    /// example: `SegTree(+, [1, 2, 3, 4]).left_while((0..4).r, s => s <= 7) == (0..2).r`
    /// example: `SegTree(+, [1, 2, 3, 4]).left_while((0..4).r, s => s <= 3) == (0..4).r`
    pub fn left_while(self, to: Cut<D>, ok: T -> Bool) -> Cut<D>:
        assert ok(m.id), "`ok` must hold on the identity"
        let x = to.ord - base
        if x == 0: return to
        var acc: ?T = None
        let at = self.left_from(1, 0, size, x, None, mut acc, ok)
        D.cut(base + (if at < 0: 0 else: at))

    /// A tree whose position `i` holds `f(i)`. `f` is kept: when something it reads changes,
    /// `refresh(i)` recomputes position `i`, so the formula is written once instead of at every
    /// update site.
    /// For an op with no identity, build over `op.opt()` and `Some` cells; each fold is then `None` when empty.
    ///
    /// cost: \(O(\lvert D \rvert)\), calling `f` once per position in key order
    /// empty: `SegTree<0..0, Int>.of(+, i => 1).all == 0`
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
        if lc.len > 0:
            let x = self.slot(i.ord)
            var (p, nl, nr) = (1, 0, size)
            var pend: ?F = None
            while nr - nl > 1:
                pend = self.below(pend, p)
                let mid = (nl + nr) / 2
                if x < mid:
                    p = self.kid(p, false)
                    nr = mid
                else:
                    p = self.kid(p, true)
                    nl = mid
            return self.seen(pend, p, nl, nr)
        let p = size + i.ord
        if act == None: return t[p]
        var v = t[p]
        unchecked:
            for k in 1..=log:
                if has_lz[p >> k]: v = act!.apply(lz[p >> k], v, 1)
        v

    /// Make position `i` hold `x`, replacing any tags applied to it, and update its ancestors.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// traps: when `i` is outside `D`
    /// example: `var s = SegTree(+, [1, 2, 3]); s.set(0, 10); s.all == 15`
    @lint(hand_rolled_range_tree, from: "while i > 0:\n    t[i] = t[2 * i] + t[2 * i + 1]\n    i >>= 1", to: "t.set(i, t[i])", holes: "t")
    @lint(hand_rolled_range_tree, from: "while i > 1:\n    i >>= 1\n    t[i] = t[2 * i] + t[2 * i + 1]", to: "t.set(i, t[i])", holes: "t")
    @lint(hand_rolled_range_tree, from: "while i > 1:\n    i >>= 1\n    t[i] = f(t[2 * i], t[2 * i + 1])", to: "t.set(i, t[i])", holes: "t, f")
    pub fn set(mut self, i: D, x: T):
        let p = self.place(i)
        t[p] = x
        self.pull(i)

    /// Position `i` as a writable place: `st.at(i) += x`, `st.at(i) max= x`. The ancestors are
    /// updated when the write ends, so each use is one `set`; read with `get` instead.
    ///
    /// cost: \(O(\log \lvert D \rvert)\) per write
    /// traps: when `i` is outside `D`
    /// example: `var s = SegTree(+, [1, 2, 3]); s.at(0) += 5; s.all == 11`
    pub fn at(mut self, i: D) -> lends mut T:
        let p = self.place(i)
        lend mut t[p]
        self.pull(i)

    // The leaf for `i`, with every tag above it pushed down (and, with `over:`, its path made).
    fn place(mut self, i: D) -> Int:
        if lc.len == 0:
            let p = size + i.ord
            if act != None:
                var w = size
                for k in (1..=log).rev:
                    w /= 2
                    self.push_at(p >> k, w)
            return p
        let x = self.slot(i.ord)
        var (p, nl, nr) = (1, 0, size)
        while nr - nl > 1:
            self.open_node(p, nl, nr)
            let mid = (nl + nr) / 2
            if x < mid:
                p = lc[p]
                nr = mid
            else:
                p = rc[p]
                nl = mid
        p

    fn pull(mut self, i: D):
        if lc.len > 0:
            self.pull_path(1, 0, size, self.slot(i.ord))
            return
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
    /// empty: `SegTree<0..0, Int>(+).all == 0 and SegTree<0..0, Int>(max).all == Int.MIN and SegTree<0..0, ?Int>(max.opt()).all == None`
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
    /// empty: `SegTree(+, [1, 2]).try_fold(2..2) == None`
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
    /// empty: `var s = SegTree<0..0, Int>(+); s.clear(fill: 2); s.all == 0`
    /// traps: never
    /// example: `var s = SegTree(+, [1, 2, 3]); s.clear(); s.all == 0`
    /// example: `var s = SegTree(+, [1, 2, 3]); s.clear(fill: 2); s.all == 6`
    pub fn clear(mut self, fill: ?T = None):
        if lc.len > 0:
            self.seed(fill)
            return
        let v = fill ?? m.id
        if let f = act:
            for p in 0..has_lz.len:
                lz[p] = f.id
                has_lz[p] = false
        for i in 0..size: t[size + i] = if i < D.count(): v else: m.id
        for p in (1..size).rev: t[p] = m.op(t[2 * p], t[2 * p + 1])

    /// The values of the canonical nodes that exactly cover `r`: the maximal aligned blocks of the
    /// tree's `bit_ceil(|D|)` leaves inside `r`. Their fold, left to right, is `fold(r)`.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// empty: `SegTree(+, [1, 2, 3, 4]).walk(2..2).collect() == []`
    /// order: left to right; at most `2 * log2(bit_ceil(|D|))` values
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `SegTree(+, [1, 2, 3, 4]).walk(0..3).collect() == [3, 3]`
    /// example: `SegTree(+, [1, 2, 3, 4]).walk(1..4).collect() == [2, 7]`
    pub fn walk(self, r: Range<D>) -> yields T:
        let (lo, hi) = r.ends
        if lc.len > 0 or act != None:
            if lo >= hi: return
            let (l, h) = if lc.len > 0: (self.slot(lo), self.slot(hi - 1) + 1) else: (lo, hi)
            var stack: List<(Int, Int, Int, ?F)> = [(1, 0, size, None)]
            while let (p, nl, nr, pend) = stack.pop():
                if h <= nl or nr <= l: continue
                if l <= nl and nr <= h:
                    yield self.seen(pend, p, nl, nr)
                    continue
                let down = self.below(pend, p)
                let mid = (nl + nr) / 2
                stack.push((self.kid(p, true), mid, nr, down))
                stack.push((self.kid(p, false), nl, mid, down))
            return
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
            a[i] max= x
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
fn pushed_tags_keep_their_order():
    let n = 13
    var a = [rand(-5, 5) for _ in 0..n]
    var st = SegTree(+, a, act: affine)
    for _ in 0..300:
        let l = rand(0, n - 1)
        let r = rand(l + 1, n)
        let f = (if rand(0, 1) == 0: -1 else: 1, rand(-3, 3))
        st.apply(l..r, f)
        for i in l..r: a[i] = f.0 * a[i] + f.1
        let ql = rand(0, n - 1)
        let qr = rand(ql + 1, n)
        assert st.fold(ql..qr) == sum(a[i] for i in ql..qr)
    assert st.all == a.sum()

@test
fn an_assign_tree_folds_an_empty_range_to_the_identity():
    domain V = 1..=10
    var st = SegTree(min, [V] v => 0, act: assign)
    st.apply(V.nth(0)..=V.nth(9), Some(3))
    assert st.fold(..V.nth(0)) == Int.MAX and st.fold(V.nth(4)..V.nth(4)) == Int.MAX
    assert st.fold(V.nth(2)..V.nth(5)) == 3

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

@test
fn over_matches_a_plain_array():
    domain P = 0..=100
    for (lo, hi) in [(0, 1), (3, 10), (7, 40), (0, 33), (50, 100)]:
        let k = hi - lo
        let v = rand(-3, 3)
        var a = List.filled(k, v)
        var s = SegTree<P, Int>(+, fill: v, act: add, over: lo..hi)
        var top = SegTree<P, Int>(max, fill: v, over: lo..hi)
        for _ in 0..80:
            let l = rand(0, k - 1)
            let h = rand(l + 1, k)
            let x = rand(-9, 9)
            if rand(0, 1) == 0:
                a[l] = x
                s.set(P(lo + l), x)
                top.set(P(lo + l), x)
            else:
                for j in l..h: a[j] += x
                s.apply(P(lo + l)..P(lo + h), x)
                for j in l..h: top.set(P(lo + j), a[j])
            assert s.fold(P(lo + l)..P(lo + h)) == sum(a[j] for j in l..h)
            assert top.fold(P(lo + l)..P(lo + h)) == [a[j] for j in l..h].max()!
            assert s.get(P(lo + l)) == a[l] and top.get(P(lo + h - 1)) == a[h - 1]
            assert s.all == a.sum() and top.all == a.max()!

/// Static merge-sort tree for orthogonal 2D range counting over `[D]T`.
/// Build in \(O(n \log n)\) time and memory; answer value-range counts over any `Range<D>` in \(O(\log^2 n)\).
pub struct MergeSortTree<D, T>:
    data: List<T>
    beg: List<0..=2_000_000_000>
    fin: List<0..=2_000_000_000>
    size: Len

    /// A static tree over `a` whose nodes hold their values sorted, for counting values in a range.
    ///
    /// cost: \(O(\lvert D \rvert \log \lvert D \rvert)\) time and memory
    /// empty: `MergeSortTree<0..0, Int>([]).count_le(0..0, 3) == 0`
    /// order: positions keep the domain `D` of `a`
    /// traps: never
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).count_lt(0..5, 3) == 2`
    pub fn new(a: [D]T) -> MergeSortTree<D, T> where T: Ord:
        let size = bit_ceil(D.count)
        var data: List<T> = []
        var beg: List<Pos> = List.filled(2 * size, 0)
        var fin: List<Pos> = List.filled(2 * size, 0)
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

    fn lower_idx(self, u: Int, x: T) -> Int where T: Ord:
        unchecked:
            let base = beg[u]
            var (lo, hi) = (base, fin[u])
            while lo < hi:
                let mid = (lo + hi) / 2
                if data[mid] < x: lo = mid + 1
                else: hi = mid
            return lo - base

    fn upper_idx(self, u: Int, x: T) -> Int where T: Ord:
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
    /// empty: `MergeSortTree([3, 1, 4]).count_lt(1..1, 9) == 0`
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
    /// empty: `MergeSortTree([3, 1, 4]).count_le(1..1, 9) == 0`
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
    /// empty: `MergeSortTree([3, 1, 4]).count_in(1..1, 0, 9) == 0`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).count_in(1..4, 1, 4) == 3`
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).count_in(0..5, 4, 3) == 0`
    pub fn count_in(self, r: Range<D>, lo_v: T, hi_v: T) -> Int where T: Ord:
        if lo_v > hi_v: return 0
        self.count_le(r, hi_v) - self.count_lt(r, lo_v)

    /// The number of positions in `r` whose value is `> x`.
    ///
    /// cost: \(O(\log^2 \lvert D \rvert)\)
    /// empty: `MergeSortTree([3, 1, 4]).count_gt(1..1, 0) == 0`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).count_gt(0..5, 3) == 2`
    pub fn count_gt(self, r: Range<D>, x: T) -> Int:
        let (lo, hi) = r.ends
        (hi - lo) - self.count_le(r, x)

    /// The number of positions in `r` whose value is `>= x`.
    ///
    /// cost: \(O(\log^2 \lvert D \rvert)\)
    /// empty: `MergeSortTree([3, 1, 4]).count_ge(1..1, 0) == 0`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).count_ge(0..5, 4) == 2`
    pub fn count_ge(self, r: Range<D>, x: T) -> Int:
        let (lo, hi) = r.ends
        (hi - lo) - self.count_lt(r, x)

    /// The smallest value in `r` that is `>= x`, or `None`.
    /// It returns a value, so equal values are interchangeable.
    ///
    /// cost: \(O(\log^2 \lvert D \rvert)\)
    /// empty: `MergeSortTree([3, 1, 4]).first_ge(1..1, 0) == None`
    /// ties: `MergeSortTree([3, 1, 3]).first_ge(0..3, 2) == Some(3)`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).first_ge(0..3, 2) == Some(3)`
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).first_ge(0..3, 5) == None`
    pub fn first_ge(self, r: Range<D>, x: T) -> ?T where T: Ord:
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
    /// It returns a value, so equal values are interchangeable.
    ///
    /// cost: \(O(\log^2 \lvert D \rvert)\)
    /// empty: `MergeSortTree([3, 1, 4]).last_le(1..1, 9) == None`
    /// ties: `MergeSortTree([1, 4, 1]).last_le(0..3, 2) == Some(1)`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).last_le(2..5, 3) == Some(1)`
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).last_le(2..5, 0) == None`
    pub fn last_le(self, r: Range<D>, x: T) -> ?T where T: Ord:
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

/// Iterative 2D segment tree: point update and rectangle fold in \(O(\log \lvert R \rvert \cdot
/// \log \lvert C \rvert)\). \(O(\lvert R \rvert \cdot \lvert C \rvert)\) space.
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
    /// empty: `SegTree2D<0..0, 0..3, Int>(+).all == 0`
    /// order: cells keep the domains `R` and `C` of `a`
    /// traps: when both `a` and `fill` are given
    /// example: `SegTree2D<0..3, 0..3, Int>(+, fill: 1).all == 9`
    /// trap: `domain R = 0..2; let g: [R, R]Int = 1; SegTree2D<R, R, Int>(+, a: g, fill: 1)`
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
    /// empty: `let g = SegTree2D<0..3, 0..3, Int>(min, fill: 1); g.fold(1..1, 0..3) == Int.MAX and g.fold(0..3, 2..2) == Int.MAX`
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
    /// empty: `SegTree2D<0..2, 0..0, Int>(+).all == 0`
    /// traps: never
    /// example: `var g = SegTree2D<0..2, 0..2, Int>(max, fill: 0); g.set(1, 0, 8); g.all == 8`
    @get pub fn all(self) -> T:
        if R.count == 0 or C.count == 0: m.id else: t[2 * C.count + 1]

/// Wavelet matrix over non-negative integer values: \(O(\log V)\) range k-th smallest, frequency,
/// value-range count, predecessor/successor, and quantile sum. \(O(\lvert D \rvert \log V)\) build
/// and memory.
pub struct WaveletMatrix<D>:
    bits: Int
    n: Len
    zs: List<Int>
    // Level `l`'s bits packed 64 to a word, with the count of ones before each word, so a
    // query's rank lookups touch a few hot words instead of a full Int per position.
    words: List<U64>
    before: List<Int>
    zsum: List<Int>

    fn ones(self, l: Int, i: Int) -> Int:
        let at = l * (n / 64 + 1) + i / 64
        before[at] + popcount(words[at] & ((U64(1) << (i % 64)) - U64(1)))

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
    /// empty: `WaveletMatrix<0..0>([]).count_lt(0..0, 3) == 0`
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
            List.filled(bits * (n / 64 + 1), U64(0)),
            List.filled(bits * (n / 64 + 1), 0),
            List.filled(bits * (n + 1), 0),
        )
        for l in 0..bits:
            let b = bits - 1 - l
            let base = l * (n + 1)
            let wbase = l * (n / 64 + 1)
            var (zero, one): (List<Int>, List<Int>) = ([], [])
            for i in 0..n:
                let v = cur[i]
                if v.bit(b):
                    w.words[wbase + i / 64] |= U64(1) << (i % 64)
                    w.zsum[base + i + 1] = w.zsum[base + i]
                    one.push(v)
                else:
                    w.zsum[base + i + 1] = w.zsum[base + i] + v
                    zero.push(v)
            for k in 1..n / 64 + 1:
                w.before[wbase + k] = w.before[wbase + k - 1] + popcount(w.words[wbase + k - 1])
            w.zs.push(zero.len)
            for v in one: zero.push(v)
            cur = zero
        w

    /// The `k`-th smallest value in `r`, counting from 0 and with repeats, or `None` when
    /// `k >= r.len`.
    ///
    /// cost: \(O(\log V)\)
    /// empty: `WaveletMatrix([3, 1, 4]).kth(1..1, 0) == None`
    /// ties: `let w = WaveletMatrix([2, 1, 2]); w.kth(0..3, 1) == Some(2) and w.kth(0..3, 2) == Some(2)`
    /// traps: when `k < 0`, or a non-empty `r` reaches outside `D`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).kth(0..5, 0) == Some(1)`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).kth(0..5, 2) == Some(3)`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).kth(0..5, 5) == None`
    pub fn kth(self, r: Range<D>, k: 0..) -> ?Int:
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
    /// empty: `WaveletMatrix([3, 1, 4]).rank(1..1, 1) == 0`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).rank(0..5, 1) == 2`
    pub fn rank(self, r: Range<D>, x: Int) -> Int:
        let (lo, hi) = r.ends
        if x < 0: return 0
        self.below(lo, hi, x + 1) - self.below(lo, hi, x)

    /// The number of positions in `r` whose value is `< x`.
    ///
    /// cost: \(O(\log V)\)
    /// empty: `WaveletMatrix([3, 1, 4]).count_lt(1..1, 9) == 0`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).count_lt(0..5, 4) == 3`
    pub fn count_lt(self, r: Range<D>, x: Int) -> Int:
        let (lo, hi) = r.ends
        self.below(lo, hi, x)

    /// The number of positions in `r` whose value lies in `lo_v..=hi_v`; 0 when `lo_v > hi_v`.
    ///
    /// cost: \(O(\log V)\)
    /// empty: `WaveletMatrix([3, 1, 4]).count_in(1..1, 0, 9) == 0`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).count_in(0..5, 1, 3) == 3`
    pub fn count_in(self, r: Range<D>, lo_v: Int, hi_v: Int) -> Int:
        let (lo, hi) = r.ends
        if lo_v > hi_v: return 0
        self.below(lo, hi, hi_v + 1) - self.below(lo, hi, lo_v)

    /// The largest value in `r` that is `< x`, or `None`.
    /// It returns a value, so equal values are interchangeable.
    ///
    /// cost: \(O(\log V)\)
    /// empty: `WaveletMatrix([3, 1, 4]).prev_val(1..1, 9) == None`
    /// ties: `WaveletMatrix([1, 4, 1]).prev_val(0..3, 3) == Some(1)`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).prev_val(0..5, 3) == Some(1)`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).prev_val(0..5, 1) == None`
    pub fn prev_val(self, r: Range<D>, x: Int) -> ?Int:
        let (lo, hi) = r.ends
        let c = self.below(lo, hi, x)
        if c == 0: None else: self.kth(r, c - 1)

    /// The smallest value in `r` that is `> x`, or `None`.
    /// It returns a value, so equal values are interchangeable.
    ///
    /// cost: \(O(\log V)\)
    /// empty: `WaveletMatrix([3, 1, 4]).next_val(1..1, 0) == None`
    /// ties: `WaveletMatrix([4, 1, 4]).next_val(0..3, 3) == Some(4)`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).next_val(0..5, 3) == Some(4)`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).next_val(0..5, 5) == None`
    pub fn next_val(self, r: Range<D>, x: Int) -> ?Int:
        let (lo, hi) = r.ends
        let c = self.below(lo, hi, x + 1)
        if c >= hi - lo: None else: self.kth(r, c)

    /// The sum of the `k` smallest values in `r`, with repeats.
    /// On an empty `r` only `k == 0` is allowed.
    ///
    /// cost: \(O(\log V)\)
    /// empty: `WaveletMatrix([3, 1, 4]).quantile_sum(1..1, 0) == 0`
    /// ties: `WaveletMatrix([2, 1, 2]).quantile_sum(0..3, 3) == 5`
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

/// \(O(\lvert D \rvert \log^2 \lvert D \rvert)\) space, or \(O(\lvert vs \rvert + \lvert D \rvert \log \lvert vs \rvert)\) built with
/// `values: vs`. Dynamic point-update + 2D range value-count and k-th structure (Fenwick of order
/// trees, or a segment tree over the given values' ranks) over `[D]T`.
pub struct DynamicOrderTree<D, T>:
    n: Len
    val: List<T> = []
    // `bit[j]` for `j >= 1` holds the values at positions `j - (j & -j)..j`; `bit[0]` holds
    // every value, the candidates `kth` searches.
    var bit: List<SortedBag<T>> = []
    // With `values:`, the sorted distinct values `vs`, and `at[p]`, a segment tree of `sz` leaves
    // over their ranks whose node `p` holds the positions whose value's rank is under it.
    var ranked: Bool = false
    vs: List<T> = []
    var sz: Int = 1
    var at: List<SortedBag<Int>> = []

    fn rank_of(self, x: T, le: Bool) -> Int = (0..vs.len).first_true(
        i => if le: vs[i] > x else: vs[i] >= x
    ) ?? vs.len

    fn bump(mut self, i: Int, x: T, put: Bool) where T: Ord:
        if ranked:
            let b = self.rank_of(x, false)
            assert b < vs.len and
                vs[b] == x, "a `DynamicOrderTree` built with `values:` holds only those values"
            var p = b + sz
            while p >= 1:
                if put: at[p].add(i)
                else: at[p].remove(i)
                p /= 2
            return
        if put: bit[0].add(x)
        else: bit[0].remove(x)
        var j = i + 1
        while j <= n:
            if put: bit[j].add(x)
            else: bit[j].remove(x)
            j += j & -j

    fn below(self, end: Int, x: T, le: Bool) -> Int where T: Ord:
        var acc = 0
        var j = end
        if ranked:
            var (l, h) = (sz, self.rank_of(x, le) + sz)
            while l < h:
                if l % 2 == 1:
                    acc += at[l].rank(end)
                    l += 1
                if h % 2 == 1:
                    h -= 1
                    acc += at[h].rank(end)
                l /= 2
                h /= 2
            return acc
        while j > 0:
            acc += bit[j].rank(x)
            if le: acc += bit[j].count(x)
            j -= j & -j
        acc

    fn ends(self, r: Range<D>) -> (Int, Int) = (r.first!.ord, r.last!.ord + 1)

    fn span_count(self, r: Range<D>, x: T, le: Bool) -> Int:
        if r.is_empty: return 0
        let (lo, hi) = self.ends(r)
        self.below(hi, x, le) - self.below(lo, x, le)

    /// A structure over `a` answering value counts and order statistics on ranges while positions
    /// are reassigned.
    ///
    /// With `values: vs`, every value it will hold, given ahead: a segment tree over their ranks
    /// holds each node's positions in order, so `kth` descends it in \(O(\log \lvert D \rvert \log \lvert vs \rvert)\) instead of searching values.
    ///
    /// cost: \(O(\lvert D \rvert \log^2 \lvert D \rvert)\) time and memory; with `values: vs`, \(O(\lvert vs \rvert \log \lvert vs \rvert + \lvert D \rvert \log \lvert D \rvert \log \lvert vs \rvert)\) time and \(O(\lvert vs \rvert + \lvert D \rvert \log \lvert vs \rvert)\) memory
    /// empty: `DynamicOrderTree<0..0, Int>([]).count_lt(0..0, 3) == 0`
    /// order: positions keep the domain `D` of `a`
    /// traps: with `values:`, when `a` holds a value not in it
    /// example: `DynamicOrderTree([3, 1, 4]).count_lt(0..3, 4) == 2`
    /// example: `DynamicOrderTree([3, 1, 4], values: [1, 3, 4, 9]).kth(0..3, 2) == Some(4)`
    pub fn new(a: [D]T, values: ?List<T> = None) -> DynamicOrderTree<D, T> where T: Ord:
        let val = [a[i] for i in D]
        if let given = values:
            var vs = List<T>()
            for x in given.sorted():
                if vs.is_empty or vs.last! != x: vs.push(x)
            var sz = 1
            while sz < vs.len: sz *= 2
            let at = [SortedBag<Int>.new() for _ in 0..2 * sz]
            var t = DynamicOrderTree<D, T>(n: D.count, val:, ranked: true, vs:, sz:, at:)
            for i in 0..D.count: t.bump(i, t.val[i], true)
            return t
        var t = DynamicOrderTree<D, T>(n: D.count, val:, bit: [SortedBag<T>.new() for _ in 0..D.count + 1])
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
    /// empty: `DynamicOrderTree([3, 1, 4]).count_lt(1..1, 9) == 0`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `DynamicOrderTree([3, 1, 4]).count_lt(1..3, 4) == 1`
    pub fn count_lt(self, r: Range<D>, x: T) -> Int = self.span_count(r, x, false)

    /// The number of positions in `r` whose value lies in `lo_v..=hi_v`; 0 when `lo_v > hi_v`.
    ///
    /// cost: \(O(\log^2 \lvert D \rvert)\)
    /// empty: `DynamicOrderTree([3, 1, 4]).count_in(1..1, 0, 9) == 0`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `DynamicOrderTree([3, 1, 4]).count_in(0..3, 1, 3) == 2`
    pub fn count_in(self, r: Range<D>, lo_v: T, hi_v: T) -> Int where T: Ord:
        if hi_v < lo_v: return 0
        self.span_count(r, hi_v, true) - self.span_count(r, lo_v, false)

    /// The `k`-th smallest value in `r`, counting from 0 and with repeats, or `None` when
    /// `k >= r.len`.
    ///
    /// cost: \(O(\log^3 \lvert D \rvert)\): a binary search over all stored values, counting each in `r`; with `values: vs`, \(O(\log \lvert D \rvert \log \lvert vs \rvert)\), one descent of the rank tree
    /// empty: `DynamicOrderTree([3, 1, 4]).kth(1..1, 0) == None`
    /// ties: `let t = DynamicOrderTree([2, 1, 2]); t.kth(0..3, 1) == Some(2) and t.kth(0..3, 2) == Some(2)`
    /// traps: when `k < 0`, or a non-empty `r` reaches outside `D`
    /// example: `DynamicOrderTree([3, 1, 4]).kth(0..3, 1) == Some(3)`
    /// example: `DynamicOrderTree([3, 1, 4]).kth(0..3, 3) == None`
    pub fn kth(self, r: Range<D>, k: 0..) -> ?T where T: Ord:
        assert k >= 0, "`kth` takes `k >= 0`"
        if k >= r.len: return None
        if ranked: return self.walk(r, k)
        let lo = (0..bit[0].len - 1).first_true(mid => self.span_count(r, bit[0].nth(mid)!, true) > k) ?? (bit[0].len - 1)
        bit[0].nth(lo)

// The `k`-th smallest value in `r`, descending the rank tree by how many of `r`'s
// positions each left child holds.
impl DynamicOrderTree<D, T>:
    fn walk(self, r: Range<D>, k: Int) -> ?T:
        let (l, h) = self.ends(r)
        var (p, left) = (1, k)
        while p < sz:
            let here = at[2 * p].rank(h) - at[2 * p].rank(l)
            if left < here: p = 2 * p
            else:
                left -= here
                p = 2 * p + 1
        vs.get(p - sz)

/// One set of positions of `D` in a `MergeSegs<D>` pool: a handle, so copying it is O(1).
/// `MergeSeg<D>()` is the empty set.
pub struct MergeSeg<D>:
    var root: Int = 0

/// A pool of mergeable sets of positions of `D`, each stored as a sparse binary segment tree whose
/// nodes the sets share. `merge` builds new nodes only where both trees overlap and leaves both inputs
/// valid, and `first_alive` drops dead positions for good, so a lazily pruned minimum-alive query is
/// \(O(\log \lvert D \rvert)\) amortized.
pub struct MergeSegs<D>:
    ls: List<0..=2_000_000_000> = [0]
    rs: List<0..=2_000_000_000> = [0]

    fn node(mut self, l: Int, r: Int) -> Int:
        ls.push(l)
        rs.push(r)
        ls.len - 1

    fn build(mut self, k: Int, lo: Int, hi: Int) -> Int:
        if lo + 1 == hi: return self.node(0, 0)
        let mid = (lo + hi) / 2
        if k < mid: return self.node(self.build(k, lo, mid), 0)
        self.node(0, self.build(k, mid, hi))

    fn cat(mut self, a: Int, b: Int, lo: Int, hi: Int) -> Int:
        if a == 0: return b
        if b == 0 or lo + 1 == hi: return a
        let mid = (lo + hi) / 2
        let l = self.cat(ls[a], ls[b], lo, mid)
        let r = self.cat(rs[a], rs[b], mid, hi)
        self.node(l, r)

    fn prune(mut self, u: Int, lo: Int, hi: Int, alive: D -> Bool) -> (Int, ?D):
        if u == 0: return (0, None)
        if lo + 1 == hi:
            let x = D.nth(lo)
            return if alive(x): (u, Some(x)) else: (0, None)
        let mid = (lo + hi) / 2
        let (l, a) = self.prune(ls[u], lo, mid, alive)
        ls[u] = l
        if let x = a: return (u, Some(x))
        let (r, b) = self.prune(rs[u], mid, hi, alive)
        rs[u] = r
        if let x = b: (u, Some(x)) else: (0, None)

    fn first_from(self, u: Int, lo: Int, hi: Int, k: Int) -> ?Int:
        if u == 0 or hi <= k: return None
        if lo + 1 == hi: return Some(lo)
        let mid = (lo + hi) / 2
        self.first_from(ls[u], lo, mid, k) ?? self.first_from(rs[u], mid, hi, k)

    /// The set holding only `x`.
    ///
    /// cost: \(O(\log \lvert D \rvert)\) time and new nodes
    /// traps: never
    /// example: `var p = MergeSegs<0..10>(); var s = p.singleton(3); p.first_alive(mut s, x => true) == Some(3)`
    pub fn singleton(mut self, x: D) -> MergeSeg<D> = MergeSeg(self.build(x.ord, 0, D.count))

    /// The union of `a` and `b`. Both stay valid: the result shares their nodes and builds new ones only
    /// where both trees have a node.
    ///
    /// cost: \(O(\log \lvert D \rvert)\) amortized per position, over a sequence of merges whose
    /// results replace their inputs; \(O(\text{overlap})\) for one merge
    /// traps: never
    /// example: `var p = MergeSegs<0..10>(); var m = p.merge(p.singleton(7), p.singleton(3)); p.first_alive(mut m, x => true) == Some(3)`
    pub fn merge(mut self, a: MergeSeg<D>, b: MergeSeg<D>) -> MergeSeg<D> = MergeSeg(self.cat(a.root, b.root, 0, D.count))

    /// The smallest position `x` in `s` with `alive(x)`, or `None`.
    ///
    /// Every position found dead on the
    /// way is removed for good, from `s` and from every set sharing its nodes, so `alive` must be
    /// monotone over time: a position never comes back to life.
    ///
    /// cost: \(O(\log \lvert D \rvert)\) amortized
    /// ties: `var p = MergeSegs<0..10>(); var m = p.merge(p.singleton(7), p.singleton(3)); p.first_alive(mut m, x => true) == Some(3)`
    /// traps: never
    /// example: `var p = MergeSegs<0..10>(); var m = p.merge(p.singleton(3), p.singleton(7)); p.first_alive(mut m, x => x > 5) == Some(7)`
    /// example: `var p = MergeSegs<0..10>(); var m = p.merge(p.singleton(3), p.singleton(7)); p.first_alive(mut m, x => x > 5); p.first_alive(mut m, x => true) == Some(7)`
    pub fn first_alive(mut self, mut s: MergeSeg<D>, alive: D -> Bool) -> ?D:
        let (root, x) = self.prune(s.root, 0, D.count, alive)
        s.root = root
        x

    /// The smallest position of `s` at or after `x`, or `None`. Read-only: `s` and every set sharing
    /// its nodes stay as they are.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// ties: `var p = MergeSegs<0..10>(); let m = p.merge(p.singleton(7), p.singleton(3)); p.first_ge(m, 2) == Some(3)`
    /// traps: never
    /// example: `var p = MergeSegs<0..10>(); let m = p.merge(p.singleton(3), p.singleton(7)); p.first_ge(m, 4) == Some(7)`
    /// example: `var p = MergeSegs<0..10>(); let m = p.merge(p.singleton(3), p.singleton(7)); p.first_ge(m, 8) == None`
    pub fn first_ge(self, s: MergeSeg<D>, x: D) -> ?D:
        if let k = self.first_from(s.root, 0, D.count, x.ord): Some(D.nth(k)) else: None

    /// Whether `s` holds a position in `r`. Read-only, as `first_ge` is.
    /// An empty `s` or `r` gives `false`.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// empty: `var p = MergeSegs<0..10>(); let m = p.singleton(3); not p.any_in(m, 3..3)`
    /// traps: never
    /// example: `var p = MergeSegs<0..10>(); let m = p.merge(p.singleton(3), p.singleton(7)); p.any_in(m, 4..8) and not p.any_in(m, 4..7)`
    pub fn any_in(self, s: MergeSeg<D>, r: Range<D>) -> Bool:
        let (lo, hi) = r.ends
        if let k = self.first_from(s.root, 0, D.count, lo): k < hi else: false

/// One tree of values in a `MergeTrees<D, Z>` pool: a handle, so copying it is O(1).
/// `MergeTree<D>()` holds `Z.zero` at every position.
pub struct MergeTree<D>:
    var root: Int = 0

// The largest `MergeTrees` node index, and the `r` that marks a chain node.
const MERGE_CHAIN = 2_147_483_647

// A node's index in its `MergeTrees` pool, stored narrow.
type MergeAt = 0..=MERGE_CHAIN

// A `MergeTrees` node: its children (0 for the empty tree), its subtree's sum, and the
// multiply tag still owed to its children. A chain node (`r == MERGE_CHAIN`) stands for a subtree
// with one value, `sm` at position `l`, and has no children until a `set` or `merge` reaches below
// it, so a `set` into an empty tree costs one node, as a lazily pushed-down pool would.

struct MergeNode<Z>:
    var l: MergeAt = 0
    var r: MergeAt = 0
    var sm: Z
    var tag: Z

/// A pool of sparse segment trees over `D` holding values `Z`, each with a whole-tree multiply tag.
/// `merge` combines two trees position by position through a hook that sees both values and both
/// trees' sums left of the position, the segment-tree merge that DP over a tree uses for a max- or
/// min-convolution. A tree costs nodes only where it has been `set`, and a position set alone in a
/// subtree costs one node until a `set` or `merge` reaches below it; the nodes live in one list,
/// node 0 being the shared all-zero tree.
pub struct MergeTrees<D, Z: Add + Mul>:
    _nodes: List<MergeNode<Z>>

    /// An empty pool.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `domain D = 0..8; let p = MergeTrees<D, Int>(); p.total(MergeTree<D>()) == 0`
    pub fn new() -> MergeTrees<D, Z> where Z: Zero + One = MergeTrees([MergeNode(sm: Z.zero, tag: Z.one)])

    /// Preallocate the nodes for `sets` more calls to `set`, so they do not regrow the pool:
    /// a `set` and the `merge`s that later split its position off add at most
    /// \(\lceil \log_2 \lvert D \rvert \rceil + 1\) nodes for it, and `scale` and the queries add
    /// none. Changes no tree.
    ///
    /// cost: \(O(\mathit{sets} \log \lvert D \rvert)\)
    /// traps: when `sets < 0` or `sets >= 2^31`
    /// example: `domain D = 0..8; var p = MergeTrees<D, Int>(); p.reserve(sets: 2); var t = MergeTree<D>(); p.set(mut t, D(3), 5); p.get(t, D(3)) == 5`
    pub fn reserve(mut self, sets: Len):
        assert sets >= 0, "reserve requires sets >= 0"
        var depth = 1
        while 1 << (depth - 1) < D.count: depth += 1
        _nodes.reserve(_nodes.len + sets * depth)

    fn scale_node(mut self, u: Int, k: Z):
        if u == 0: return
        _nodes[u].sm *= k
        _nodes[u].tag *= k

    fn down(mut self, u: Int) where Z: One:
        let k = _nodes[u].tag
        self.scale_node(_nodes[u].l, k)
        self.scale_node(_nodes[u].r, k)
        _nodes[u].tag = Z.one

    fn chain(mut self, k: Int, v: Z) -> Int where Z: One:
        _nodes.push(MergeNode(l: k, r: MERGE_CHAIN, sm: v, tag: Z.one))
        _nodes.len - 1

    // Gives the chain node c over lo..hi one child, the chain one level down.
    fn split(mut self, c: Int, lo: Int, hi: Int) where Z: One:
        let k = _nodes[c].l
        let w = self.chain(k, _nodes[c].sm)
        (_nodes[c].l, _nodes[c].r) = if k < (lo + hi) / 2: (w, 0) else: (0, w)
        _nodes[c].tag = Z.one

    fn put(mut self, u: Int, lo: Int, hi: Int, k: Int, v: Z) -> Int:
        if u == 0: return self.chain(k, v)
        if lo + 1 == hi:
            _nodes[u].sm = v
            return u
        if _nodes[u].r == MERGE_CHAIN:
            if _nodes[u].l == k:
                _nodes[u].sm = v
                return u
            self.split(u, lo, hi)
        else:
            self.down(u)
        let mid = (lo + hi) / 2
        if k < mid:
            let l = self.put(_nodes[u].l, lo, mid, k, v)
            _nodes[u].l = l
        else:
            let r = self.put(_nodes[u].r, mid, hi, k, v)
            _nodes[u].r = r
        _nodes[u].sm = _nodes[_nodes[u].l].sm + _nodes[_nodes[u].r].sm
        u

    // The sum over the positions before b, walking one root-to-leaf path.
    fn prefix(self, root: Int, b: Int) -> Z where Z: One + Zero:
        var (u, lo, hi, k, s) = (root, 0, D.count, Z.one, Z.zero)
        while u != 0 and lo < b:
            if hi <= b or _nodes[u].r == MERGE_CHAIN and _nodes[u].l < b:
                return s + _nodes[u].sm * k
            if _nodes[u].r == MERGE_CHAIN: return s
            let mid = (lo + hi) / 2
            k = _nodes[u].tag * k
            if b <= mid:
                (u, hi) = (_nodes[u].l, mid)
            else:
                s += _nodes[_nodes[u].l].sm * k
                (u, lo) = (_nodes[u].r, mid)
        s

    fn part(self, u: Int, lo: Int, hi: Int, a: Int, b: Int) -> Z where Z: Zero:
        if u == 0 or b <= lo or hi <= a: return Z.zero
        if a <= lo and hi <= b: return _nodes[u].sm
        if _nodes[u].r == MERGE_CHAIN:
            return if a <= _nodes[u].l and _nodes[u].l < b: _nodes[u].sm else: Z.zero
        let mid = (lo + hi) / 2
        (
            self.part(_nodes[u].l, lo, mid, a, b) + self.part(_nodes[u].r, mid, hi, a, b)
        ) * _nodes[u].tag

    fn join(
        mut self,
        a: Int,
        b: Int,
        lo: Int,
        hi: Int,
        pa: Z,
        pb: Z,
        leaf: (Z, Z, Z, Z) -> Z,
    ) -> Int where Z: One + Zero:
        if b == 0:
            self.scale_node(a, leaf(Z.one, Z.zero, pa, pb))
            return a
        if a == 0:
            self.scale_node(b, leaf(Z.zero, Z.one, pa, pb))
            return b
        if lo + 1 == hi:
            _nodes[a].sm = leaf(_nodes[a].sm, _nodes[b].sm, pa, pb)
            return a
        let (ca, cb) = (_nodes[a].r == MERGE_CHAIN, _nodes[b].r == MERGE_CHAIN)
        if ca and cb and _nodes[a].l == _nodes[b].l:
            _nodes[a].sm = leaf(_nodes[a].sm, _nodes[b].sm, pa, pb)
            return a
        if ca: self.split(a, lo, hi) else: self.down(a)
        if cb: self.split(b, lo, hi) else: self.down(b)
        let mid = (lo + hi) / 2
        let la = _nodes[_nodes[a].l].sm
        let lb = _nodes[_nodes[b].l].sm
        let l = self.join(_nodes[a].l, _nodes[b].l, lo, mid, pa, pb, leaf)
        let r = self.join(_nodes[a].r, _nodes[b].r, mid, hi, pa + la, pb + lb, leaf)
        _nodes[a].l = l
        _nodes[a].r = r
        _nodes[a].sm = _nodes[l].sm + _nodes[r].sm
        a

    /// Sets position `x` of `t` to `v`.
    ///
    /// cost: \(O(\log \lvert D \rvert)\) time and new nodes
    /// traps: never
    /// example: `domain D = 0..8; var p = MergeTrees<D, Int>(); var t = MergeTree<D>(); p.set(mut t, D(3), 5); p.set(mut t, D(6), 2); p.total(t) == 7`
    pub fn set(mut self, mut t: MergeTree<D>, x: D, v: Z):
        t.root = self.put(t.root, 0, D.count, x.ord, v)

    /// The sum of `t` over the positions in `r`.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// empty: `domain D = 0..8; var p = MergeTrees<D, Int>(); var t = MergeTree<D>(); p.set(mut t, D(3), 5); p.sum(t, D(3)..D(3)) == 0`
    /// traps: never
    /// example: `domain D = 0..8; var p = MergeTrees<D, Int>(); var t = MergeTree<D>(); p.set(mut t, D(3), 5); p.set(mut t, D(6), 2); p.sum(t, D(0)..D(4)) == 5 and p.sum(t, D(4)..D(4)) == 0`
    pub fn sum(self, t: MergeTree<D>, r: Range<D>) -> Z:
        let (lo, hi) = r.ends
        if lo == 0: self.prefix(t.root, hi) else: self.part(t.root, 0, D.count, lo, hi)

    /// The value of `t` at `x`.
    ///
    /// cost: \(O(\log \lvert D \rvert)\)
    /// traps: never
    /// example: `domain D = 0..8; var p = MergeTrees<D, Int>(); var t = MergeTree<D>(); p.set(mut t, D(3), 5); p.get(t, D(3)) == 5 and p.get(t, D(4)) == 0`
    pub fn get(self, t: MergeTree<D>, x: D) -> Z = self.part(t.root, 0, D.count, x.ord, x.ord + 1)

    /// The sum of every value of `t`.
    ///
    /// cost: \(O(1)\)
    /// empty: `domain D = 0..8; let p = MergeTrees<D, Int>(); p.total(MergeTree<D>()) == 0`
    /// traps: never
    /// example: `domain D = 0..8; let p = MergeTrees<D, Int>(); p.total(MergeTree<D>()) == 0`
    pub fn total(self, t: MergeTree<D>) -> Z = _nodes[t.root].sm

    /// Multiplies every value of `t` by `k`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `domain D = 0..8; var p = MergeTrees<D, Int>(); var t = MergeTree<D>(); p.set(mut t, D(3), 5); p.scale(t, 3); p.get(t, D(3)) == 15`
    pub fn scale(mut self, t: MergeTree<D>, k: Z) = self.scale_node(t.root, k)

    /// The tree holding `leaf(a[x], b[x], pre_a, pre_b)` at each position `x`; it uses up `a` and `b`.
    ///
    /// `pre_a` and `pre_b` are the sums of `a` and `b` over the positions before `x`, plus the
    /// `pre_a:` and `pre_b:` arguments (0 by default). The result owns the nodes of both inputs.
    ///
    /// A subtree only `a` reaches is scaled by `leaf(Z.one, Z.zero, pre_a, pre_b)` at its first
    /// position, and one only `b` reaches by `leaf(Z.zero, Z.one, pre_a, pre_b)`. That is exact when
    /// `leaf(v, 0, pa, pb)` is `v` times a factor free of `pa`, and `leaf(0, v, pa, pb)` is `v` times
    /// a factor free of `pb`, as for `(va, vb, pa, pb) => va * (pb + vb) + vb * pa` (a max-convolution:
    /// the result at `x` counts pairs whose larger position is `x`).
    ///
    /// cost: \(O(\log \lvert D \rvert)\) amortized per `set`, over a sequence of merges whose results
    /// replace their inputs; \(O(\text{overlap})\) for one merge
    /// traps: never
    /// example: `domain D = 0..4; var p = MergeTrees<D, Int>(); var a = MergeTree<D>(); var b = MergeTree<D>(); p.set(mut a, D(1), 2); p.set(mut b, D(2), 3); let m = p.merge(a, b, (va, vb, pa, pb) => va * (pb + vb) + vb * pa); p.get(m, D(1)) == 0 and p.get(m, D(2)) == 6`
    pub fn merge(
        mut self,
        a: MergeTree<D>,
        b: MergeTree<D>,
        leaf: (Z, Z, Z, Z) -> Z,
        pre_a: Z = Z.zero,
        pre_b: Z = Z.zero,
    ) -> MergeTree<D> =
        MergeTree(self.join(a.root, b.root, 0, D.count, pre_a, pre_b, leaf))

impl Show for MergeSeg<D>:
    /// The handle's root node in its pool, `#0` for the empty set.
    fn show(s) = "#{s.root}"

impl Show for MergeSegs<D>:
    /// The pool's node count, past the shared empty node.
    fn show(p) = "{p.ls.len - 1} nodes"

impl Show for MergeTree<D>:
    /// The handle's root node in its pool, `#0` for the all-zero tree.
    fn show(t) = "#{t.root}"

impl Show for MergeTrees<D, Z>:
    /// The pool's node count, past the shared empty node.
    fn show(p) = "{p._nodes.len - 1} nodes"

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

impl Show for SegTree<D, T: Show, F>:
    /// Every position's value, with tags applied, space-separated; a sparse tree (built with `over:`) shows its size and fold.
    ///
    /// example: `var s = SegTree(+, [4, 5, 6]); s.set(1, 9); "{s}" == "4 9 6"`
    /// example: `var s = SegTree<Int, Int>(+, over: 0..1_000_000); s.set(7, 4); "{s}" == "1000000 positions, all 4"`
    fn show(s):
        if s.lc.len > 0: return "{s.count} positions, all {s.all}"
        var out = ""
        for i in 0..D.count():
            if not out.is_empty: out += " "
            out += "{s.get(D.at(i))}"
        out

impl Show for MergeSortTree<D, T: Show>:
    /// The values it was built from, space-separated in position order.
    ///
    /// example: `"{MergeSortTree([3, 1, 4])}" == "3 1 4"`
    fn show(t):
        var out = ""
        for i in 0..D.count:
            if not out.is_empty: out += " "
            out += "{t.data[t.beg[t.size + i]]}"
        out

impl Show for SegTree2D<R, C, T>:
    /// Every cell's value, space-separated within a row, rows separated by ` / `.
    ///
    /// example: `var g = SegTree2D<0..2, 0..2, Int>(+, fill: 1); g.set(1, 0, 7); "{g}" == "1 1 / 7 1"`
    fn show(g):
        var out = ""
        for r in 0..R.count:
            if r > 0: out += " / "
            for c in 0..C.count:
                if c > 0: out += " "
                out += "{g.get(R.at(r), C.at(c))}"
        out

impl Show for WaveletMatrix<D>:
    /// The values it was built from, space-separated in position order.
    ///
    /// example: `"{WaveletMatrix([3, 1, 4])}" == "3 1 4"`
    fn show(w):
        var out = ""
        for i in 0..D.count:
            if not out.is_empty: out += " "
            out += "{w.kth(D.at(i)..=D.at(i), 0)!}"
        out

impl Show for DynamicOrderTree<D, T>:
    /// The current values, space-separated in position order.
    ///
    /// example: `var t = DynamicOrderTree([3, 1, 4]); t.set(2, 9); "{t}" == "3 1 9"`
    fn show(t):
        var out = ""
        for i in 0..t.n:
            if not out.is_empty: out += " "
            out += "{t.get(D.at(i))}"
        out

fn affine_onto(mut old: (Int, Int), new: (Int, Int)):
    old = (new.0 * old.0, new.0 * old.1 + new.1)

@test
fn an_in_place_compose_matches_a_plain_array():
    let n = 13
    var a = [rand(-5, 5) for _ in 0..n]
    let act = Action<Int, (Int, Int)>(
        id: (1, 0),
        compose: (nf, of) => (nf.0 * of.0, nf.0 * of.1 + nf.1),
        apply: (f, v, span) => f.0 * v + f.1 * span,
        compose_in: (mut o, nf) => affine_onto(mut o, nf),
    )
    var st = SegTree(+, a, act:)
    for _ in 0..300:
        let l = rand(0, n - 1)
        let r = rand(l + 1, n)
        let f = (if rand(0, 1) == 0: -1 else: 1, rand(-3, 3))
        st.apply(l..r, f)
        for i in l..r: a[i] = f.0 * a[i] + f.1
        let ql = rand(0, n - 1)
        let qr = rand(ql + 1, n)
        assert st.fold(ql..qr) == sum(a[i] for i in ql..qr)
        assert st.get(ql) == a[ql]
    assert st.all == a.sum()

