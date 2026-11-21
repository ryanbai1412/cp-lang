//! Disjoint sets over a domain, with optional values per set, parities, and rollback.

/// A point in the history of a structure that can step back in time, returned by `mark()`
/// and taken by `undo_to`. Its own type, so it can't be mixed up with a count or an index.
pub struct Mark(at: 0..=2_000_000_000)

/// Union-find with path halving and union by size.
/// `find`, `unite`, `same` and `size` are O(alpha(n)) amortized; memory is one `[V]` array (4 B/vertex).
///
/// It can also step back in time: `let snap = d.mark()` remembers the present and
/// `d.undo_to(snap)` returns to it. From the first
/// `mark` on, paths are no longer shortened (a shortened path cannot be restored), so every
/// operation is O(log n) and each `unite` costs one entry of memory until it is undone.
pub struct Dsu<V>:
    p: List<-2_147_483_648..=2_147_483_647>
    parts: Int
    /// Pairs `[child_ord, old_neg_size]` pushed flat since the first `mark`.
    joined: List<Int>
    marked: Bool

    /// Every position starts in its own set; pass `rollback: true` to enable rollback up front (or call `mark()` lazily). O(n).
    pub fn new(rollback: Bool = false) -> Dsu<V>:
        Dsu(List.filled(V.K, -1), V.K, [], rollback)

    fn find_ord(mut self, start: Int) -> Int:
        var x = start
        unchecked:
            while p[x] >= 0:
                let par = p[x]
                if not marked and p[par] >= 0:
                    p[x] = p[par]
                x = par
        x

    /// The representative of `v`'s set.
    pub fn find(mut self, v: V) -> V = V.at(self.find_ord(v.ord))

    /// The present, to return to with `undo_to`. O(1).
    pub fn mark(mut self) -> Mark:
        marked = true
        Mark(joined.len)

    /// Undo every `unite` made since `snap = mark()`. O(1) per undone `unite`.
    pub fn undo_to(mut self, snap: Mark):
        var i = joined.len
        unchecked:
            while i > snap.at:
                i -= 2
                let y = joined[i]
                let neg_sy = joined[i + 1]
                let r = p[y]
                p[r] -= neg_sy
                p[y] = neg_sy
                parts += 1
        joined.truncate(snap.at)

    /// Merge the sets of `a` and `b`. Returns false if they were already one set.
    pub fn unite(mut self, a: V, b: V) -> Bool:
        var x = self.find_ord(a.ord)
        var y = self.find_ord(b.ord)
        if x == y: return false
        unchecked:
            if p[x] > p[y]: swap(mut x, mut y)
            let neg_sy = p[y]
            p[x] += neg_sy
            p[y] = x
            parts -= 1
            if marked:
                joined.push(y)
                joined.push(neg_sy)
        true

    /// As `unite`, but says which root stayed and which went under it, so data kept per
    /// set can follow: `if let (kept, merged) = d.merge(a, b): sum[kept] += sum[merged]`.
    /// `none` if `a` and `b` were already one set.
    pub fn merge(mut self, a: V, b: V) -> ?(kept: V, merged: V):
        var x = self.find_ord(a.ord)
        var y = self.find_ord(b.ord)
        if x == y: return none
        unchecked:
            if p[x] > p[y]: swap(mut x, mut y)
            let neg_sy = p[y]
            p[x] += neg_sy
            p[y] = x
            parts -= 1
            if marked:
                joined.push(y)
                joined.push(neg_sy)
        some((kept: V.at(x), merged: V.at(y)))

    /// Whether `a` and `b` are in the same component. O(alpha(n)) amortized.
    pub fn same(mut self, a: V, b: V) -> Bool = self.find_ord(a.ord) == self.find_ord(b.ord)

    /// The number of positions in `v`'s set.
    pub fn size(mut self, v: V) -> Int:
        let r = self.find_ord(v.ord)
        unchecked:
            return -p[r]

    /// The number of sets.
    pub @get fn components(self) -> Int = parts

    /// Yield the representative root of each connected component. O(n), zero allocations.
    pub fn roots(self) -> yields V:
        for v in V:
            unchecked:
                if p[v.ord] < 0: yield v

    /// Every connected component as a list of its vertices, each vertex appearing once. O(n alpha(n)).
    pub fn groups(mut self) -> List<List<V>>:
        var gid: [V]Int = -1
        var out: List<List<V>> = []
        for v in V:
            unchecked:
                if p[v.ord] < 0:
                    gid[v] = out.len
                    out.push([])
        for v in V:
            out[gid[self.find(v)]].push(v)
        out

