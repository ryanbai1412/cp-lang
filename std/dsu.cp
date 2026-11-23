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
    @cache p: List<-2_147_483_648..=2_147_483_647>
    parts: Int
    /// Pairs `[child_ord, old_neg_size]` pushed flat since the first `mark`.
    pub joined: List<Int>
    marked: Bool

    /// A union-find over `V` with every position in its own set. Pass `rollback: true` to switch
    /// path shortening off from the start, as the first `mark()` would.
    ///
    /// cost: O(n) for n = `V.count`; one `[V]` array
    /// traps: never
    /// example: `let d = Dsu<0..3>(); d.components == 3`
    pub fn new(rollback: Bool = false) -> Dsu<V> = Dsu(
        List.filled(V.count, -1),
        V.count,
        [],
        rollback,
    )

    fn find_ord(self, start: Int) -> Int:
        var x = start
        unchecked:
            while p[x] >= 0:
                let par = p[x]
                if not marked and p[par] >= 0:
                    p[x] = p[par]
                x = par
        x

    /// The representative of `v`'s set: the root that union by size kept. Two positions are in one
    /// set exactly when they have the same representative.
    ///
    /// Halves the path it walks through `@cache`, so a read-only `Dsu` answers it too; from the
    /// first `mark` on, paths are left alone.
    ///
    /// cost: O(alpha(n)) amortized; O(log n) after the first `mark`
    /// ties: None; the representative is fixed by the merges (larger set's root, `a`'s root on equal sizes)
    /// traps: never
    /// example: `var d = Dsu<0..4>(); d.unite(2, 3); d.find(3) == 2 and d.find(0) == 0`
    pub fn find(self, v: V) -> V = V.nth(self.find_ord(v.ord))

    /// The present, to return to with [`undo_to`](Dsu.undo_to). The first call switches path
    /// shortening off for good.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var d = Dsu<0..3>(); let m = d.mark(); d.unite(0, 1); d.undo_to(m); d.components == 3`
    pub fn mark(mut self) -> Mark:
        marked = true
        Mark(joined.len)

    /// Undoes every `unite` made since `snap = mark()`, newest first, so the sets, sizes and
    /// representatives are exactly what they were at the mark. Marks nest: undoing to an older mark
    /// passes over newer ones.
    ///
    /// cost: O(1) per undone `unite`
    /// traps: when `snap` lies after the present (its changes were already undone)
    /// example: `var d = Dsu<0..3>(); d.unite(0, 1); let m = d.mark(); d.unite(1, 2); d.undo_to(m); d.same(0, 1) and not d.same(0, 2)`
    pub fn undo_to(mut self, snap: Mark):
        assert snap.at <= joined.len
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

    /// Merges the sets of `a` and `b`; `false` if they were already one set (nothing changes then).
    ///
    /// The root of the larger set is kept; on equal sizes `a`'s root is kept.
    ///
    /// cost: O(alpha(n)) amortized; O(log n) after the first `mark`, plus one log entry
    /// traps: never
    /// example: `var d = Dsu<0..3>(); d.unite(0, 1) and not d.unite(1, 0)`
    pub fn unite(mut self, a: V, b: V) -> Bool = self.merge(a, b) != None

    /// As [`unite`](Dsu.unite), but says which root stayed and which went under it, so data kept
    /// per set can follow: `if let (kept, merged) = d.merge(a, b): sum[kept] += sum[merged]`.
    /// `None` if `a` and `b` were already one set.
    ///
    /// cost: as `unite`
    /// traps: never
    /// example: `var d = Dsu<0..3>(); d.merge(1, 0) == Some((kept: 1, merged: 0))`
    /// example: `var d = Dsu<0..3>(); d.unite(0, 1); d.merge(2, 1) == Some((kept: 0, merged: 2))`
    /// example: `var d = Dsu<0..3>(); d.unite(0, 1); d.merge(1, 0) == None`
    pub fn merge(mut self, a: V, b: V) -> ?(kept: V, merged: V):
        var x = self.find_ord(a.ord)
        var y = self.find_ord(b.ord)
        if x == y: return None
        unchecked:
            if p[x] > p[y]: swap(mut x, mut y)
            let neg_sy = p[y]
            p[x] += neg_sy
            p[y] = x
            parts -= 1
            if marked:
                joined.push(y)
                joined.push(neg_sy)
        Some((kept: V.nth(x), merged: V.nth(y)))

    /// Whether `a` and `b` are in the same set.
    ///
    /// cost: O(alpha(n)) amortized; O(log n) after the first `mark`
    /// traps: never
    /// example: `var d = Dsu<0..3>(); d.unite(0, 2); d.same(2, 0) and not d.same(0, 1)`
    pub fn same(self, a: V, b: V) -> Bool = self.find_ord(a.ord) == self.find_ord(b.ord)

    /// The number of positions in `v`'s set, `v` included.
    ///
    /// cost: O(alpha(n)) amortized; O(log n) after the first `mark`
    /// traps: never
    /// example: `var d = Dsu<0..3>(); d.unite(0, 2); d.size(2) == 2 and d.size(1) == 1`
    pub fn size(self, v: V) -> Int:
        let r = self.find_ord(v.ord)
        unchecked:
            return -p[r]

    /// The number of sets: `V.count` minus the successful `unite`s still in effect.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var d = Dsu<0..4>(); d.unite(0, 1); d.unite(1, 0); d.components == 3`
    @get pub fn components(self) -> Int = parts

    /// The representative of each set, once each, lazily.
    ///
    /// cost: O(n) for the whole walk, no allocation
    /// empty: yields nothing when `V` is empty
    /// order: increasing positions of `V`
    /// traps: never
    /// example: `var d = Dsu<0..4>(); d.unite(3, 1); d.roots().collect() == [0, 2, 3]`
    pub fn roots(self) -> yields V:
        for v in V:
            unchecked:
                if p[v.ord] < 0: yield v

    /// Every set as a list of its positions, each position in exactly one list.
    ///
    /// cost: O(n alpha(n)); allocates the lists
    /// empty: `[]` when `V` is empty
    /// order: lists by their least position, positions increasing within each
    /// traps: never
    /// example: `var d = Dsu<0..4>(); d.unite(3, 1); d.groups() == [[0], [1, 3], [2]]`
    pub fn groups(self) -> List<List<V>>:
        var gid: [V]Int = -1
        var out: List<List<V>> = []
        for v in V:
            let r = self.find(v)
            if gid[r] < 0:
                gid[r] = out.len
                out.push([])
            out[gid[r]].push(v)
        out

