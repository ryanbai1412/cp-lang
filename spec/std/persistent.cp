//! Persistent segment tree and static range k-th smallest queries.
//!
//! Each update copies only the O(log n) nodes on the root-to-leaf path and returns a
//! new `Version` handle. All earlier versions remain readable in O(log n).
use std.algebra.{Action, Monoid}
use std.seq.{deduped, ends}

/// Opaque version handle for `PersistentSegTree`.
pub struct Version(root: Int)

/// Path-copying persistent segment tree over a domain `D` and monoid `Monoid<T>`.
/// `set` and `update` return a new `Version` in O(log n) time and memory; `fold` and `get` take O(log n).
pub struct PersistentSegTree<D, T>(m: Monoid<T>):
    left: List<0..=2_000_000_000>
    right: List<0..=2_000_000_000>
    val: List<T>
    base_root: Int

    /// Build version 0 over `D` from `a:`, a uniform `fill:`, or the monoid identity everywhere, and
    /// keep it as `base`. Every later write returns a new `Version`; no version is ever changed.
    ///
    /// cost: O(n) time and memory with `a:` or `fill:`, O(1) otherwise, for n = `|D|`
    /// empty: an empty `D` gives a tree whose every fold is the identity
    /// traps: when both `a:` and `fill:` are given
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); t.all(t.base) == 6`
    pub fn new<M>(op: M, a: ?[D]T = None, fill: ?T = None) -> PersistentSegTree<D, T> = todo

    fn push_node(mut self, l: Int, r: Int, v: T) -> Int = todo

    fn build_list(mut self, vals: List<T>, nl: Int, nr: Int) -> Int = todo

    /// The version built by `new`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); t.get(t.base, 0) == 1`
    @get pub fn base(self) -> Version = todo

    fn set_rec(mut self, u: Int, nl: Int, nr: Int, idx: Int, x: T) -> Int = todo

    /// A new version equal to `v` with position `i` set to `x`. `v` and every other version are
    /// unchanged; the new version shares all but the O(log n) copied nodes with `v`.
    ///
    /// cost: O(log n) time and memory
    /// traps: when `i` is not in `D`, or `v` did not come from this tree
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); let v = t.set(t.base, 1, 10); t.all(v) == 14 and t.all(t.base) == 6`
    pub fn set(mut self, v: Version, i: D, x: T) -> Version = todo

    fn update_rec(mut self, u: Int, nl: Int, nr: Int, idx: Int, f: T -> T) -> Int = todo

    /// A new version equal to `v` with position `i` replaced by `f` of its value in `v`. `f` is called
    /// once.
    ///
    /// cost: O(log n) time and memory, one call of `f`
    /// traps: when `i` is not in `D`, or `v` did not come from this tree
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); let v = t.update(t.base, 2, x => x * 5); t.get(v, 2) == 15 and t.get(t.base, 2) == 3`
    pub fn update(mut self, v: Version, i: D, f: T -> T) -> Version = todo

    /// The value at position `i` in version `v`.
    ///
    /// cost: O(log n)
    /// traps: when `i` is not in `D`, or `v` did not come from this tree
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); t.get(t.base, 1) == 2`
    pub fn get(self, v: Version, i: D) -> T = todo

    fn fold_rec(self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int) -> T = todo

    /// The fold of `r` in version `v`, left to right, so a non-commutative monoid sees its elements in
    /// position order.
    ///
    /// cost: O(log n)
    /// empty: the monoid identity
    /// traps: when a non-empty `r` reaches outside `D`, or `v` did not come from this tree
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); t.fold(t.base, 1..3) == 5`
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); t.fold(t.base, 2..2) == 0`
    pub fn fold(self, v: Version, r: Range<D>) -> T = todo

    /// The fold of every position in version `v`, read from its root.
    ///
    /// cost: O(1)
    /// empty: the monoid identity when `D` is empty
    /// traps: when `v` did not come from this tree
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); let v = t.set(t.base, 0, 0); t.all(v) == 5`
    pub fn all(self, v: Version) -> T = todo

    /// Preallocate room for `nodes` tree nodes in total, so later writes do not regrow the arena.
    /// Changes no version.
    ///
    /// cost: O(nodes)
    /// traps: when `nodes < 0`
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); t.reserve(100); t.all(t.base) == 6`
    pub fn reserve(mut self, nodes: Int) = todo

    /// The rightmost cut `b` at or after `from` such that `ok(fold(v, from..b))` holds, reading
    /// version `v`.
    ///
    /// Law: `ok` is monotone (once false, false for every longer range) and `ok(identity)` holds.
    ///
    /// cost: O(log n) calls of `ok`
    /// ties: the rightmost qualifying cut
    /// traps: when `from` is not a cut of `D`, when `ok(identity)` is false, or `v` did not come from this tree
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); t.right_while(t.base, 0, s => s <= 3) == 2`
    pub fn right_while(self, v: Version, from: Cut<D>, ok: T -> Bool) -> Cut<D> = todo

    /// The leftmost cut `b` at or before `to` such that `ok(fold(v, b..to))` holds, reading version
    /// `v`.
    ///
    /// Law: `ok` is monotone (once false, false for every longer range) and `ok(identity)` holds.
    ///
    /// cost: O(log n) calls of `ok`
    /// ties: the leftmost qualifying cut
    /// traps: when `to` is not a cut of `D`, when `ok(identity)` is false, or `v` did not come from this tree
    /// example: `var t = PersistentSegTree(+, a: [1, 2, 3]); t.left_while(t.base, 3, s => s <= 5) == 1`
    pub fn left_while(self, v: Version, to: Cut<D>, ok: T -> Bool) -> Cut<D> = todo

    /// For a tree of counts under `+`, the position of the `k`-th item (1-based) in the difference
    /// `hi - lo` (0-based, like `RangeKth.kth`), walking both versions at once; `None` when the difference holds at most `k` items.
    /// A position with count `c` holds `c` items.
    ///
    /// Law: `hi` was derived from `lo` by adding counts, so no position of `hi - lo` is negative.
    ///
    /// cost: O(log n)
    /// ties: items at one position are interchangeable; the position is returned
    /// traps: when `k < 0`, or a version did not come from this tree
    /// example: `var t = PersistentSegTree(+, a: [0, 0, 0, 0]); let v1 = t.set(t.base, 1, 1); let v2 = t.set(v1, 3, 2); t.diff_kth(v2, t.base, 1) == Some(3) and t.diff_kth(v2, t.base, 3) == None`
    /// example: `var t = PersistentSegTree(+, a: [0, 0, 0, 0]); let v1 = t.set(t.base, 1, 1); let v2 = t.set(v1, 3, 2); t.diff_kth(v2, v1, 0) == Some(3)`
    pub fn diff_kth(self, hi: Version, lo: Version, k: Int) -> ?D = todo