/// Union-find where every set carries a value of type `T`. When two sets merge, the
/// merged set gets `join(value of a's set, value of b's set)` for `unite(a, b)`, whichever
/// root is kept, so a directional join like `(_, b) => b` means "b's value wins".
/// Costs are those of `Dsu`, plus one `join` per merge; memory is three `[V]` arrays.
///
/// `mark` and `undo_to` step back in time like `Dsu`'s, and restore the values too: every
/// merge and every `set` since the mark is undone, O(1) each.
pub struct DsuOf<V, T>:
    up: [V]V
    sz: [V]1..=V.count
    val: [V]T
    join: (T, T) -> T
    parts: 0..=2_000_000_000
    /// Since the first `mark`, one entry per change: the root that went under another (the
    /// root itself for a `set`), the root whose value changed, and that value before.
    log: List<(V, V, T)>
    marked: Bool

    /// Every position starts in its own set, with value `init(v)`. O(n).
    pub fn new(init: (V) -> T, join: (T, T) -> T) -> DsuOf<V, T>:
        DsuOf([V] v => v, [V] v => 1, [V] v => init(v), join, V.K, [], false)

    /// The representative of `v`'s set.
    pub fn find(mut self, v: V) -> V:
        var x = v
        while up[x] != x:
            if not marked: up[x] = up[up[x]]
            x = up[x]
        x

    /// The value of `v`'s set.
    pub fn get(mut self, v: V) -> T:
        let r = self.find(v)
        val[r]

    /// Replace the value of `v`'s set.
    pub fn set(mut self, v: V, x: T):
        let r = self.find(v)
        if marked: log.push((r, r, val[r]))
        val[r] = x

    /// Merge the sets of `a` and `b`, joining their values. Returns false if they were
    /// already one set (the value is left alone then).
    pub fn unite(mut self, a: V, b: V) -> Bool:
        let ra = self.find(a)
        let rb = self.find(b)
        if ra == rb: return false
        let joined = join(val[ra], val[rb])
        let (x, y) = if sz[ra] >= sz[rb]: (ra, rb) else: (rb, ra)
        if marked: log.push((y, x, val[x]))
        up[y] = x
        sz[x] += sz[y]
        val[x] = joined
        parts -= 1
        true

    /// As `unite`, but says which root stayed and which went under it. `none` if `a` and
    /// `b` were already one set.
    pub fn merge(mut self, a: V, b: V) -> ?(kept: V, merged: V):
        let ra = self.find(a)
        let rb = self.find(b)
        if ra == rb: return none
        let (x, y) = if sz[ra] >= sz[rb]: (ra, rb) else: (rb, ra)
        self.unite(ra, rb)
        some((kept: x, merged: y))

    /// Whether `a` and `b` are in the same component. O(alpha(n)) amortized.
    pub fn same(mut self, a: V, b: V) -> Bool = self.find(a) == self.find(b)

    /// The number of positions in `v`'s set.
    pub fn size(mut self, v: V) -> Int:
        let r = self.find(v)
        sz[r]

    /// The number of sets.
    pub @get fn components(self) -> Int = parts

    /// Yield the representative root of each connected component. O(n), zero allocations.
    pub fn roots(self) -> yields V:
        for v in V:
            if up[v] == v: yield v

    /// Yield each connected component's payload once (one per root). O(n), zero allocations.
    pub fn values(self) -> yields T:
        for v in V:
            if up[v] == v: yield val[v]

    /// The present, to return to with `undo_to`. O(1).
    pub fn mark(mut self) -> Mark:
        marked = true
        Mark(log.len)

    /// Undo every merge and `set` made since `snap = mark()`. O(1) each.
    pub fn undo_to(mut self, snap: Mark):
        while log.len > snap.at:
            let (y, x, old) = log.pop()!
            if y != x:
                sz[x] -= sz[y]
                up[y] = y
                parts += 1
            val[x] = old