/// Union-find where every set carries a value of type `T`. When two sets merge, the
/// merged set gets `join(value of a's set, value of b's set)` for `unite(a, b)`, whichever
/// root is kept, so a directional join like `(_, b) => b` means "b's value wins".
/// Costs are those of `Dsu`, plus one `join` per merge; memory is three `[V]` arrays.
///
/// `mark` and `undo_to` step back in time like `Dsu`'s, and restore the values too: every
/// merge and every `set` since the mark is undone, O(1) each.
pub struct DsuOf<V, T>:
    @cache up: [V]V
    sz: [V]1..=V.count
    val: [V]T
    join: (T, T) -> T
    parts: 0..=2_000_000_000
    /// Since the first `mark`, one entry per change: the root that went under another (the
    /// root itself for a `set`), the root whose value changed, and that value before.
    pub log: List<(V, V, T)>
    marked: Bool

    /// A union-find over `V` with every position alone, the set of `v` holding `init(v)`, and
    /// `join` combining the values of two sets when they merge.
    ///
    /// cost: O(n) calls of `init`; three `[V]` arrays
    /// traps: never
    /// example: `let d = DsuOf<0..3, Int>(init: v => v + 1, join: (x, y) => x + y); d.get(2) == 3`
    pub fn new(init: (V) -> T, join: (T, T) -> T) -> DsuOf<V, T> =
        DsuOf([V] v => v, [V] _ => 1, [V] v => init(v), join, V.count, [], false)

    /// The representative of `v`'s set, as for [`Dsu.find`](Dsu.find). Halves paths through
    /// `@cache` until the first `mark`.
    ///
    /// cost: O(alpha(n)) amortized; O(log n) after the first `mark`
    /// ties: None; the representative is fixed by the merges (larger set's root, `a`'s root on equal sizes)
    /// traps: never
    /// example: `var d = DsuOf<0..3, Int>(init: v => Int(v), join: (x, y) => x + y); d.unite(1, 2); d.find(2) == 1`
    pub fn find(self, v: V) -> V:
        var x = v
        while up[x] != x:
            let par = up[x]
            if not marked: up[x] = up[par]
            x = par
        x

    /// The value of `v`'s set.
    ///
    /// cost: O(alpha(n)) amortized; O(log n) after the first `mark`
    /// traps: never
    /// example: `var d = DsuOf<0..3, Int>(init: v => v + 1, join: (x, y) => x + y); d.unite(0, 2); d.get(2) == 4 and d.get(1) == 2`
    pub fn get(self, v: V) -> T:
        let r = self.find(v)
        val[r]

    /// Replaces the value of `v`'s whole set with `x`.
    ///
    /// cost: O(alpha(n)) amortized; O(log n) after the first `mark`, plus one log entry
    /// traps: never
    /// example: `var d = DsuOf<0..3, Int>(init: v => Int(v), join: (x, y) => x + y); d.unite(0, 1); d.set(1, 7); d.get(0) == 7`
    pub fn set(mut self, v: V, x: T):
        let r = self.find(v)
        if marked: log.push((r, r, val[r]))
        val[r] = x

    /// Merges the sets of `a` and `b`, giving the merged set `join(value of a's set, value of b's
    /// set)` in that argument order whichever root is kept; `false` if they were already one set
    /// (the value is left alone then and `join` is not called).
    ///
    /// The root of the larger set is kept; on equal sizes `a`'s root is kept.
    ///
    /// cost: as [`Dsu.unite`](Dsu.unite), plus one call of `join`
    /// traps: never
    /// example: `var d = DsuOf<0..3, Int>(init: v => Int(v), join: (_, y) => y); d.unite(2, 0); d.get(2) == 0`
    /// example: `var d = DsuOf<0..3, Int>(init: v => Int(v), join: (x, y) => x + y); d.unite(0, 1); not d.unite(1, 0) and d.get(0) == 1`
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

    /// As [`unite`](DsuOf.unite), but says which root stayed and which went under it. `None` if
    /// `a` and `b` were already one set.
    ///
    /// cost: as `unite`
    /// traps: never
    /// example: `var d = DsuOf<0..3, Int>(init: v => Int(v), join: (x, y) => x + y); d.merge(2, 1) == Some((kept: 2, merged: 1))`
    pub fn merge(mut self, a: V, b: V) -> ?(kept: V, merged: V):
        let ra = self.find(a)
        let rb = self.find(b)
        if ra == rb: return None
        let (x, y) = if sz[ra] >= sz[rb]: (ra, rb) else: (rb, ra)
        self.unite(ra, rb)
        Some((kept: x, merged: y))

    /// Whether `a` and `b` are in the same set.
    ///
    /// cost: O(alpha(n)) amortized; O(log n) after the first `mark`
    /// traps: never
    /// example: `var d = DsuOf<0..3, Int>(init: v => Int(v), join: (x, y) => x + y); d.unite(0, 2); d.same(0, 2) and not d.same(1, 2)`
    pub fn same(self, a: V, b: V) -> Bool = self.find(a) == self.find(b)

    /// The number of positions in `v`'s set, `v` included.
    ///
    /// cost: O(alpha(n)) amortized; O(log n) after the first `mark`
    /// traps: never
    /// example: `var d = DsuOf<0..3, Int>(init: v => Int(v), join: (x, y) => x + y); d.unite(0, 2); d.size(0) == 2`
    pub fn size(mut self, v: V) -> Int:
        let r = self.find(v)
        sz[r]

    /// The number of sets.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var d = DsuOf<0..3, Int>(init: v => Int(v), join: (x, y) => x + y); d.unite(0, 2); d.components == 2`
    @get pub fn components(self) -> Int = parts

    /// The representative of each set, once each, lazily.
    ///
    /// cost: O(n) for the whole walk, no allocation
    /// empty: yields nothing when `V` is empty
    /// order: increasing positions of `V`
    /// traps: never
    /// example: `var d = DsuOf<0..3, Int>(init: v => Int(v), join: (x, y) => x + y); d.unite(2, 0); d.roots().collect() == [1, 2]`
    pub fn roots(self) -> yields V:
        for v in V:
            if up[v] == v: yield v

    /// The value of each set, once each, lazily: one per root, in the order of
    /// [`roots`](DsuOf.roots). O(1) to make.
    ///
    /// cost: O(n) for the whole walk, no allocation
    /// empty: yields nothing when `V` is empty
    /// order: by representative, increasing positions of `V`
    /// traps: never
    /// example: `var d = DsuOf<0..3, Int>(init: v => v + 1, join: (x, y) => x + y); d.unite(2, 0); d.values.collect() == [2, 4]`
    @get pub fn values(self) -> yields T:
        for v in V:
            if up[v] == v: yield val[v]

    /// The present, to return to with [`undo_to`](DsuOf.undo_to). The first call switches path
    /// shortening off for good.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var d = DsuOf<0..2, Int>(init: v => Int(v), join: (x, y) => x + y); let m = d.mark(); d.unite(0, 1); d.undo_to(m); d.get(0) == 0`
    pub fn mark(mut self) -> Mark:
        marked = true
        Mark(log.len)

    /// Undoes every merge and every `set` made since `snap = mark()`, newest first, restoring the
    /// sets and their values exactly.
    ///
    /// cost: O(1) per undone change
    /// traps: when `snap` lies after the present (its changes were already undone)
    /// example: `var d = DsuOf<0..2, Int>(init: v => Int(v), join: (x, y) => x + y); let m = d.mark(); d.unite(0, 1); d.set(0, 9); d.undo_to(m); d.get(0) == 0 and d.get(1) == 1`
    pub fn undo_to(mut self, snap: Mark):
        assert snap.at <= log.len
        while log.len > snap.at:
            let (y, x, old) = log.pop()!
            if y != x:
                sz[x] -= sz[y]
                up[y] = y
                parts += 1
            val[x] = old

    /// Every set as a list of its positions, as `Dsu.groups`; values are read with `get`.
    ///
    /// cost: O(n alpha(n)); allocates the lists
    /// empty: `[]` when `V` is empty
    /// order: lists by their least position, positions increasing within each
    /// traps: never
    /// example: `var d = DsuOf<0..3, Int>(init: v => Int(v), join: (x, y) => x + y); d.unite(2, 0); d.groups() == [[0, 2], [1]]`
    pub fn groups(mut self) -> List<List<V>>:
        var gid: [V]Int = -1
        var out: List<List<V>> = []
        for v in V:
            let r = self.find(v)
            if gid[r] < 0:
                gid[r] = out.len
                out.push([])
            out[gid[r]].push(v)
        out