/// Static range k-th smallest and range rank queries over `[D]T`.
/// Built in O(n log n) using prefix versions of a persistent count tree; queries run in O(log n).
pub struct RangeKth<D, T>:
    vals: List<T>
    left: List<0..=2_000_000_000>
    right: List<0..=2_000_000_000>
    cnt: List<0..=2_000_000_000>
    roots: List<0..=2_000_000_000>

    /// Build the static structure over `a`: one persistent count tree over the sorted distinct values
    /// per prefix of `a`. `a` is copied; later changes to it are not seen.
    ///
    /// cost: O(n log n) time and memory for n = `a.len`
    /// empty: an empty `a` builds; every query range is then empty
    /// order: queries take ranges of `a`'s domain
    /// traps: never
    /// example: `RangeKth([5, 1, 4, 2]).kth(0..4, 0) == 1`
    pub fn new(a: [D]T) -> RangeKth<D, T> = todo

    fn lower_val(self, x: T) -> Int = todo

    fn upper_val(self, x: T) -> Int = todo

    fn inc(mut self, root: Int, m: Int, pos: Int) -> Int = todo

    /// The `k`-th smallest element (0-based) of `a` over `r`, counting repeats.
    ///
    /// cost: O(log n)
    /// empty: traps (no `k` is in range)
    /// ties: equal values are indistinguishable; the value is returned
    /// traps: when `k < 0` or `k >= r.len`, or `r` reaches outside `D`
    /// example: `RangeKth([5, 1, 4, 2]).kth(1..4, 1) == 2`
    pub fn kth(self, r: Range<D>, k: Int) -> T = todo

    fn count_prefix(self, u: Int, v: Int, nl: Int, nr: Int, limit: Int) -> Int = todo

    /// The number of elements of `a` over `r` that are at most `x`.
    ///
    /// cost: O(log n)
    /// empty: 0
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `RangeKth([5, 1, 4, 2]).count_le(0..4, 4) == 3`
    pub fn count_le(self, r: Range<D>, x: T) -> Int = todo

    /// The number of elements of `a` over `r` that are less than `x`.
    ///
    /// cost: O(log n)
    /// empty: 0
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `RangeKth([5, 1, 4, 2]).count_lt(0..4, 4) == 2`
    pub fn count_lt(self, r: Range<D>, x: T) -> Int = todo

    /// The number of elements of `a` over `r` in `lo_v ..= hi_v` (both ends inclusive); 0 when
    /// `lo_v > hi_v`.
    ///
    /// cost: O(log n)
    /// empty: 0
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `RangeKth([5, 1, 4, 2]).count_in(0..4, 2, 5) == 3`
    /// example: `RangeKth([5, 1, 4, 2]).count_in(0..4, 5, 2) == 0`
    pub fn count_in(self, r: Range<D>, lo_v: T, hi_v: T) -> Int = todo