/// Union-find that knows, for two positions of one set, whether they are on the same side
/// or on opposite sides: two-colouring, parity constraints, `x[a] xor x[b] == d`.
/// `unite` and `diff` are O(log n) (union by size; paths are never shortened, so `mark` and
/// `undo_to` work at any time, O(1) per undone `unite`). Memory is three `[V]` arrays.
pub struct ParityDsu<V>:
    up: [V]V
    sz: [V]1..=V.count
    /// Is the position on the other side from `up` of it?
    flip: [V]Bool
    /// One entry per `unite` that changed something: the root it hung under another root,
    /// or (with `false`) the root of a set where it found a contradiction.
    log: List<(V, Bool)>
    conflicts: 0..=2_000_000_000

    /// Every position starts alone. O(n).
    pub fn new() -> ParityDsu<V>:
        ParityDsu([V] v => v, [V] v => 1, [V] v => false, [], 0)

    /// The representative of `v`'s set and whether `v` is on the other side from it.
    pub fn find(self, v: V) -> (root: V, flip: Bool):
        var (x, side) = (v, false)
        while up[x] != x:
            side = side != flip[x]
            x = up[x]
        (root: x, flip: side)

    /// Record that `a` and `b` are on opposite sides (`differ: false`: the same side).
    /// `false` if that contradicts what is already known; the contradiction is counted, so
    /// `consistent` turns false until it is undone. An edge of a graph is `unite(u, v)`.
    pub fn unite(mut self, a: V, b: V, differ: Bool = true) -> Bool:
        let (ra, sa) = self.find(a)
        let (rb, sb) = self.find(b)
        if ra == rb:
            if (sa != sb) == differ: return true
            conflicts += 1
            log.push((ra, false))
            return false
        let (big, small) = if sz[ra] >= sz[rb]: (ra, rb) else: (rb, ra)
        up[small] = big
        flip[small] = (sa != sb) != differ
        sz[big] += sz[small]
        log.push((small, true))
        true

    /// Are `a` and `b` on opposite sides? `none` when nothing relates them yet.
    pub fn diff(self, a: V, b: V) -> ?Bool:
        let (ra, sa) = self.find(a)
        let (rb, sb) = self.find(b)
        if ra != rb: none else: sa != sb

    /// Whether `a` and `b` are in the same component. O(log n).
    pub fn same(self, a: V, b: V) -> Bool = self.find(a).root == self.find(b).root

    /// The number of positions in `v`'s set.
    pub fn size(self, v: V) -> Int = sz[self.find(v).root]

    /// Has every `unite` so far agreed with the ones before it? For a graph: is it
    /// two-colourable?
    pub @get fn consistent(self) -> Bool = conflicts == 0

    /// The present, to return to with `undo_to`. O(1).
    pub fn mark(self) -> Mark = Mark(log.len)

    /// Take back every `unite` since `snap = mark()`, contradictions included.
    pub fn undo_to(mut self, snap: Mark):
        while log.len > snap.at:
            let (y, joined) = log.pop()!
            if not joined:
                conflicts -= 1
            else:
                sz[up[y]] -= sz[y]
                up[y] = y
                flip[y] = false

