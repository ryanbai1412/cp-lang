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
    joined: List<Int>
    marked: Bool

    /// A union-find over `V` with every position in its own set. Pass `rollback: true` to switch
    /// path shortening off from the start, as the first `mark()` would.
    ///
    /// cost: O(n) for n = `V.count`; one `[V]` array
    /// traps: never
    /// example: `let d = Dsu<0..3>(); d.components == 3`
    pub fn new(rollback: Bool = false) -> Dsu<V> = todo

    fn find_ord(self, start: Int) -> Int = todo

    /// The representative of `v`'s set: the root that union by size kept. Two positions are in one
    /// set exactly when they have the same representative.
    ///
    /// Halves the path it walks through `@cache`, so a read-only `Dsu` answers it too; from the
    /// first `mark` on, paths are left alone.
    ///
    /// cost: O(alpha(n)) amortized; O(log n) after the first `mark`
    /// ties: none; the representative is fixed by the merges (larger set's root, `a`'s root on equal sizes)
    /// traps: never
    /// example: `var d = Dsu<0..4>(); d.unite(2, 3); d.find(3) == 2 and d.find(0) == 0`
    pub fn find(self, v: V) -> V = todo

    /// The present, to return to with [`undo_to`](Dsu.undo_to). The first call switches path
    /// shortening off for good.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var d = Dsu<0..3>(); let m = d.mark(); d.unite(0, 1); d.undo_to(m); d.components == 3`
    pub fn mark(mut self) -> Mark = todo

    /// Undoes every `unite` made since `snap = mark()`, newest first, so the sets, sizes and
    /// representatives are exactly what they were at the mark. Marks nest: undoing to an older mark
    /// passes over newer ones.
    ///
    /// cost: O(1) per undone `unite`
    /// traps: when `snap` lies after the present (its changes were already undone)
    /// example: `var d = Dsu<0..3>(); d.unite(0, 1); let m = d.mark(); d.unite(1, 2); d.undo_to(m); d.same(0, 1) and not d.same(0, 2)`
    pub fn undo_to(mut self, snap: Mark) = todo

    /// Merges the sets of `a` and `b`; `false` if they were already one set (nothing changes then).
    ///
    /// The root of the larger set is kept; on equal sizes `a`'s root is kept.
    ///
    /// cost: O(alpha(n)) amortized; O(log n) after the first `mark`, plus one log entry
    /// traps: never
    /// example: `var d = Dsu<0..3>(); d.unite(0, 1) and not d.unite(1, 0)`
    pub fn unite(mut self, a: V, b: V) -> Bool = todo

    /// As [`unite`](Dsu.unite), but says which root stayed and which went under it, so data kept
    /// per set can follow: `if let (kept, merged) = d.merge(a, b): sum[kept] += sum[merged]`.
    /// `None` if `a` and `b` were already one set.
    ///
    /// cost: as `unite`
    /// traps: never
    /// example: `var d = Dsu<0..3>(); d.merge(1, 0) == Some((kept: 1, merged: 0))`
    /// example: `var d = Dsu<0..3>(); d.unite(0, 1); d.merge(2, 1) == Some((kept: 0, merged: 2))`
    /// example: `var d = Dsu<0..3>(); d.unite(0, 1); d.merge(1, 0) == None`
    pub fn merge(mut self, a: V, b: V) -> ?(kept: V, merged: V) = todo

    /// Whether `a` and `b` are in the same set.
    ///
    /// cost: O(alpha(n)) amortized; O(log n) after the first `mark`
    /// traps: never
    /// example: `var d = Dsu<0..3>(); d.unite(0, 2); d.same(2, 0) and not d.same(0, 1)`
    pub fn same(mut self, a: V, b: V) -> Bool = todo

    /// The number of positions in `v`'s set, `v` included.
    ///
    /// cost: O(alpha(n)) amortized; O(log n) after the first `mark`
    /// traps: never
    /// example: `var d = Dsu<0..3>(); d.unite(0, 2); d.size(2) == 2 and d.size(1) == 1`
    pub fn size(mut self, v: V) -> Int = todo

    /// The number of sets: `V.count` minus the successful `unite`s still in effect.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var d = Dsu<0..4>(); d.unite(0, 1); d.unite(1, 0); d.components == 3`
    pub @get fn components(self) -> Int = todo

    /// The representative of each set, once each, lazily.
    ///
    /// cost: O(n) for the whole walk, no allocation
    /// empty: yields nothing when `V` is empty
    /// order: increasing positions of `V`
    /// traps: never
    /// example: `var d = Dsu<0..4>(); d.unite(3, 1); d.roots().collect() == [0, 2, 3]`
    pub fn roots(self) -> yields V = todo

    /// Every set as a list of its positions, each position in exactly one list.
    ///
    /// cost: O(n alpha(n)); allocates the lists
    /// empty: `[]` when `V` is empty
    /// order: lists by their least position, positions increasing within each
    /// traps: never
    /// example: `var d = Dsu<0..4>(); d.unite(3, 1); d.groups() == [[0], [1, 3], [2]]`
    pub fn groups(mut self) -> List<List<V>> = todo

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
    log: List<(V, V, T)>
    marked: Bool

    /// A union-find over `V` with every position alone, the set of `v` holding `init(v)`, and
    /// `join` combining the values of two sets when they merge.
    ///
    /// cost: O(n) calls of `init`; three `[V]` arrays
    /// traps: never
    /// example: `let d = DsuOf<0..3, Int>(init: v => v + 1, join: (x, y) => x + y); d.get(2) == 3`
    pub fn new(init: (V) -> T, join: (T, T) -> T) -> DsuOf<V, T> = todo

    /// The representative of `v`'s set, as for [`Dsu.find`](Dsu.find). Halves paths through
    /// `@cache` until the first `mark`.
    ///
    /// cost: O(alpha(n)) amortized; O(log n) after the first `mark`
    /// ties: none; the representative is fixed by the merges (larger set's root, `a`'s root on equal sizes)
    /// traps: never
    /// example: `var d = DsuOf<0..3, Int>(init: v => v, join: (x, y) => x + y); d.unite(1, 2); d.find(2) == 1`
    pub fn find(self, v: V) -> V = todo

    /// The value of `v`'s set.
    ///
    /// cost: O(alpha(n)) amortized; O(log n) after the first `mark`
    /// traps: never
    /// example: `var d = DsuOf<0..3, Int>(init: v => v + 1, join: (x, y) => x + y); d.unite(0, 2); d.get(2) == 4 and d.get(1) == 2`
    pub fn get(mut self, v: V) -> T = todo

    /// Replaces the value of `v`'s whole set with `x`.
    ///
    /// cost: O(alpha(n)) amortized; O(log n) after the first `mark`, plus one log entry
    /// traps: never
    /// example: `var d = DsuOf<0..3, Int>(init: v => v, join: (x, y) => x + y); d.unite(0, 1); d.set(1, 7); d.get(0) == 7`
    pub fn set(mut self, v: V, x: T) = todo

    /// Merges the sets of `a` and `b`, giving the merged set `join(value of a's set, value of b's
    /// set)` in that argument order whichever root is kept; `false` if they were already one set
    /// (the value is left alone then and `join` is not called).
    ///
    /// The root of the larger set is kept; on equal sizes `a`'s root is kept.
    ///
    /// cost: as [`Dsu.unite`](Dsu.unite), plus one call of `join`
    /// traps: never
    /// example: `var d = DsuOf<0..3, Int>(init: v => v, join: (_, y) => y); d.unite(2, 0); d.get(2) == 0`
    /// example: `var d = DsuOf<0..3, Int>(init: v => v, join: (x, y) => x + y); d.unite(0, 1); not d.unite(1, 0) and d.get(0) == 1`
    pub fn unite(mut self, a: V, b: V) -> Bool = todo

    /// As [`unite`](DsuOf.unite), but says which root stayed and which went under it. `None` if
    /// `a` and `b` were already one set.
    ///
    /// cost: as `unite`
    /// traps: never
    /// example: `var d = DsuOf<0..3, Int>(init: v => v, join: (x, y) => x + y); d.merge(2, 1) == Some((kept: 2, merged: 1))`
    pub fn merge(mut self, a: V, b: V) -> ?(kept: V, merged: V) = todo

    /// Whether `a` and `b` are in the same set.
    ///
    /// cost: O(alpha(n)) amortized; O(log n) after the first `mark`
    /// traps: never
    /// example: `var d = DsuOf<0..3, Int>(init: v => v, join: (x, y) => x + y); d.unite(0, 2); d.same(0, 2) and not d.same(1, 2)`
    pub fn same(mut self, a: V, b: V) -> Bool = todo

    /// The number of positions in `v`'s set, `v` included.
    ///
    /// cost: O(alpha(n)) amortized; O(log n) after the first `mark`
    /// traps: never
    /// example: `var d = DsuOf<0..3, Int>(init: v => v, join: (x, y) => x + y); d.unite(0, 2); d.size(0) == 2`
    pub fn size(mut self, v: V) -> Int = todo

    /// The number of sets.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var d = DsuOf<0..3, Int>(init: v => v, join: (x, y) => x + y); d.unite(0, 2); d.components == 2`
    pub @get fn components(self) -> Int = todo

    /// The representative of each set, once each, lazily.
    ///
    /// cost: O(n) for the whole walk, no allocation
    /// empty: yields nothing when `V` is empty
    /// order: increasing positions of `V`
    /// traps: never
    /// example: `var d = DsuOf<0..3, Int>(init: v => v, join: (x, y) => x + y); d.unite(2, 0); d.roots().collect() == [1, 2]`
    pub fn roots(self) -> yields V = todo

    /// The value of each set, once each, lazily: one per root, in the order of
    /// [`roots`](DsuOf.roots). O(1) to make.
    ///
    /// cost: O(n) for the whole walk, no allocation
    /// empty: yields nothing when `V` is empty
    /// order: by representative, increasing positions of `V`
    /// traps: never
    /// example: `var d = DsuOf<0..3, Int>(init: v => v + 1, join: (x, y) => x + y); d.unite(2, 0); d.values.collect() == [2, 4]`
    pub @get fn values(self) -> yields T = todo

    /// The present, to return to with [`undo_to`](DsuOf.undo_to). The first call switches path
    /// shortening off for good.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var d = DsuOf<0..2, Int>(init: v => v, join: (x, y) => x + y); let m = d.mark(); d.unite(0, 1); d.undo_to(m); d.get(0) == 0`
    pub fn mark(mut self) -> Mark = todo

    /// Undoes every merge and every `set` made since `snap = mark()`, newest first, restoring the
    /// sets and their values exactly.
    ///
    /// cost: O(1) per undone change
    /// traps: when `snap` lies after the present (its changes were already undone)
    /// example: `var d = DsuOf<0..2, Int>(init: v => v, join: (x, y) => x + y); let m = d.mark(); d.unite(0, 1); d.set(0, 9); d.undo_to(m); d.get(0) == 0 and d.get(1) == 1`
    pub fn undo_to(mut self, snap: Mark) = todo

    /// Every set as a list of its positions, as `Dsu.groups`; values are read with `get`.
    ///
    /// cost: O(n alpha(n)); allocates the lists
    /// empty: `[]` when `V` is empty
    /// order: lists by their least position, positions increasing within each
    /// traps: never
    /// example: `var d = DsuOf<0..3, Int>(init: v => v, join: (x, y) => x + y); d.unite(2, 0); d.groups() == [[0, 2], [1]]`
    pub fn groups(mut self) -> List<List<V>> = todo

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

    /// A parity union-find over `V` with every position alone and nothing known.
    ///
    /// cost: O(n); three `[V]` arrays
    /// traps: never
    /// example: `let d = ParityDsu<0..3>(); d.consistent and d.diff(0, 1) == None`
    pub fn new() -> ParityDsu<V> = todo

    /// The representative of `v`'s set and whether `v` is on the other side from it.
    ///
    /// cost: O(log n)
    /// ties: none; the representative is fixed by the merges (larger set's root, `a`'s root on equal sizes)
    /// traps: never
    /// example: `var d = ParityDsu<0..3>(); d.unite(0, 1); d.find(1) == (root: 0, flip: true)`
    /// example: `let d = ParityDsu<0..3>(); d.find(2) == (root: 2, flip: false)`
    pub fn find(self, v: V) -> (root: V, flip: Bool) = todo

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
    pub fn unite(mut self, a: V, b: V, differ: Bool = true) -> Bool = todo

    /// Whether `a` and `b` are on opposite sides; `None` when nothing relates them yet.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `var d = ParityDsu<0..3>(); d.unite(0, 1); d.unite(1, 2); d.diff(0, 2) == Some(false) and d.diff(1, 0) == Some(true)`
    /// example: `ParityDsu<0..3>().diff(0, 2) == None`
    pub fn diff(self, a: V, b: V) -> ?Bool = todo

    /// Whether `a` and `b` are in the same set, so that their relative side is known.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `var d = ParityDsu<0..3>(); d.unite(0, 1); d.same(1, 0) and not d.same(0, 2)`
    pub fn same(self, a: V, b: V) -> Bool = todo

    /// The number of positions in `v`'s set, `v` included.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `var d = ParityDsu<0..3>(); d.unite(0, 1); d.unite(1, 2); d.size(2) == 3`
    pub fn size(self, v: V) -> Int = todo

    /// Whether no contradiction is in effect: every `unite` since the last undone one agreed with
    /// those before it. For a graph: is it two-colourable?
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var d = ParityDsu<0..3>(); d.unite(0, 1); d.unite(1, 2); d.unite(2, 0); not d.consistent`
    pub @get fn consistent(self) -> Bool = todo

    /// The present, to return to with [`undo_to`](ParityDsu.undo_to). Paths are never shortened,
    /// so marking changes nothing else.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var d = ParityDsu<0..2>(); let m = d.mark(); d.unite(0, 1); d.undo_to(m); d.diff(0, 1) == None`
    pub fn mark(self) -> Mark = todo

    /// Takes back every `unite` since `snap = mark()`, contradictions included, newest first.
    ///
    /// cost: O(1) per undone `unite`
    /// traps: when `snap` lies after the present (its changes were already undone)
    /// example: `var d = ParityDsu<0..3>(); d.unite(0, 1); d.unite(1, 2); let m = d.mark(); d.unite(2, 0); d.undo_to(m); d.consistent`
    pub fn undo_to(mut self, snap: Mark) = todo

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

    /// A potential union-find over `V` with every position alone at potential 0. Offsets are
    /// `Int` and combine with `+`.
    ///
    /// cost: O(n); three `[V]` arrays
    /// traps: never
    /// example: `let d = PotentialDsu<0..3>(); d.components == 3 and d.consistent`
    pub fn new() -> PotentialDsu<V> = todo

    /// The representative of `v`'s set and `val[v] - val[root]`.
    ///
    /// cost: O(log n)
    /// ties: none; the representative is fixed by the merges (larger set's root, `a`'s root on equal sizes)
    /// traps: never
    /// example: `var d = PotentialDsu<0..3>(); d.unite(0, 1, 5); d.find(1) == (root: 0, offset: 5)`
    /// example: `let d = PotentialDsu<0..3>(); d.find(2) == (root: 2, offset: 0)`
    pub fn find(self, v: V) -> (root: V, offset: Int) = todo

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
    pub fn unite(mut self, a: V, b: V, w: Int) -> Bool = todo

    /// `val[b] - val[a]` when `a` and `b` are in the same set, else `None`.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `var d = PotentialDsu<0..3>(); d.unite(0, 1, 5); d.unite(1, 2, 3); d.diff(0, 2) == Some(8) and d.diff(2, 0) == Some(-8)`
    /// example: `PotentialDsu<0..3>().diff(0, 1) == None`
    pub fn diff(self, a: V, b: V) -> ?Int = todo

    /// Whether `a` and `b` are in the same set, so that their difference is known.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `var d = PotentialDsu<0..3>(); d.unite(2, 0, 1); d.same(0, 2) and not d.same(0, 1)`
    pub fn same(self, a: V, b: V) -> Bool = todo

    /// The number of positions in `v`'s set, `v` included.
    ///
    /// cost: O(log n)
    /// traps: never
    /// example: `var d = PotentialDsu<0..3>(); d.unite(2, 0, 1); d.size(0) == 2`
    pub fn size(self, v: V) -> Int = todo

    /// The number of sets.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var d = PotentialDsu<0..3>(); d.unite(0, 1, 5); d.components == 2`
    pub @get fn components(self) -> Int = todo

    /// Whether no contradiction is in effect.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var d = PotentialDsu<0..2>(); d.unite(0, 1, 5); not d.unite(1, 0, 5) and not d.consistent`
    pub @get fn consistent(self) -> Bool = todo

    /// The present, to return to with [`undo_to`](PotentialDsu.undo_to).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var d = PotentialDsu<0..2>(); let m = d.mark(); d.unite(0, 1, 5); d.undo_to(m); d.diff(0, 1) == None`
    pub fn mark(self) -> Mark = todo

    /// Takes back every `unite` since `snap = mark()`, contradictions included, newest first.
    ///
    /// cost: O(1) per undone `unite`
    /// traps: when `snap` lies after the present (its changes were already undone)
    /// example: `var d = PotentialDsu<0..2>(); d.unite(0, 1, 5); let m = d.mark(); d.unite(1, 0, 5); d.undo_to(m); d.consistent and d.diff(0, 1) == Some(5)`
    pub fn undo_to(mut self, snap: Mark) = todo