/// O(q log |D|) space. Path-copying persistent segment tree with lazy range updates:
/// every `apply`/`set` returns a new `Version` in O(log |D|) time and memory.
pub struct PersistentLazySegTree<D, T, F>(act: Action<T, F>):
    base_root: Int

    /// Build version 0 over `D` from `a:`, a uniform `fill:`, or the identity of `act`'s monoid, and
    /// keep it as `base`. Range writes copy O(log n) nodes and their pending tags, never changing an
    /// older version.
    ///
    /// cost: O(n) time and memory with `a:` or `fill:`, O(1) otherwise, for n = `|D|`
    /// empty: an empty `D` gives a tree whose every fold is the identity
    /// traps: when both `a:` and `fill:` are given
    /// example: `var t = PersistentLazySegTree(Action<Int, Int>(apply: (d, x, span) => x + d * span), a: [1, 2, 3]); t.all(t.base) == 6`
    pub fn new(act: Action<T, F>, a: ?[D]T = None, fill: ?T = None) -> PersistentLazySegTree<D, T, F> = todo

    /// The version built by `new`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var t = PersistentLazySegTree(Action<Int, Int>(apply: (d, x, span) => x + d * span), a: [1, 2, 3]); t.get(t.base, 2) == 3`
    @get pub fn base(self) -> Version = todo

    /// A new version equal to `v` with the tag `f` applied to every position of `r`.
    ///
    /// cost: O(log n) time and memory
    /// empty: an empty `r` returns a version equal to `v`
    /// traps: when a non-empty `r` reaches outside `D`, or `v` did not come from this tree
    /// example: `var t = PersistentLazySegTree(Action<Int, Int>(apply: (d, x, span) => x + d * span), a: [1, 2, 3]); let v = t.apply(t.base, 0..2, 10); t.all(v) == 26 and t.all(t.base) == 6`
    pub fn apply(mut self, v: Version, r: Range<D>, f: F) -> Version = todo

    /// A new version equal to `v` with position `i` set to `x`, discarding tags pending above it.
    ///
    /// cost: O(log n) time and memory
    /// traps: when `i` is not in `D`, or `v` did not come from this tree
    /// example: `var t = PersistentLazySegTree(Action<Int, Int>(apply: (d, x, span) => x + d * span), a: [1, 2, 3]); let v = t.apply(t.base, 0..3, 1); let w = t.set(v, 0, 0); t.all(w) == 7`
    pub fn set(mut self, v: Version, i: D, x: T) -> Version = todo

    /// The fold of `r` in version `v`, left to right, with every applied tag accounted for.
    ///
    /// cost: O(log n)
    /// empty: the monoid identity
    /// traps: when a non-empty `r` reaches outside `D`, or `v` did not come from this tree
    /// example: `var t = PersistentLazySegTree(Action<Int, Int>(apply: (d, x, span) => x + d * span), a: [1, 2, 3]); let v = t.apply(t.base, 1..3, 1); t.fold(v, 1..3) == 7`
    pub fn fold(self, v: Version, r: Range<D>) -> T = todo

    /// The value at position `i` in version `v`, with every applied tag accounted for.
    ///
    /// cost: O(log n)
    /// traps: when `i` is not in `D`, or `v` did not come from this tree
    /// example: `var t = PersistentLazySegTree(Action<Int, Int>(apply: (d, x, span) => x + d * span), a: [1, 2, 3]); let v = t.apply(t.base, 1..3, 1); t.get(v, 1) == 3 and t.get(t.base, 1) == 2`
    pub fn get(self, v: Version, i: D) -> T = todo

    /// The fold of every position in version `v`, read from its root.
    ///
    /// cost: O(1)
    /// empty: the monoid identity when `D` is empty
    /// traps: when `v` did not come from this tree
    /// example: `var t = PersistentLazySegTree(Action<Int, Int>(apply: (d, x, span) => x + d * span), a: [1, 2, 3]); t.all(t.apply(t.base, 0..3, 2)) == 12`
    pub fn all(self, v: Version) -> T = todo