/// Weighted (potential / difference-constraint) union-find over domain `V` with `Mark` rollback.
/// Maintains constraints of the form `val[b] - val[a] == w`. All operations are O(log n);
/// `mark` and `consistent` are O(1); `undo_to` is O(1) per undone step.
pub struct PotentialDsu<V>:
    up: [V]V
    sz: [V]1..=V.count
    pot: [V]Int
    parts: 0..=2_000_000_000
    conflicts: 0..=2_000_000_000
    log: List<(V, Bool)>

    /// Every position starts in its own set with potential 0, optionally combined by `op:` (default `+`). O(n).
    pub fn new(op: ?((Int, Int) -> Int) = none) -> PotentialDsu<V>:
        PotentialDsu([V] v => v, [V] _ => 1, [V] _ => 0, V.K, 0, [])

    /// Root of `v`'s set and `val[v] - val[root]`. O(log n).
    pub fn find(self, v: V) -> (root: V, offset: Int):
        var x = v
        var d = 0
        while up[x] != x:
            d += pot[x]
            x = up[x]
        (root: x, offset: d)

    /// Enforce `val[b] - val[a] == w`. Returns `false` and increments conflicts on contradiction. O(log n).
    pub fn unite(mut self, a: V, b: V, w: Int) -> Bool:
        let (ra, da) = self.find(a)
        let (rb, db) = self.find(b)
        if ra == rb:
            if db - da == w: return true
            conflicts += 1
            log.push((ra, false))
            return false
        if sz[ra] >= sz[rb]:
            up[rb] = ra
            pot[rb] = da + w - db
            sz[ra] += sz[rb]
            parts -= 1
            log.push((rb, true))
        else:
            up[ra] = rb
            pot[ra] = db - w - da
            sz[rb] += sz[ra]
            parts -= 1
            log.push((ra, true))
        true

    /// `val[b] - val[a]` if `a` and `b` are in the same component, else `none`. O(log n).
    pub fn diff(self, a: V, b: V) -> ?Int:
        let (ra, da) = self.find(a)
        let (rb, db) = self.find(b)
        if ra != rb: none else: db - da

    /// Whether `a` and `b` are in the same component. O(log n).
    pub fn same(self, a: V, b: V) -> Bool = self.find(a).root == self.find(b).root

    /// Number of vertices in `v`'s component. O(log n).
    pub fn size(self, v: V) -> Int = sz[self.find(v).root]

    /// Number of connected components. O(1).
    pub @get fn components(self) -> Int = parts

    /// Whether no contradicting constraints are currently active. O(1).
    pub @get fn consistent(self) -> Bool = conflicts == 0

    /// Snapshot the current state for `undo_to`. O(1).
    pub fn mark(self) -> Mark = Mark(log.len)

    /// Revert every `unite` since `snap = mark()`. O(1) per undone step.
    pub fn undo_to(mut self, snap: Mark):
        while log.len > snap.at:
            let (y, joined) = log.pop()!
            if not joined:
                conflicts -= 1
            else:
                sz[up[y]] -= sz[y]
                up[y] = y
                pot[y] = 0
                parts += 1

/// Domain-indexed array `[D]T` with `Mark` rollback.
/// `get`, `set`, and `mark` are O(1); `undo_to` is O(1) per undone write.
pub struct RollbackArray<D, T>:
    data: [D]T
    log: List<(D, T)>

    /// Build a rollback array initialized by `init(i)` for each `i` in `D`. O(D.K).
    pub fn new(init: D -> T) -> RollbackArray<D, T> = RollbackArray([D] i => init(i), [])

    /// Read the current value at `i`. O(1).
    pub fn get(self, i: D) -> T = data[i]

    /// Set `i` to `val`, recording the previous value for rollback. O(1).
    pub fn set(mut self, i: D, val: T):
        log.push((i, data[i]))
        data[i] = val

    /// Snapshot the current state for `undo_to`. O(1).
    pub fn mark(self) -> Mark = Mark(log.len)

    /// Revert every `set` since `snap = mark()`. O(1) per undone write.
    pub fn undo_to(mut self, snap: Mark):
        while log.len > snap.at:
            let (i, old) = log.pop()!
            data[i] = old