/// Domain-indexed array `[D]T` with `Mark` rollback.
/// `get`, `set`, and `mark` are O(1); `undo_to` is O(1) per undone write.
pub struct RollbackArray<D, T>:
    data: [D]T
    log: List<(D, T)>

    /// An array over `D` holding `init(i)` at each `i`, that can step back to any mark.
    ///
    /// cost: O(n) calls of `init`, for n = `D.count`
    /// traps: never
    /// example: `let a = RollbackArray<0..3, Int>(init: i => i * i); a.get(2) == 4`
    pub fn new(init: D -> T) -> RollbackArray<D, T> = todo

    /// The current value at `i`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var a = RollbackArray<0..3, Int>(init: i => i); a.set(1, 7); a.get(1) == 7 and a.get(2) == 2`
    pub fn get(self, i: D) -> T = todo

    /// Sets `i` to `val`, logging the previous value so `undo_to` can restore it. Every `set` is
    /// logged, marked or not.
    ///
    /// cost: O(1), plus one log entry
    /// traps: never
    /// example: `var a = RollbackArray<0..2, Int>(init: _ => 0); a.set(0, 5); a.get(0) == 5`
    pub fn set(mut self, i: D, val: T) = todo

    /// The present, to return to with [`undo_to`](RollbackArray.undo_to).
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var a = RollbackArray<0..2, Int>(init: _ => 0); let m = a.mark(); a.set(1, 3); a.undo_to(m); a.get(1) == 0`
    pub fn mark(self) -> Mark = todo

    /// Reverts every `set` since `snap = mark()`, newest first, so each position holds what it held
    /// at the mark.
    ///
    /// cost: O(1) per undone `set`
    /// traps: when `snap` lies after the present (its changes were already undone)
    /// example: `var a = RollbackArray<0..2, Int>(init: _ => 0); let m = a.mark(); a.set(0, 5); a.set(0, 6); a.undo_to(m); a.get(0) == 0`
    pub fn undo_to(mut self, snap: Mark) = todo

/// Path-copying persistent disjoint-set union over domain `V`.
/// Every version remains readable; `find`, `unite`, `same`, and `size` run in O(log^2 n).
pub struct PersistentDsu<V>:
    _parts: Int

    /// A persistent union-find over `V` holding version 0, every position in its own set.
    ///
    /// cost: O(n)
    /// traps: never
    /// example: `let d = PersistentDsu<0..3>(); not d.same(d.base, 0, 1)`
    pub fn new() -> PersistentDsu<V> = todo

    /// The handle of version 0, where every position is alone.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `PersistentDsu<0..3>().base == 0`
    pub @get fn base(self) -> Int = todo

    /// The representative of `v`'s set in version `ver`, chosen by union by size as in
    /// [`Dsu.find`](Dsu.find).
    ///
    /// cost: O(log² n)
    /// ties: none; the representative is fixed by the merges (larger set's root, `a`'s root on equal sizes)
    /// traps: when `ver` is not a version handle of `self`
    /// example: `var d = PersistentDsu<0..3>(); let v1 = d.unite(d.base, 2, 1); d.find(v1, 1) == 2 and d.find(d.base, 1) == 1`
    pub fn find(self, ver: Int, v: V) -> V = todo

    /// Whether `a` and `b` are in the same set in version `ver`.
    ///
    /// cost: O(log² n)
    /// traps: when `ver` is not a version handle of `self`
    /// example: `var d = PersistentDsu<0..3>(); let v1 = d.unite(d.base, 0, 1); d.same(v1, 0, 1) and not d.same(d.base, 0, 1)`
    pub fn same(self, ver: Int, a: V, b: V) -> Bool = todo

    /// The number of positions in `v`'s set in version `ver`, `v` included.
    ///
    /// cost: O(log² n)
    /// traps: when `ver` is not a version handle of `self`
    /// example: `var d = PersistentDsu<0..3>(); let v1 = d.unite(d.base, 0, 1); d.size(v1, 1) == 2 and d.size(d.base, 1) == 1`
    pub fn size(self, ver: Int, v: V) -> Int = todo

    /// A new version equal to `ver` with the sets of `a` and `b` merged; `ver` itself is unchanged.
    ///
    /// Handles are numbered in creation order: the k-th call returns `k`. A call whose `a` and `b`
    /// are already one set in `ver` still returns a fresh handle, equal in content to `ver`. Merging
    /// keeps the larger set's root, `a`'s root on equal sizes.
    ///
    /// cost: O(log² n) time and O(log n) memory
    /// traps: when `ver` is not a version handle of `self`
    /// example: `var d = PersistentDsu<0..3>(); let v1 = d.unite(d.base, 0, 1); let v2 = d.unite(d.base, 1, 2); v1 == 1 and v2 == 2 and d.same(v2, 1, 2) and not d.same(v2, 0, 1)`
    pub fn unite(mut self, ver: Int, a: V, b: V) -> Int = todo

    /// Preallocates room for at least `nodes` persistent-array nodes, so later `unite`s do not
    /// grow the pool. Changes no version.
    ///
    /// cost: O(nodes)
    /// traps: when `nodes < 0`
    /// example: `var d = PersistentDsu<0..3>(); d.reserve(100); d.size(d.base, 0) == 1`
    pub fn reserve(mut self, nodes: Int) = todo

@test
fn potential_dsu_and_rollback_array() = todo

@test
fn unite_and_find() = todo

@test
fn agrees_with_labels_on_random_unions() = todo

@test
fn undo_to_returns_to_every_mark() = todo

@test
fn groups_partitions_all_vertices_by_component() = todo

@test
fn merge_says_which_root_stayed() = todo

@test
fn values_follow_merges_and_roll_back() = todo

@test
fn a_directional_join_keeps_the_second_value() = todo

@test
fn parity_matches_a_brute_colouring_and_rolls_back() = todo

@test
fn roots_and_values_yield_one_entry_per_component() = todo
