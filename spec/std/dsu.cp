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

    /// Every position starts in its own set; pass `rollback: true` to enable rollback up front (or call `mark()` lazily). O(n).
    pub fn new(rollback: Bool = false) -> Dsu<V> = todo

    fn find_ord(self, start: Int) -> Int = todo

    /// The representative of `v`'s set. Shortens paths through `@cache`, so a read-only
    /// `Dsu` answers it too.
    pub fn find(self, v: V) -> V = todo

    /// The present, to return to with `undo_to`. O(1).
    pub fn mark(mut self) -> Mark = todo

    /// Undo every `unite` made since `snap = mark()`. O(1) per undone `unite`.
    pub fn undo_to(mut self, snap: Mark) = todo

    /// Merge the sets of `a` and `b`. Returns false if they were already one set.
    pub fn unite(mut self, a: V, b: V) -> Bool = todo

    /// As `unite`, but says which root stayed and which went under it, so data kept per
    /// set can follow: `if let (kept, merged) = d.merge(a, b): sum[kept] += sum[merged]`.
    /// `none` if `a` and `b` were already one set.
    pub fn merge(mut self, a: V, b: V) -> ?(kept: V, merged: V) = todo

    /// Whether `a` and `b` are in the same component. O(alpha(n)) amortized.
    pub fn same(mut self, a: V, b: V) -> Bool = todo

    /// The number of positions in `v`'s set.
    pub fn size(mut self, v: V) -> Int = todo

    /// The number of sets.
    pub @get fn components(self) -> Int = todo

    /// Yield the representative root of each connected component. O(n), zero allocations.
    pub fn roots(self) -> yields V = todo

    /// Every connected component as a list of its vertices, each vertex appearing once. O(n alpha(n)).
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

    /// Every position starts in its own set, with value `init(v)`. O(n).
    pub fn new(init: (V) -> T, join: (T, T) -> T) -> DsuOf<V, T> = todo

    /// The representative of `v`'s set. Shortens paths through `@cache`, so a read-only
    /// `Dsu` answers it too.
    pub fn find(self, v: V) -> V = todo

    /// The value of `v`'s set.
    pub fn get(mut self, v: V) -> T = todo

    /// Replace the value of `v`'s set.
    pub fn set(mut self, v: V, x: T) = todo

    /// Merge the sets of `a` and `b`, joining their values. Returns false if they were
    /// already one set (the value is left alone then).
    pub fn unite(mut self, a: V, b: V) -> Bool = todo

    /// As `unite`, but says which root stayed and which went under it. `none` if `a` and
    /// `b` were already one set.
    pub fn merge(mut self, a: V, b: V) -> ?(kept: V, merged: V) = todo

    /// Whether `a` and `b` are in the same component. O(alpha(n)) amortized.
    pub fn same(mut self, a: V, b: V) -> Bool = todo

    /// The number of positions in `v`'s set.
    pub fn size(mut self, v: V) -> Int = todo

    /// The number of sets.
    pub @get fn components(self) -> Int = todo

    /// Yield the representative root of each connected component. O(n), zero allocations.
    pub fn roots(self) -> yields V = todo

    /// Yield each connected component's payload once (one per root). O(n), zero allocations.
    pub fn values(self) -> yields T = todo

    /// The present, to return to with `undo_to`. O(1).
    pub fn mark(mut self) -> Mark = todo

    /// Undo every merge and `set` made since `snap = mark()`. O(1) each.
    pub fn undo_to(mut self, snap: Mark) = todo

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
    pub fn new() -> ParityDsu<V> = todo

    /// The representative of `v`'s set and whether `v` is on the other side from it.
    pub fn find(self, v: V) -> (root: V, flip: Bool) = todo

    /// Record that `a` and `b` are on opposite sides (`differ: false`: the same side).
    /// `false` if that contradicts what is already known; the contradiction is counted, so
    /// `consistent` turns false until it is undone. An edge of a graph is `unite(u, v)`.
    pub fn unite(mut self, a: V, b: V, differ: Bool = true) -> Bool = todo

    /// Are `a` and `b` on opposite sides? `none` when nothing relates them yet.
    pub fn diff(self, a: V, b: V) -> ?Bool = todo

    /// Whether `a` and `b` are in the same component. O(log n).
    pub fn same(self, a: V, b: V) -> Bool = todo

    /// The number of positions in `v`'s set.
    pub fn size(self, v: V) -> Int = todo

    /// Has every `unite` so far agreed with the ones before it? For a graph: is it
    /// two-colourable?
    pub @get fn consistent(self) -> Bool = todo

    /// The present, to return to with `undo_to`. O(1).
    pub fn mark(self) -> Mark = todo

    /// Take back every `unite` since `snap = mark()`, contradictions included.
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

    /// [rename] Every position starts in its own set with identity potential, combined by W10 group operation `op` (default `+`; replaces `GroupPotentialDsu`). O(n).
    pub fn new<G>(op: ?G = none) -> PotentialDsu<V> = todo

    /// Root of `v`'s set and `val[v] - val[root]`. O(log n).
    pub fn find(self, v: V) -> (root: V, offset: Int) = todo

    /// Enforce `val[b] - val[a] == w`. Returns `false` and increments conflicts on contradiction. O(log n).
    pub fn unite(mut self, a: V, b: V, w: Int) -> Bool = todo

    /// `val[b] - val[a]` if `a` and `b` are in the same component, else `none`. O(log n).
    pub fn diff(self, a: V, b: V) -> ?Int = todo

    /// Whether `a` and `b` are in the same component. O(log n).
    pub fn same(self, a: V, b: V) -> Bool = todo

    /// Number of vertices in `v`'s component. O(log n).
    pub fn size(self, v: V) -> Int = todo

    /// Number of connected components. O(1).
    pub @get fn components(self) -> Int = todo

    /// Whether no contradicting constraints are currently active. O(1).
    pub @get fn consistent(self) -> Bool = todo

    /// Snapshot the current state for `undo_to`. O(1).
    pub fn mark(self) -> Mark = todo

    /// Revert every `unite` since `snap = mark()`. O(1) per undone step.
    pub fn undo_to(mut self, snap: Mark) = todo

/// Domain-indexed array `[D]T` with `Mark` rollback.
/// `get`, `set`, and `mark` are O(1); `undo_to` is O(1) per undone write.
pub struct RollbackArray<D, T>:
    data: [D]T
    log: List<(D, T)>

    /// Build a rollback array initialized by `init(i)` for each `i` in `D`. O(D.K).
    pub fn new(init: D -> T) -> RollbackArray<D, T> = todo

    /// Read the current value at `i`. O(1).
    pub fn get(self, i: D) -> T = todo

    /// Set `i` to `val`, recording the previous value for rollback. O(1).
    pub fn set(mut self, i: D, val: T) = todo

    /// Snapshot the current state for `undo_to`. O(1).
    pub fn mark(self) -> Mark = todo

    /// Revert every `set` since `snap = mark()`. O(1) per undone write.
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