/// Union-find that knows, for two positions of one set, whether they are on the same side
/// or on opposite sides: two-colouring, parity constraints, `x[a] xor x[b] == d`.
/// `unite` and `diff` are O(log n) (union by size; paths are never shortened, so `mark` and
/// `undo_to` work at any time, O(1) per undone `unite`). Memory is three `[V]` arrays.
pub struct ParityDsu<V>:
    up: [V]V
    sz: [V]1..=V.count
    /// Is the position on the other side from `up` of it?
    pub flip: [V]Bool
    /// One entry per `unite` that changed something: the root it hung under another root,
    /// or (with `false`) the root of a set where it found a contradiction.
    pub log: List<(V, Bool)>
    conflicts: 0..=2_000_000_000

    /// A parity union-find over `V` with every position alone and nothing known.
    ///
    /// cost: O(n); three `[V]` arrays
    /// traps: never
    /// example: `let d = ParityDsu<0..3>(); d.consistent and d.diff(0, 1) == None`
    pub fn new() -> ParityDsu<V>:
        ParityDsu([V] v => v, [V] v => 1, [V] v => false, [], 0)

    /// The representative of `v`'s set and whether `v` is on the other side from it.
    ///
    /// cost: O(log n)
    /// ties: None; the representative is fixed by the merges (larger set's root, `a`'s root on equal sizes)
    /// traps: never
    /// example: `var d = ParityDsu<0..3>(); d.unite(0, 1); d.find(1) == (root: 0, flip: true)`
    /// example: `let d = ParityDsu<0..3>(); d.find(2) == (root: 2, flip: false)`
    pub fn find(self, v: V) -> (root: V, flip: Bool):
        var (x, side) = (v, false)
        while up[x] != x:
            side = side != flip[x]
            x = up[x]
        (root: x, flip: side)

    /// Records that `a` and `b` are on opposite sides (`differ: false`: the same side). An edge of
    /// a graph is `unite(u, v)`.
    ///
    /// Returns `false` when that contradicts what is already known: nothing is merged, the
    /// contradiction is counted, and [`consistent`](ParityDsu.consistent) stays false until it is
    /// undone. A constraint that is already implied returns `true` and changes nothing. Merging keeps
    /// the larger set's root, `a`'s root on equal sizes.
    ///
    /// cost: O(log n), plus one log entry when it merges or contradicts
    /// traps: never
    /// example: `var d = ParityDsu<0..3>(); d.unite(0, 1); d.unite(1, 2); not d.unite(0, 2) and not d.consistent`
    /// example: `var d = ParityDsu<0..3>(); d.unite(0, 1); d.unite(1, 2); d.unite(0, 2, differ: false) and d.consistent`
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

    /// Whether `a` and `b` are on opposite sides; `None` when nothing relates them yet.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `var d = ParityDsu<0..3>(); d.unite(0, 1); d.unite(1, 2); d.diff(0, 2) == Some(false) and d.diff(1, 0) == Some(true)`
    /// example: `ParityDsu<0..3>().diff(0, 2) == None`
    pub fn diff(self, a: V, b: V) -> ?Bool:
        let (ra, sa) = self.find(a)
        let (rb, sb) = self.find(b)
        if ra != rb: None else: sa != sb

    /// Whether `a` and `b` are in the same set, so that their relative side is known.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `var d = ParityDsu<0..3>(); d.unite(0, 1); d.same(1, 0) and not d.same(0, 2)`
    pub fn same(self, a: V, b: V) -> Bool = self.find(a).root == self.find(b).root

    /// The number of positions in `v`'s set, `v` included.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `var d = ParityDsu<0..3>(); d.unite(0, 1); d.unite(1, 2); d.size(2) == 3`
    pub fn size(self, v: V) -> Int = sz[self.find(v).root]

    /// Whether no contradiction is in effect: every `unite` since the last undone one agreed with
    /// those before it. For a graph: is it two-colourable?
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var d = ParityDsu<0..3>(); d.unite(0, 1); d.unite(1, 2); d.unite(2, 0); not d.consistent`
    @get pub fn consistent(self) -> Bool = conflicts == 0

    /// The present, to return to with [`undo_to`](ParityDsu.undo_to). Paths are never shortened,
    /// so marking changes nothing else.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var d = ParityDsu<0..2>(); let m = d.mark(); d.unite(0, 1); d.undo_to(m); d.diff(0, 1) == None`
    pub fn mark(self) -> Mark = Mark(log.len)

    /// Takes back every `unite` since `snap = mark()`, contradictions included, newest first.
    ///
    /// cost: O(1) per undone `unite`
    /// traps: when `snap` lies after the present (its changes were already undone)
    /// example: `var d = ParityDsu<0..3>(); d.unite(0, 1); d.unite(1, 2); let m = d.mark(); d.unite(2, 0); d.undo_to(m); d.consistent`
    pub fn undo_to(mut self, snap: Mark):
        assert snap.at <= log.len
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

    /// A potential union-find over `V` with every position alone at the identity potential.
    /// Constraints combine with the group operation `op`, `+` on `Int` when it is `None`.
    ///
    /// cost: O(n); three `[V]` arrays
    /// traps: never
    /// example: `let d = PotentialDsu<0..3>(); d.components == 3 and d.consistent`
    pub fn new<G>(op: ?G = None) -> PotentialDsu<V>:
        if let Some(_) = op: todo
        PotentialDsu([V] v => v, [V] _ => 1, [V] _ => 0, V.count, 0, [])

    /// The representative of `v`'s set and `val[v] - val[root]`.
    ///
    /// cost: O(log n)
    /// ties: None; the representative is fixed by the merges (larger set's root, `a`'s root on equal sizes)
    /// traps: never
    /// example: `var d = PotentialDsu<0..3>(); d.unite(0, 1, 5); d.find(1) == (root: 0, offset: 5)`
    /// example: `let d = PotentialDsu<0..3>(); d.find(2) == (root: 2, offset: 0)`
    pub fn find(self, v: V) -> (root: V, offset: Int):
        var x = v
        var d = 0
        while up[x] != x:
            d += pot[x]
            x = up[x]
        (root: x, offset: d)

    /// Records `val[b] - val[a] == w`.
    ///
    /// Returns `false` when that contradicts what is already known: nothing is merged, the
    /// contradiction is counted, and [`consistent`](PotentialDsu.consistent) stays false until it is
    /// undone. A constraint that is already implied returns `true` and changes nothing. Merging keeps
    /// the larger set's root, `a`'s root on equal sizes.
    ///
    /// cost: O(log n), plus one log entry when it merges or contradicts
    /// traps: never
    /// example: `var d = PotentialDsu<0..3>(); d.unite(0, 1, 5); d.unite(1, 2, 3); not d.unite(0, 2, 7) and d.unite(0, 2, 8)`
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

    /// `val[b] - val[a]` when `a` and `b` are in the same set, else `None`.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `var d = PotentialDsu<0..3>(); d.unite(0, 1, 5); d.unite(1, 2, 3); d.diff(0, 2) == Some(8) and d.diff(2, 0) == Some(-8)`
    /// example: `PotentialDsu<0..3>().diff(0, 1) == None`
    pub fn diff(self, a: V, b: V) -> ?Int:
        let (ra, da) = self.find(a)
        let (rb, db) = self.find(b)
        if ra != rb: None else: db - da

    /// Whether `a` and `b` are in the same set, so that their difference is known.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `var d = PotentialDsu<0..3>(); d.unite(2, 0, 1); d.same(0, 2) and not d.same(0, 1)`
    pub fn same(self, a: V, b: V) -> Bool = self.find(a).root == self.find(b).root

    /// The number of positions in `v`'s set, `v` included.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `var d = PotentialDsu<0..3>(); d.unite(2, 0, 1); d.size(0) == 2`
    pub fn size(self, v: V) -> Int = sz[self.find(v).root]

    /// The number of sets.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var d = PotentialDsu<0..3>(); d.unite(0, 1, 5); d.components == 2`
    @get pub fn components(self) -> Int = parts

    /// Whether no contradiction is in effect.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var d = PotentialDsu<0..2>(); d.unite(0, 1, 5); not d.unite(1, 0, 5) and not d.consistent`
    @get pub fn consistent(self) -> Bool = conflicts == 0

    /// The present, to return to with [`undo_to`](PotentialDsu.undo_to).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var d = PotentialDsu<0..2>(); let m = d.mark(); d.unite(0, 1, 5); d.undo_to(m); d.diff(0, 1) == None`
    pub fn mark(self) -> Mark = Mark(log.len)

    /// Takes back every `unite` since `snap = mark()`, contradictions included, newest first.
    ///
    /// cost: O(1) per undone `unite`
    /// traps: when `snap` lies after the present (its changes were already undone)
    /// example: `var d = PotentialDsu<0..2>(); d.unite(0, 1, 5); let m = d.mark(); d.unite(1, 0, 5); d.undo_to(m); d.consistent and d.diff(0, 1) == Some(5)`
    pub fn undo_to(mut self, snap: Mark):
        assert snap.at <= log.len
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

    /// An array over `D` holding `init(i)` at each `i`, that can step back to any mark.
    ///
    /// cost: O(n) calls of `init`, for n = `D.count`
    /// traps: never
    /// example: `let a = RollbackArray<0..3, Int>(init: i => Int(i) * Int(i)); a.get(2) == 4`
    pub fn new(init: D -> T) -> RollbackArray<D, T> = RollbackArray([D] i => init(i), [])

    /// The current value at `i`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var a = RollbackArray<0..3, Int>(init: i => Int(i)); a.set(1, 7); a.get(1) == 7 and a.get(2) == 2`
    pub fn get(self, i: D) -> T = data[i]

    /// Sets `i` to `val`, logging the previous value so `undo_to` can restore it. Every `set` is
    /// logged, marked or not.
    ///
    /// cost: O(1), plus one log entry
    /// traps: never
    /// example: `var a = RollbackArray<0..2, Int>(init: _ => 0); a.set(0, 5); a.get(0) == 5`
    pub fn set(mut self, i: D, val: T):
        log.push((i, data[i]))
        data[i] = val

    /// The present, to return to with [`undo_to`](RollbackArray.undo_to).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var a = RollbackArray<0..2, Int>(init: _ => 0); let m = a.mark(); a.set(1, 3); a.undo_to(m); a.get(1) == 0`
    pub fn mark(self) -> Mark = Mark(log.len)

    /// Reverts every `set` since `snap = mark()`, newest first, so each position holds what it held
    /// at the mark.
    ///
    /// cost: O(1) per undone `set`
    /// traps: when `snap` lies after the present (its changes were already undone)
    /// example: `var a = RollbackArray<0..2, Int>(init: _ => 0); let m = a.mark(); a.set(0, 5); a.set(0, 6); a.undo_to(m); a.get(0) == 0`
    pub fn undo_to(mut self, snap: Mark):
        assert snap.at <= log.len
        while log.len > snap.at:
            let (i, old) = log.pop()!
            data[i] = old

