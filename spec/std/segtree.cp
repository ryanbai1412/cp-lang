//! Segment tree over a domain, for any associative operation.
use std.algebra.{Monoid}
use std.math.{gcd}
use std.seq.{ends}

/// Point update and range fold. The operation is a bare operator (`+`, `*`, `^`, `&`, `|`,
/// `min`, `max`, `gcd`) or your own `Monoid(id, op)`: `SegTree<V, Int>(max, a)`,
/// `SegTree<V, Int>(+)`, `SegTree<V, Run>(Monoid(Run.empty, Run.join), a)`.
/// `set` and `fold` are O(log n), and so are `right_while` and `left_while`; memory is
/// 2 * bit_ceil(n) values. The fold keeps the order of the positions, so
/// non-commutative operations work.
pub struct SegTree<D, T>(m: Monoid<T>):
    t: List<T>
    size: Int
    /// What each position holds, for a tree built with `of`.
    source: ?(D -> T)

    /// A tree over `a`, filled with `fill`, or with the identity everywhere. O(n).
    /// With `act:` the tree also applies tags to ranges (`apply`, `apply_all`); with `over:`
    /// it covers a huge integer range and makes nodes on first touch. With `act:` and no op it
    /// applies tags and reads points. Push-down is skipped when the tags commute.
    pub fn new<M, A>(op: ?M = None, a: ?[D]T = None, fill: ?T = None, act: ?A = None, over: ?Range<Int> = None) -> SegTree<D, T> = todo

    /// Apply tag `f` to every position in `r`. O(log n).
    pub fn apply(mut self, r: Range<D>, f: F) = todo

    /// Apply tag `f` everywhere. O(1) amortized for commuting tags, else O(n).
    pub fn apply_all(mut self, f: F) = todo

    /// Fold of `r`, left to right. O(log n).
    pub fn fold(self, r: Range<D>) -> T = todo

    /// The last cut `d` with `ok(fold(from..d))`, `ok` monotone. O(log n).
    pub fn right_while(self, from: Cut<D>, ok: T -> Bool) -> Cut<D> = todo

    /// The first cut `d` with `ok(fold(d..to))`, `ok` monotone. O(log n).
    pub fn left_while(self, to: Cut<D>, ok: T -> Bool) -> Cut<D> = todo

    /// A tree whose position `i` holds `f(i)`. When something `f` reads changes, `refresh(i)`
    /// brings position `i` up to date, so the formula is written once, not at every update. O(n).
    pub fn of<M>(op: M, f: D -> T) -> SegTree<D, T> = todo

    /// Recompute position `i` from the formula given to `of`. O(log n).
    pub fn refresh(mut self, i: D) = todo

    /// Read the leaf value at position `i`. O(1).
    pub fn get(self, i: D) -> T = todo

    /// Assign leaf `i` to `x` and pull ancestors. O(log |D|).
    pub fn set(mut self, i: D, x: T) = todo

    /// Leaf `i` as a writable place: `st.at(i) += x`, `st.at(i) max= x`. The ancestors
    /// are pulled when the write ends, so each use is O(log |D|); read with `get`.
    pub fn at(mut self, i: D) lends mut T:
        lend mut t[size + i.to_idx]
        self.pull(i)

    fn pull(mut self, i: D) = todo

    /// Replace the value at `i` with `f` of it, as `apply` does on a point. O(log n).
    pub fn update(mut self, i: D, f: T -> T) = todo

    /// The fold of a range, left to right: `st.fold(l..=r)`, `st.fold(..i)`, `st.fold(i..)`.
    /// An empty range gives the identity.
    pub fn fold(self, r: Range<D>) -> T = todo

    /// The fold of the whole array. O(1).
    pub @get fn all(self) -> T = todo

    /// The operation the tree folds with, as its `(id, op)` pair.
    pub @get fn monoid(self) -> Monoid<T> = todo



    /// O(log |D|). Fold of `r`, or `None` when `r` is empty (avoids sentinel identities for semigroups).
    pub fn try_fold(self, r: Range<D>) -> ?T = todo

    /// O(|D|). Reset all leaves to `fill` (or `m.id` when `None`) without reallocating.
    pub fn clear(mut self, fill: ?T = None) = todo

    /// O(log |D|). Yield the O(log |D|) canonical node aggregates covering `r` left to right.
    pub fn walk(self, r: Range<D>) -> yields T = todo

@test
fn folds_match_a_plain_array() = todo

@test
fn at_is_a_writable_leaf() = todo

@test
fn every_bare_operator_folds() = todo

@test
fn of_and_refresh_follow_the_formula() = todo

@test
fn keeps_the_order_for_non_commutative_monoids() = todo

@test
fn right_while_and_left_while_binary_search() = todo

@test
fn update_changes_one_value_in_place() = todo

@test
fn fill_option_initializes_leaves_without_array() = todo

