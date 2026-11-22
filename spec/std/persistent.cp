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

    /// Build version 0 from `a:`, a uniform `fill:`, or the monoid identity. O(n) when `a:` or `fill:` is given, O(1) otherwise.
    pub fn new<M>(op: M, a: ?[D]T = none, fill: ?T = none) -> PersistentSegTree<D, T> = todo

    fn push_node(mut self, l: Int, r: Int, v: T) -> Int = todo

    fn build_list(mut self, vals: List<T>, nl: Int, nr: Int) -> Int = todo

    /// The initial version created by `new`. O(1).
    pub @get fn base(self) -> Version = todo

    fn set_rec(mut self, u: Int, nl: Int, nr: Int, idx: Int, x: T) -> Int = todo

    /// Return a new `Version` equal to `v` with position `i` set to `x`. O(log n) time and memory.
    pub fn set(mut self, v: Version, i: D, x: T) -> Version = todo

    fn update_rec(mut self, u: Int, nl: Int, nr: Int, idx: Int, f: T -> T) -> Int = todo

    /// Return a new `Version` equal to `v` with position `i` replaced by `f(current)`. O(log n) time and memory.
    pub fn update(mut self, v: Version, i: D, f: T -> T) -> Version = todo

    /// Element at position `i` in version `v`. O(log n).
    pub fn get(self, v: Version, i: D) -> T = todo

    fn fold_rec(self, u: Int, nl: Int, nr: Int, ql: Int, qr: Int) -> T = todo

    /// Fold `r` in version `v`. O(log n).
    pub fn fold(self, v: Version, r: Range<D>) -> T = todo

    /// Fold the entire domain in version `v`. O(1).
    pub fn all(self, v: Version) -> T = todo

    /// O(nodes). Preallocate arena capacity for `nodes` tree nodes.
    pub fn reserve(mut self, nodes: Int) = todo

    /// O(log |D|). Furthest boundary `b` right of `from` in version `v` such that `ok(fold(v, from..b))`.
    pub fn right_while(self, v: Version, from: Cut<D>, ok: T -> Bool) -> Cut<D> = todo

    /// O(log |D|). Furthest boundary `b` left of `to` in version `v` such that `ok(fold(v, b..to))`.
    pub fn left_while(self, v: Version, to: Cut<D>, ok: T -> Bool) -> Cut<D> = todo

    /// O(log |D|). Walk the difference of two count-tree versions (`hi - lo`) to find the `k`-th
    /// position (1-based), or `none` when fewer than `k` items exist.
    pub fn diff_kth(self, hi: Version, lo: Version, k: Int) -> ?D = todo

/// Static range k-th smallest and range rank queries over `[D]T`.
/// Built in O(n log n) using prefix versions of a persistent count tree; queries run in O(log n).
pub struct RangeKth<D, T>:
    vals: List<T>
    left: List<0..=2_000_000_000>
    right: List<0..=2_000_000_000>
    cnt: List<0..=2_000_000_000>
    roots: List<0..=2_000_000_000>

    /// Build from `a: [D]T`. O(n log n) time and memory.
    pub fn new(a: [D]T) -> RangeKth<D, T> = todo

    fn lower_val(self, x: T) -> Int = todo

    fn upper_val(self, x: T) -> Int = todo

    fn inc(mut self, root: Int, m: Int, pos: Int) -> Int = todo

    /// The `k`-th smallest element (0-based) in `r`. Traps if `k < 0` or `k >= r.len`. O(log n).
    pub fn kth(self, r: Range<D>, k: Int) -> T = todo

    fn count_prefix(self, u: Int, v: Int, nl: Int, nr: Int, limit: Int) -> Int = todo

    /// Count elements in `r` less than or equal to `x`. O(log n).
    pub fn count_le(self, r: Range<D>, x: T) -> Int = todo

    /// O(log |D|). Count elements in `r` strictly less than `x`.
    pub fn count_lt(self, r: Range<D>, x: T) -> Int = todo

    /// O(log |D|). Count elements in `r` within `[lo_v, hi_v]` inclusive.
    pub fn count_in(self, r: Range<D>, lo_v: T, hi_v: T) -> Int = todo

/// O(q log |D|) space. Path-copying persistent segment tree with lazy range updates:
/// every `apply`/`set` returns a new `Version` in O(log |D|) time and memory.
pub struct PersistentLazySegTree<D, T, F>(act: Action<T, F>):
    base_root: Int

    /// O(|D|) with `a`/`fill`, O(1) otherwise. Create version 0 over `D`.
    pub fn new(act: Action<T, F>, a: ?[D]T = none, fill: ?T = none) -> PersistentLazySegTree<D, T, F> = todo

    /// O(1). The initial version created by `new`.
    pub @get fn base(self) -> Version = todo

    /// O(log |D|) time and memory. Return a new `Version` applying `f` across `r` in version `v`.
    pub fn apply(mut self, v: Version, r: Range<D>, f: F) -> Version = todo

    /// O(log |D|) time and memory. Return a new `Version` setting position `i` to `x` in version `v`.
    pub fn set(mut self, v: Version, i: D, x: T) -> Version = todo

    /// O(log |D|). Fold range `r` in version `v`.
    pub fn fold(self, v: Version, r: Range<D>) -> T = todo

    /// O(log |D|). Read position `i` in version `v`.
    pub fn get(self, v: Version, i: D) -> T = todo

    /// O(1). Fold the entire domain in version `v`.
    pub fn all(self, v: Version) -> T = todo

/// O(q log |D|) space. Persistent array over domain `D`: O(log |D|) `get` and path-copying `set`.
pub struct PersistentArray<D, T>:
    base_root: Int

    /// O(|D|). Build version 0 from `a` or uniform `fill`.
    pub fn new(a: ?[D]T = none, fill: ?T = none) -> PersistentArray<D, T> = todo

    /// O(1). Initial version.
    pub @get fn base(self) -> Version = todo

    /// O(log |D|). Read position `i` in version `v`.
    pub fn get(self, v: Version, i: D) -> T = todo

    /// O(log |D|) time and memory. Return a new `Version` with position `i` set to `x`.
    pub fn set(mut self, v: Version, i: D, x: T) -> Version = todo

/// O(log |D|) per update. Hashed persistent multiset over domain `D`:
/// O(1) equality, O(log |D|) lexicographic comparison (`desc: true` compares largest element first).
pub struct PMultiset<D>:
    root: Int

    /// O(1). The empty persistent multiset over `D`.
    pub fn empty() -> PMultiset<D> = todo

    /// O(log |D|) time and memory. Return a new multiset with one copy of `x` inserted.
    pub fn with(self, x: D) -> PMultiset<D> = todo

    /// O(log |D|) time and memory. Return a new multiset with one copy of `x` removed (if present).
    pub fn without(self, x: D) -> PMultiset<D> = todo

    /// O(log |D|). Multiplicity of `x` in this multiset.
    pub fn count(self, x: D) -> Int = todo

    /// O(log |D|). Lexicographic comparison against `other` (`desc: true` compares largest key first),
    /// returning `-1`, `0`, or `1`.
    pub fn cmp(self, other: PMultiset<D>, desc: Bool = false) -> Int = todo

@test
fn persistent_segtree_and_range_kth_match_brute() = todo