/// Path-copying persistent disjoint-set union over domain `V`.
/// Every version remains readable; `find`, `unite`, `same`, and `size` run in O(log^2 n).
pub struct PersistentDsu<V>:
    _parts: Int = 0
    _kid: List<(Int, Int)> = []
    _cell: List<(Int, Int)> = []
    _roots: List<Int> = []

    /// A persistent union-find over `V` holding version 0, every position in its own set.
    ///
    /// cost: O(n)
    /// traps: never
    /// example: `let d = PersistentDsu<0..3>(); not d.same(d.base, 0, 1)`
    pub fn new() -> PersistentDsu<V>:
        var d: PersistentDsu<V> = PersistentDsu(V.count, [], [], [])
        let root = d.build(0, V.count)
        d._roots.push(root)
        d

    fn build(mut self, lo: Int, hi: Int) -> Int:
        if hi - lo <= 1:
            _kid.push((-1, -1))
            _cell.push((lo, 1))
            return _kid.len - 1
        let mid = (lo + hi) / 2
        let l = self.build(lo, mid)
        let r = self.build(mid, hi)
        _kid.push((l, r))
        _cell.push((0, 0))
        _kid.len - 1

    pub fn get(self, root: Int, i: Int) -> (Int, Int):
        var t = root
        var (lo, hi) = (0, V.count)
        while hi - lo > 1:
            let mid = (lo + hi) / 2
            let (l, r) = _kid[t]
            if i < mid:
                t = l
                hi = mid
            else:
                t = r
                lo = mid
        _cell[t]

    fn put(mut self, t: Int, lo: Int, hi: Int, i: Int, cell: (Int, Int)) -> Int:
        if hi - lo <= 1:
            _kid.push((-1, -1))
            _cell.push(cell)
            return _kid.len - 1
        let mid = (lo + hi) / 2
        var (l, r) = _kid[t]
        if i < mid: l = self.put(l, lo, mid, i, cell)
        else: r = self.put(r, mid, hi, i, cell)
        _kid.push((l, r))
        _cell.push((0, 0))
        _kid.len - 1

    fn root_ord(self, ver: Int, start: Int) -> Int:
        assert 0 <= ver and ver < _roots.len, "`ver` is not a version of this PersistentDsu"
        var x = start
        while true:
            let (p, _) = self.get(_roots[ver], x)
            if p == x: return x
            x = p
        x

    /// The handle of version 0, where every position is alone.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `PersistentDsu<0..3>().base == 0`
    @get pub fn base(self) -> Int = 0

    /// The representative of `v`'s set in version `ver`, chosen by union by size as in
    /// [`Dsu.find`](Dsu.find).
    ///
    /// cost: O(log² n)
    /// ties: None; the representative is fixed by the merges (larger set's root, `a`'s root on equal sizes)
    /// traps: when `ver` is not a version handle of `self`
    /// example: `var d = PersistentDsu<0..3>(); let v1 = d.unite(d.base, 2, 1); d.find(v1, 1) == 2 and d.find(d.base, 1) == 1`
    pub fn find(self, ver: Int, v: V) -> V = V.nth(self.root_ord(ver, v.ord))

    /// Whether `a` and `b` are in the same set in version `ver`.
    ///
    /// cost: O(log² n)
    /// traps: when `ver` is not a version handle of `self`
    /// example: `var d = PersistentDsu<0..3>(); let v1 = d.unite(d.base, 0, 1); d.same(v1, 0, 1) and not d.same(d.base, 0, 1)`
    pub fn same(
        self,
        ver: Int,
        a: V,
        b: V,
    ) -> Bool = self.root_ord(ver, a.ord) == self.root_ord(ver, b.ord)

    /// The number of positions in `v`'s set in version `ver`, `v` included.
    ///
    /// cost: O(log² n)
    /// traps: when `ver` is not a version handle of `self`
    /// example: `var d = PersistentDsu<0..3>(); let v1 = d.unite(d.base, 0, 1); d.size(v1, 1) == 2 and d.size(d.base, 1) == 1`
    pub fn size(self, ver: Int, v: V) -> Int:
        let (_, n) = self.get(_roots[ver], self.root_ord(ver, v.ord))
        n

    /// A new version equal to `ver` with the sets of `a` and `b` merged; `ver` itself is unchanged.
    ///
    /// Handles are numbered in creation order: the k-th call returns `k`. A call whose `a` and `b`
    /// are already one set in `ver` still returns a fresh handle, equal in content to `ver`. Merging
    /// keeps the larger set's root, `a`'s root on equal sizes.
    ///
    /// cost: O(log² n) time and O(log n) memory
    /// traps: when `ver` is not a version handle of `self`
    /// example: `var d = PersistentDsu<0..3>(); let v1 = d.unite(d.base, 0, 1); let v2 = d.unite(d.base, 1, 2); v1 == 1 and v2 == 2 and d.same(v2, 1, 2) and not d.same(v2, 0, 1)`
    pub fn unite(mut self, ver: Int, a: V, b: V) -> Int:
        var x = self.root_ord(ver, a.ord)
        var y = self.root_ord(ver, b.ord)
        var root = _roots[ver]
        if x != y:
            let (_, nx) = self.get(root, x)
            let (_, ny) = self.get(root, y)
            if nx < ny: (x, y) = (y, x)
            root = self.put(root, 0, V.count, y, (x, ny))
            root = self.put(root, 0, V.count, x, (x, nx + ny))
        _roots.push(root)
        _roots.len - 1

    /// Preallocates room for at least `nodes` persistent-array nodes, so later `unite`s do not
    /// grow the pool. Changes no version.
    ///
    /// cost: O(nodes)
    /// traps: when `nodes < 0`
    /// example: `var d = PersistentDsu<0..3>(); d.reserve(100); d.size(d.base, 0) == 1`
    pub fn reserve(mut self, nodes: Int):
        _kid.reserve(nodes)
        _cell.reserve(nodes)