/// Group-valued potential / weighted DSU over domain `V` using operation values `(op, inv, id)` (W10).
/// Maintains `pot(b) = op(pot(a), w)` with O(alpha(n)) amortized ops (O(log n) after `mark()`).
pub struct GroupPotentialDsu<V, G>:
    _parts: Int

    /// Create an empty `GroupPotentialDsu<V, G>` for group `(op, inv, id)`. O(n).
    pub fn new(op: (G, G) -> G, inv: G -> G, id: G) -> GroupPotentialDsu<V, G> = todo

    /// Find `(root, pot)` where `pot` is the group offset from `root` to `v`. O(alpha(n)) amortized, O(log n) after `mark`.
    pub fn find(mut self, v: V) -> (root: V, pot: G) = todo

    /// Assert `pot(b) = op(pot(a), w)`. Returns false if inconsistent or already connected. O(alpha(n)) amortized.
    pub fn unite(mut self, a: V, b: V, w: G) -> Bool = todo

    /// Whether `a` and `b` are in the same component. O(alpha(n)) amortized.
    pub fn same(mut self, a: V, b: V) -> Bool = todo

    /// Relative potential from `a` to `b`, or `none` if disconnected. O(alpha(n)) amortized.
    pub fn diff(mut self, a: V, b: V) -> ?G = todo

    /// Whether no contradictory `unite` has occurred. O(1).
    pub @get fn consistent(self) -> Bool = todo

    /// Current number of connected components. O(1).
    pub @get fn components(self) -> Int = todo

    /// Size of `v`'s component. O(alpha(n)) amortized.
    pub fn size(mut self, v: V) -> Int = todo

    /// Snapshot the current state for `undo_to`. O(1).
    pub fn mark(mut self) -> Mark = todo

    /// Revert every `unite` since `snap = mark()`. O(1) per undone unite.
    pub fn undo_to(mut self, snap: Mark) = todo

/// Path-copying persistent disjoint-set union over domain `V`.
/// Every version remains readable; `find`, `unite`, `same`, and `size` run in O(log^2 n).
pub struct PersistentDsu<V>:
    _parts: Int

    /// Create version 0 with every vertex in its own component. O(n).
    pub fn new() -> PersistentDsu<V> = todo

    /// The initial version handle (version 0). O(1).
    pub @get fn base(self) -> Int = todo

    /// Representative of `v` in version `ver`. O(log^2 n).
    pub fn find(self, ver: Int, v: V) -> V = todo

    /// Whether `a` and `b` are connected in version `ver`. O(log^2 n).
    pub fn same(self, ver: Int, a: V, b: V) -> Bool = todo

    /// Size of `v`'s component in version `ver`. O(log^2 n).
    pub fn size(self, ver: Int, v: V) -> Int = todo

    /// Return a new version handle equal to `ver` with `a` and `b` united. O(log^2 n) time and O(log n) memory.
    pub fn unite(mut self, ver: Int, a: V, b: V) -> Int = todo

    /// Preallocate pool capacity for at least `nodes` persistent array nodes. O(nodes).
    pub fn reserve(mut self, nodes: Int) = todo

@test
fn potential_dsu_and_rollback_array():
    domain V = 0..6
    var d = PotentialDsu<V>()
    assert d.components == 6 and d.consistent
    assert d.unite(V(0), V(1), 5)
    assert d.unite(V(1), V(2), -2)
    assert d.diff(V(0), V(2)) == 3 and d.diff(V(2), V(0)) == -3
    assert d.diff(V(0), V(3)) == none
    let snap = d.mark()
    assert not d.unite(V(0), V(2), 4)
    assert not d.consistent
    assert d.unite(V(2), V(3), 10)
    assert d.diff(V(0), V(3)) == 13 and d.components == 3
    d.undo_to(snap)
    assert d.consistent and d.components == 4 and d.diff(V(0), V(3)) == none
    assert d.diff(V(0), V(2)) == 3

    var ra = RollbackArray<V, Int>(v => v.to_idx)
    let m0 = ra.mark()
    ra.set(V(1), 99)
    ra.set(V(1), 42)
    assert ra.get(V(1)) == 42 and ra.get(V(2)) == 2
    ra.undo_to(m0)
    assert ra.get(V(1)) == 1