/// O(q log |D|) space. Persistent array over domain `D`: O(log |D|) `get` and path-copying `set`.
pub struct PersistentArray<D, T>:
    base_root: Int

    /// Build version 0 over `D` from `a:` or a uniform `fill:`, and keep it as `base`.
    ///
    /// cost: O(n) time and memory for n = `|D|`
    /// empty: an empty `D` gives an array with no positions
    /// traps: when both or neither of `a:` and `fill:` are given
    /// example: `var p = PersistentArray(fill: 7); p.get(p.base, 0) == 7`
    pub fn new(a: ?[D]T = None, fill: ?T = None) -> PersistentArray<D, T> = todo

    /// The version built by `new`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var p = PersistentArray(a: [1, 2, 3]); p.get(p.base, 2) == 3`
    @get pub fn base(self) -> Version = todo

    /// The value at position `i` in version `v`.
    ///
    /// cost: O(log n)
    /// traps: when `i` is not in `D`, or `v` did not come from this array
    /// example: `var p = PersistentArray(a: [1, 2, 3]); p.get(p.base, 1) == 2`
    pub fn get(self, v: Version, i: D) -> T = todo

    /// A new version equal to `v` with position `i` set to `x`; `v` is unchanged.
    ///
    /// cost: O(log n) time and memory
    /// traps: when `i` is not in `D`, or `v` did not come from this array
    /// example: `var p = PersistentArray(a: [1, 2, 3]); let v = p.set(p.base, 0, 9); p.get(v, 0) == 9 and p.get(p.base, 0) == 1`
    pub fn set(mut self, v: Version, i: D, x: T) -> Version = todo

/// O(log |D|) per update. Hashed persistent multiset over domain `D`:
/// O(1) equality, O(log |D|) lexicographic comparison (`desc: true` compares largest element first).
pub struct PMultiset<D>:
    root: Int

    /// The empty multiset over `D`. Every multiset is an immutable value; `==` compares two in O(1)
    /// by hash.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `PMultiset<0..10>.empty().count(3) == 0`
    pub fn empty() -> PMultiset<D> = todo

    /// A new multiset with one more copy of `x`; `self` is unchanged.
    ///
    /// cost: O(log |D|) time and memory
    /// traps: when `x` is not in `D`
    /// example: `let s = PMultiset<0..10>.empty().with(3).with(3); s.count(3) == 2`
    pub fn with(self, x: D) -> PMultiset<D> = todo

    /// A new multiset with one copy of `x` removed, or equal to `self` when `x` is absent; `self` is
    /// unchanged.
    ///
    /// cost: O(log |D|) time and memory
    /// traps: when `x` is not in `D`
    /// example: `let s = PMultiset<0..10>.empty().with(3).with(3); s.without(3).count(3) == 1 and s.count(3) == 2`
    /// example: `let s = PMultiset<0..10>.empty(); s.without(3) == s`
    pub fn without(self, x: D) -> PMultiset<D> = todo

    /// The number of copies of `x`.
    ///
    /// cost: O(log |D|)
    /// traps: when `x` is not in `D`
    /// example: `PMultiset<0..10>.empty().with(4).count(4) == 1`
    pub fn count(self, x: D) -> Int = todo

    /// Compare the elements of `self` and `other` as sorted sequences, lexicographically: ascending,
    /// or largest first with `desc: true`. A proper prefix is smaller. Returns `-1`, `0` or `1`.
    ///
    /// The first differing key is found by hash in O(log |D|); `0` exactly when the multisets are
    /// equal.
    ///
    /// cost: O(log |D|)
    /// ties: `0` only for equal multisets
    /// traps: never
    /// example: `let e = PMultiset<0..10>.empty(); e.with(1).with(3).cmp(e.with(2)) == -1`
    /// example: `let e = PMultiset<0..10>.empty(); e.with(1).with(3).cmp(e.with(2), desc: true) == 1`
    /// example: `let e = PMultiset<0..10>.empty(); e.with(1).cmp(e.with(1).with(2)) == -1`
    pub fn cmp(self, other: PMultiset<D>, desc: Bool = false) -> Int = todo

@test
fn persistent_segtree_and_range_kth_match_brute() = todo