@test
fn potential_dsu_and_rollback_array():
    domain V = 0..6
    var d = PotentialDsu<V>()
    assert d.components == 6 and d.consistent
    assert d.unite(V(0), V(1), 5)
    assert d.unite(V(1), V(2), -2)
    assert d.diff(V(0), V(2)) == 3 and d.diff(V(2), V(0)) == -3
    assert d.diff(V(0), V(3)) == None
    let snap = d.mark()
    assert not d.unite(V(0), V(2), 4)
    assert not d.consistent
    assert d.unite(V(2), V(3), 10)
    assert d.diff(V(0), V(3)) == 13 and d.components == 3
    d.undo_to(snap)
    assert d.consistent and d.components == 4 and d.diff(V(0), V(3)) == None
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
    domain V = 0..10
    var d = Dsu<V>()
    var marks: List<Mark> = []
    var reps: List<List<V>> = []
    var parts: List<Int> = []
    for step in 0..30:
        if step % 5 == 0:
            marks.push(d.mark())
            reps.push([d.find(v) for v in V])
            parts.push(d.components)
        d.unite(V.nth(rand(0, 9)), V.nth(rand(0, 9)))
    var i = marks.len - 1
    while i >= 0:
        d.undo_to(marks[i])
        assert [d.find(v) for v in V] == reps[i] and d.components == parts[i]
        i -= 1

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
    domain V = 0..8
    var d = DsuOf<V, Int>(v => v.idx, (x, y) => x + y)
    var label = [V] v => v
    var total = [V] v => v.idx
    var saved_label = [V] v => v
    var saved_total = [V] v => 0
    var snap = d.mark()
    for step in 0..24:
        if step == 12:
            snap = d.mark()
            for v in V:
                saved_label[v] = label[v]
                saved_total[v] = total[v]
        let a = V.nth(rand(0, 7))
        if step % 4 == 3:
            let x = rand(-5, 5)
            d.set(a, x)
            total[label[a]] = x
        else:
            let b = V.nth(rand(0, 7))
            let (la, lb) = (label[a], label[b])
            assert d.unite(a, b) == (la != lb)
            if la != lb:
                total[la] += total[lb]
                for v in V:
                    if label[v] == lb: label[v] = la
        for v in V:
            assert d.get(v) == total[label[v]]
    d.undo_to(snap)
    for v in V:
        assert d.get(v) == saved_total[saved_label[v]]
        for u in V:
            assert d.same(u, v) == (saved_label[u] == saved_label[v])