@test
fn unite_and_find():
    domain V = 1..=6
    var d = Dsu<V>()
    assert d.components == 6
    assert d.unite(V(1), V(2))
    assert d.unite(V(3), V(4))
    assert not d.unite(V(2), V(1))
    assert d.same(V(1), V(2)) and not d.same(V(1), V(3))
    assert d.unite(V(2), V(4))
    assert d.size(V(3)) == 4 and d.size(V(5)) == 1
    assert d.components == 3

@test
fn agrees_with_labels_on_random_unions():
    domain V = 0..12
    var d = Dsu<V>()
    var label = [V] v => v.to_idx
    for _ in 0..40:
        let a = V.nth(rand(0, 11))
        let b = V.nth(rand(0, 11))
        let merged = label[a] != label[b]
        assert d.unite(a, b) == merged
        let (la, lb) = (label[a], label[b])
        for v in V:
            if label[v] == lb: label[v] = la
        for v in V:
            assert d.same(a, v) == (label[v] == la)

@test
fn undo_to_returns_to_every_mark():
    domain V = 0..12
    var d = Dsu<V>()
    d.unite(V(0), V(1))
    var snaps: List<(Mark, List<V>, Int)> = []
    for round in 0..40:
        if rand(0, 2) > 0 or snaps.is_empty:
            snaps.push((d.mark(), [d.find(v) for v in V], d.components))
            d.unite(V.nth(rand(0, 11)), V.nth(rand(0, 11)))
        else:
            let (snap, roots, parts) = snaps.pop()!
            d.undo_to(snap)
            assert [d.find(v) for v in V] == roots and d.components == parts
            for v in V: assert d.size(v) == count(roots[u.to_idx] == roots[v.to_idx] for u in V)
    d.undo_to(Mark(0))
    assert d.same(V(0), V(1)) and d.components == 11

@test
fn groups_partitions_all_vertices_by_component():
    domain V = 1..=7
    var d = Dsu<V>()
    assert d.groups().len == 7
    assert [g.len for g in d.groups()] == [1, 1, 1, 1, 1, 1, 1]
    d.unite(V(1), V(3))
    d.unite(V(3), V(6))
    let snap = d.mark()
    d.unite(V(2), V(5))
    d.unite(V(5), V(7))
    let gs = d.groups()
    assert gs.len == d.components and gs.len == 3
    var flat: List<V> = []
    for g in gs:
        assert not g.is_empty
        for v in g:
            assert d.same(v, g[0])
            flat.push(v)
        assert g.len == d.size(g[0])
    assert sorted(flat) == [v for v in V]
    d.undo_to(snap)
    assert d.groups().len == 5 and d.components == 5
    for v in V: d.unite(V(1), v)
    assert d.groups() == [[v for v in V]]

@test
fn merge_says_which_root_stayed():
    domain V = 0..6
    var d = Dsu<V>()
    var total = [V] v => v.idx
    for (a, b) in [(0, 1), (2, 3), (1, 3), (0, 2), (4, 5)]:
        if let (kept, merged) = d.merge(V(a), V(b)):
            assert d.find(kept) == kept and d.find(merged) == kept
            total[kept] += total[merged]
        else:
            assert (a, b) == (0, 2)
    assert total[d.find(V(0))] == 6 and total[d.find(V(5))] == 9 and d.components == 2