/// Static merge-sort tree for orthogonal 2D range counting over `[D]T`.
/// Build in O(n log n) time and memory; answer value-range counts over any `Range<D>` in O(log^2 n).
pub struct MergeSortTree<D, T>:
    data: List<T>
    beg: List<0..=2_000_000_000>
    fin: List<0..=2_000_000_000>
    size: Int

    /// Build from an array `a: [D]T`. O(n log n).
    pub fn new(a: [D]T) -> MergeSortTree<D, T> = todo

    fn lower_idx(self, u: Int, x: T) -> Int = todo

    fn upper_idx(self, u: Int, x: T) -> Int = todo

    /// Count elements in `r` strictly less than `x`. O(log^2 n).
    pub fn count_lt(self, r: Range<D>, x: T) -> Int = todo

    /// Count elements in `r` less than or equal to `x`. O(log^2 n).
    pub fn count_le(self, r: Range<D>, x: T) -> Int = todo

    /// Count elements in `r` within `[lo_v, hi_v]` inclusive. O(log^2 n).
    pub fn count_in(self, r: Range<D>, lo_v: T, hi_v: T) -> Int = todo

    /// O(log^2 |D|). Count elements in `r` strictly greater than `x`.
    pub fn count_gt(self, r: Range<D>, x: T) -> Int = todo

    /// O(log^2 |D|). Count elements in `r` greater than or equal to `x`.
    pub fn count_ge(self, r: Range<D>, x: T) -> Int = todo

    /// O(log^2 |D|). Smallest element in `r` that is `>= x`, or `None`.
    pub fn first_ge(self, r: Range<D>, x: T) -> ?T = todo

    /// O(log^2 |D|). Largest element in `r` that is `<= x`, or `None`.
    pub fn last_le(self, r: Range<D>, x: T) -> ?T = todo

/// O(|R| * |C|) space. Iterative 2D segment tree: point update and rectangle fold in O(log |R| * log |C|).
pub struct SegTree2D<R, C, T>(m: Monoid<T>):
    t: List<T>

    /// O(|R| * |C|). Build a 2D segment tree over `(R, C)` from `a`, `fill`, or `m.id`.
    pub fn new<M>(op: M, a: ?[R, C]T = None, fill: ?T = None) -> SegTree2D<R, C, T> = todo

    /// O(1). Read cell `(r, c)`.
    pub fn get(self, r: R, c: C) -> T = todo

    /// O(log |R| * log |C|). Assign cell `(r, c)` to `x`.
    pub fn set(mut self, r: R, c: C, x: T) = todo

    /// O(log |R| * log |C|). Replace cell `(r, c)` with `f(current)`.
    pub fn update(mut self, r: R, c: C, f: T -> T) = todo

    /// O(log |R| * log |C|). Fold rectangle `rows` by `cols`.
    pub fn fold(self, rows: Range<R>, cols: Range<C>) -> T = todo

    /// O(1). Fold of the entire grid.
    pub @get fn all(self) -> T = todo

/// O(|D| log V) build and memory. Wavelet matrix over non-negative integer values:
/// O(log V) range k-th smallest, frequency, value-range count, predecessor/successor, and quantile sum.
pub struct WaveletMatrix<D>:
    bits: Int

    /// O(|D| log V). Build a wavelet matrix over `a: [D]Int` (requires all values `>= 0`).
    pub fn new(a: [D]Int) -> WaveletMatrix<D> = todo

    /// O(log V). `k`-th smallest value (0-based) in `r`, or `None` when `k >= r.len`.
    pub fn kth(self, r: Range<D>, k: Int) -> ?Int = todo

    /// O(log V). Number of occurrences of `x` in `r`.
    pub fn rank(self, r: Range<D>, x: Int) -> Int = todo

    /// O(log V). Count elements in `r` strictly less than `x`.
    pub fn count_lt(self, r: Range<D>, x: Int) -> Int = todo

    /// O(log V). Count elements in `r` within `[lo_v, hi_v]` inclusive.
    pub fn count_in(self, r: Range<D>, lo_v: Int, hi_v: Int) -> Int = todo

    /// O(log V). Largest value in `r` strictly less than `x`, or `None`.
    pub fn prev_val(self, r: Range<D>, x: Int) -> ?Int = todo

    /// O(log V). Smallest value in `r` strictly greater than `x`, or `None`.
    pub fn next_val(self, r: Range<D>, x: Int) -> ?Int = todo

    /// O(log V). Sum of the `k` smallest values in `r` (`0 <= k <= r.len`).
    pub fn quantile_sum(self, r: Range<D>, k: Int) -> Int = todo

/// O(|D| log^2 |D|) space. Dynamic point-update + 2D range value-count and k-th structure
/// (Fenwick of order trees) over `[D]T`.
pub struct DynamicOrderTree<D, T>:
    n: Int

    /// O(|D| log^2 |D|). Build from `a: [D]T`.
    pub fn new(a: [D]T) -> DynamicOrderTree<D, T> = todo

    /// O(log^2 |D|). Set position `i` to `x`.
    pub fn set(mut self, i: D, x: T) = todo

    /// O(1). Current value at position `i`.
    pub fn get(self, i: D) -> T = todo

    /// O(log^2 |D|). Count elements in `r` strictly less than `x`.
    pub fn count_lt(self, r: Range<D>, x: T) -> Int = todo

    /// O(log^2 |D|). Count elements in `r` within `[lo_v, hi_v]` inclusive.
    pub fn count_in(self, r: Range<D>, lo_v: T, hi_v: T) -> Int = todo

    /// O(log^2 |D|). `k`-th smallest value (0-based) in `r`, or `None` when `k >= r.len`.
    pub fn kth(self, r: Range<D>, k: Int) -> ?T = todo

/// O(q log |D|) space. Mergeable sparse binary segment tree over domain `D` for lazily pruned
/// minimum-alive queries (`MergeSeg.singleton`, `merge`, `first_alive`).
pub struct MergeSeg<D>:
    root: Int

    /// O(log |D|). Create a singleton tree containing only position `x`.
    pub fn singleton(x: D) -> MergeSeg<D> = todo

    /// O(log |D|) amortized per absorbed node. Merge `other` into `self` and return the combined tree.
    pub fn merge(self, other: MergeSeg<D>) -> MergeSeg<D> = todo

    /// O(log |D|) amortized. Return the smallest position `x` with `alive(x)` true, lazily pruning dead leaves.
    pub fn first_alive(mut self, alive: D -> Bool) -> ?D = todo

@test
fn merge_sort_and_sparse_segtree_match_brute() = todo