@test
fn a_directional_join_keeps_the_second_value():
    domain V = 1..=4
    var d = DsuOf<V, ?V>(v => None, (_, b) => b)
    d.set(V(2), Some(V(1)))
    assert d.unite(V(3), V(2)) and d.get(V(3)) == Some(V(1))
    let snap = d.mark()
    d.set(V(4), Some(V(4)))
    if let (kept, merged) = d.merge(V(4), V(3)):
        assert d.find(merged) == kept
    assert d.get(V(2)) == Some(V(1)) and d.components == 2
    d.undo_to(snap)
    assert d.get(V(4)) == None and d.get(V(2)) == Some(V(1)) and d.components == 3

@test
fn parity_matches_a_brute_colouring_and_rolls_back():
    domain V = 0..7
    var d = ParityDsu<V>()
    var label = [V] v => v
    var side = [V] v => false
    var conflicts = 0
    var saved_label = [V] v => v
    var saved_side = [V] v => false
    var saved_conflicts = 0
    var snap = d.mark()
    for step in 0..30:
        if step == 15:
            snap = d.mark()
            for v in V:
                saved_label[v] = label[v]
                saved_side[v] = side[v]
            saved_conflicts = conflicts
        let a = V.nth(rand(0, 6))
        let b = V.nth(rand(0, 6))
        let differ = rand(0, 1) == 1
        let (la, lb) = (label[a], label[b])
        if la == lb:
            let ok = (side[a] != side[b]) == differ
            if not ok: conflicts += 1
            assert d.unite(a, b, differ: differ) == ok
        else:
            assert d.unite(a, b, differ: differ)
            let flip = (side[a] != side[b]) != differ
            for v in V:
                if label[v] == lb:
                    label[v] = la
                    side[v] = side[v] != flip
        assert d.consistent == (conflicts == 0)
        for u in V:
            for v in V:
                let want: ?Bool = if label[u] == label[v]: Some(side[u] != side[v]) else: None
                assert d.diff(u, v) == want
    d.undo_to(snap)
    assert d.consistent == (saved_conflicts == 0)
    for u in V:
        for v in V:
            let want: ?Bool = if saved_label[u] == saved_label[v]: Some(
                saved_side[u] != saved_side[v]
            ) else: None
            assert d.diff(u, v) == want

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