@test
fn values_follow_merges_and_roll_back():
    domain V = 0..10
    for _ in 0..30:
        var d = DsuOf<V, Int>(v => v.idx, (a, b) => a + b * 100 % 1000)
        var label = [V] v => v.idx
        var value = [V] v => v.idx
        var snaps: List<(Mark, List<Int>, List<Int>)> = []
        for _ in 0..30:
            let roll = rand(0, 3)
            if roll == 0:
                snaps.push((d.mark(), [label[v] for v in V], [value[v] for v in V]))
            elif roll == 1 and not snaps.is_empty:
                let (m, ls, vs) = snaps.pop()!
                d.undo_to(m)
                for v in V:
                    label[v] = ls[v.idx]
                    value[v] = vs[v.idx]
            elif roll == 2:
                let (v, x) = (V.nth(rand(0, 9)), rand(0, 50))
                d.set(v, x)
                for u in V:
                    if label[u] == label[v]: value[u] = x
            else:
                let (a, b) = (V.nth(rand(0, 9)), V.nth(rand(0, 9)))
                let (la, lb) = (label[a], label[b])
                let fresh = la != lb
                let joined = value[a] + value[b] * 100 % 1000
                assert d.unite(a, b) == fresh
                if fresh:
                    for v in V:
                        if label[v] == lb or label[v] == la:
                            label[v] = la
                            value[v] = joined
            for v in V:
                assert d.get(v) == value[v]
                assert d.size(v) == count(label[u] == label[v] for u in V)
            assert d.components == count(label[v] == v.idx for v in V)

@test
fn a_directional_join_keeps_the_second_value():
    domain V = 1..=4
    var d = DsuOf<V, ?V>(v => none, (_, b) => b)
    d.set(V(2), some(V(1)))
    assert d.unite(V(3), V(2)) and d.get(V(3)) == some(V(1))
    let snap = d.mark()
    d.set(V(4), some(V(4)))
    if let (kept, merged) = d.merge(V(4), V(3)):
        assert d.find(merged) == kept
    assert d.get(V(2)) == some(V(1)) and d.components == 2
    d.undo_to(snap)
    assert d.get(V(4)) == none and d.get(V(2)) == some(V(1)) and d.components == 3

@test
fn parity_matches_a_brute_colouring_and_rolls_back():
    domain V = 0..7
    for _ in 0..40:
        var d = ParityDsu<V>()
        var cons: List<(V, V, Bool)> = []
        var snaps: List<(Mark, Int)> = []
        for _ in 0..12:
            if rand(0, 3) == 0 and not snaps.is_empty:
                let (m, k) = snaps.pop()!
                d.undo_to(m)
                cons.truncate(k)
            else:
                if rand(0, 2) == 0: snaps.push((d.mark(), cons.len))
                let c = (V.nth(rand(0, 6)), V.nth(rand(0, 6)), rand(0, 1) == 1)
                let before = d.consistent
                let agreed = d.unite(c.0, c.1, differ: c.2)
                cons.push(c)
                assert not agreed or d.consistent == before
            let fits = [
                mask for mask in 0..128 if all(
                    ((mask >> a.idx) % 2 != (mask >> b.idx) % 2) == x for (a, b, x) in cons
                )
            ]
            assert d.consistent != fits.is_empty
            if d.consistent:
                for a in V:
                    for b in V:
                        let sides = [(f >> a.idx) % 2 != (f >> b.idx) % 2 for f in fits]
                        let want = if all(x == sides[0] for x in sides): some(sides[0]) else: none
                        assert d.diff(a, b) == want

@test
fn roots_and_values_yield_one_entry_per_component():
    domain V = 0..6
    var d = Dsu<V>()
    let _ = d.unite(V(0), V(2))
    let _ = d.unite(V(2), V(4))
    let _ = d.unite(V(1), V(3))
    let rs = [r for r in d.roots()]
    assert rs.len == 3
    for r in rs: assert d.find(r) == r
    var w = DsuOf<V, Int>(v => v.ord + 1, +)
    let _ = w.unite(V(0), V(2))
    let _ = w.unite(V(2), V(4))
    let _ = w.unite(V(1), V(3))
    assert [r for r in w.roots()].len == 3
    var vs = [x for x in w.values()]
    vs.sort()
    assert vs == [6, 6, 9]
