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

    /// A tree over `D` folding with `op`: built from `a`, filled with `fill`, or holding the identity
    /// everywhere when neither is given. `op` is a bare operator (`+`, `*`, `^`, `&`, `|`, `min`,
    /// `max`, `gcd`) or a `Monoid(id, op)`.
    ///
    /// `act:` adds range tags (`apply`, `apply_all`); with `act:` and no `op` the tree applies tags
    /// and reads points only. When the tag monoid is `Commutative` the tree skips push-down, which
    /// changes speed, never answers. `over: lo..hi` covers a huge `Int` range instead, making nodes
    /// on first touch; positions outside the touched ones hold `fill` or the identity.
    ///
    /// cost: O(|D|); O(1) with `over:`
    /// empty: an empty `D` gives a tree whose every fold is the identity
    /// order: positions keep the domain `D` of `a`
    /// traps: when both `a` and `fill` are given, or `over:` is given together with `a`
    /// example: `SegTree(+, [1, 2, 3]).all == 6`
    /// example: `SegTree<0..4, Int>(max, fill: 5).fold(1..3) == 5`
    /// example: `SegTree<0..4, Int>(+).all == 0`
    /// example: `var s = SegTree<Int, Int>(+, over: 0..1_000_000_000); s.set(999_999_999, 4); s.all == 4`
    pub fn new<M, A>(op: ?M = None, a: ?[D]T = None, fill: ?T = None, act: ?A = None, over: ?Range<Int> = None) -> SegTree<D, T> = todo

    /// Apply tag `f` to every position of `r`: each fold over `r` then sees the values with `f`
    /// applied, through the action given to `act:`.
    ///
    /// cost: O(log |D|)
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var s = SegTree(+, [1, 2, 3], act: add); s.apply(0..2, 10); s.all == 26`
    /// example: `var s = SegTree(max, [1, 2, 3], act: add); s.apply(0..1, 5); s.all == 6`
    pub fn apply(mut self, r: Range<D>, f: F) = todo

    /// Apply tag `f` to every position; `apply(D.all, f)`.
    ///
    /// cost: O(1) amortized when the tags commute, else O(|D|)
    /// empty: changes nothing for an empty `D`
    /// traps: never
    /// example: `var s = SegTree(+, [1, 2, 3], act: add); s.apply_all(1); s.all == 9`
    pub fn apply_all(mut self, f: F) = todo

    /// The fold of the values in `r`, left to right: `op(op(a[l], a[l + 1]), ...)`.
    ///
    /// cost: O(log |D|)
    /// empty: the identity
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var s = SegTree(+, [1, 2, 3], act: add); s.apply(1..3, 1); s.fold(1..3) == 7`
    pub fn fold(self, r: Range<D>) -> T = todo

    /// The last cut `d` at or right of `from` such that `ok(fold(from..d))` holds. `ok` must be
    /// monotone (once false it stays false as `d` moves right) and true on the identity, so `from`
    /// itself always qualifies. Replaces `max_right`.
    ///
    /// cost: O(log |D|)
    /// ties: the rightmost qualifying cut
    /// traps: when `ok(identity)` is false
    /// example: `SegTree(+, [1, 2, 3, 4]).right_while((0..0).r, s => s <= 3) == (0..2).r`
    /// example: `SegTree(+, [1, 2, 3, 4]).right_while((0..1).r, s => s <= 5) == (0..3).r`
    /// example: `SegTree(+, [1, 2, 3, 4]).right_while((0..0).r, s => s <= 100) == (0..4).r`
    pub fn right_while(self, from: Cut<D>, ok: T -> Bool) -> Cut<D> = todo

    /// The first cut `d` at or left of `to` such that `ok(fold(d..to))` holds. `ok` must be
    /// monotone (once false it stays false as `d` moves left) and true on the identity, so `to`
    /// itself always qualifies. Replaces `min_left`.
    ///
    /// cost: O(log |D|)
    /// ties: the leftmost qualifying cut
    /// traps: when `ok(identity)` is false
    /// example: `SegTree(+, [1, 2, 3, 4]).left_while((0..4).r, s => s <= 7) == (0..2).r`
    /// example: `SegTree(+, [1, 2, 3, 4]).left_while((0..4).r, s => s <= 3) == (0..4).r`
    pub fn left_while(self, to: Cut<D>, ok: T -> Bool) -> Cut<D> = todo

    /// A tree whose position `i` holds `f(i)`. `f` is kept: when something it reads changes,
    /// `refresh(i)` recomputes position `i`, so the formula is written once instead of at every
    /// update site.
    ///
    /// cost: O(|D|), calling `f` once per position in key order
    /// empty: an empty `D` gives a tree whose every fold is the identity
    /// order: position `i` holds `f(i)`
    /// traps: never
    /// example: `let w = [1, 2, 3]; SegTree<0..3, Int>.of(+, i => w[i] * 2).all == 12`
    pub fn of<M>(op: M, f: D -> T) -> SegTree<D, T> = todo

    /// Recompute position `i` as `f(i)` with the formula given to `of`, and update its ancestors.
    ///
    /// cost: O(log |D|) plus one call of `f`
    /// traps: when `i` is outside `D`, or the tree was not built with `of`
    /// example: `var w = [1, 2, 3]; var s = SegTree<0..3, Int>.of(+, i => w[i]); w[1] = 10; s.refresh(1); s.all == 14`
    pub fn refresh(mut self, i: D) = todo

    /// The value at position `i`, with every tag applied.
    ///
    /// cost: O(1); O(log |D|) on a tree with `act:`
    /// traps: when `i` is outside `D`
    /// example: `SegTree(+, [4, 5, 6]).get(1) == 5`
    /// example: `var s = SegTree(act: assign, [1, 2, 3]); s.apply(0..2, Some(9)); s.get(1) == 9`
    pub fn get(self, i: D) -> T = todo

    /// Make position `i` hold `x`, replacing any tags applied to it, and update its ancestors.
    ///
    /// cost: O(log |D|)
    /// traps: when `i` is outside `D`
    /// example: `var s = SegTree(+, [1, 2, 3]); s.set(0, 10); s.all == 15`
    pub fn set(mut self, i: D, x: T) = todo

    /// Position `i` as a writable place: `st.at(i) += x`, `st.at(i) max= x`. The ancestors are
    /// updated when the write ends, so each use is one `set`; read with `get` instead.
    ///
    /// cost: O(log |D|) per write
    /// traps: when `i` is outside `D`
    /// example: `var s = SegTree(+, [1, 2, 3]); s.at(0) += 5; s.all == 11`
    pub fn at(mut self, i: D) -> lends mut T:
        lend mut t[size + i.to_idx]
        self.pull(i)

    fn pull(mut self, i: D) = todo

    /// Replace the value `v` at `i` with `f(v)`; `set(i, f(get(i)))` in one walk.
    ///
    /// cost: O(log |D|) plus one call of `f`
    /// traps: when `i` is outside `D`
    /// example: `var s = SegTree(+, [1, 2, 3]); s.update(2, x => x * 10); s.all == 33`
    pub fn update(mut self, i: D, f: T -> T) = todo

    /// The fold of every position, left to right.
    ///
    /// cost: O(1)
    /// empty: the identity for an empty `D`
    /// traps: never
    /// example: `SegTree(max, [3, 9, 4]).all == 9`
    @get pub fn all(self) -> T = todo

    /// The operation the tree folds with, as its `(id, op)` pair.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `SegTree(+, [1]).monoid.id == 0`
    @get pub fn monoid(self) -> Monoid<T> = todo



    /// The fold of `r`, or `None` when `r` is empty; for operations whose identity is only a
    /// sentinel.
    ///
    /// cost: O(log |D|)
    /// empty: `None`
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `SegTree(+, [1, 2]).try_fold(0..2) == Some(3)`
    /// example: `SegTree(+, [1, 2]).try_fold(1..1) == None`
    pub fn try_fold(self, r: Range<D>) -> ?T = todo

    /// Make every position hold `fill`, or the identity when `fill` is `None`, dropping all pending
    /// tags; the storage is reused. A tree built with `of` keeps its formula.
    ///
    /// cost: O(|D|)
    /// empty: nothing to reset for an empty `D`
    /// traps: never
    /// example: `var s = SegTree(+, [1, 2, 3]); s.clear(); s.all == 0`
    /// example: `var s = SegTree(+, [1, 2, 3]); s.clear(fill: 2); s.all == 6`
    pub fn clear(mut self, fill: ?T = None) = todo

    /// The values of the canonical nodes that exactly cover `r`: the maximal aligned blocks of the
    /// tree's `bit_ceil(|D|)` leaves inside `r`. Their fold, left to right, is `fold(r)`.
    ///
    /// cost: O(log |D|)
    /// empty: yields nothing
    /// order: left to right; at most `2 * log2(bit_ceil(|D|))` values
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `SegTree(+, [1, 2, 3, 4]).walk(0..3).collect() == [3, 3]`
    /// example: `SegTree(+, [1, 2, 3, 4]).walk(1..4).collect() == [2, 7]`
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

    /// A static tree over `a` whose nodes hold their values sorted, for counting values in a range.
    ///
    /// cost: O(|D| log |D|) time and memory
    /// empty: an empty `a` gives a tree where every count is 0
    /// order: positions keep the domain `D` of `a`
    /// traps: never
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).count_lt(0..5, 3) == 2`
    pub fn new(a: [D]T) -> MergeSortTree<D, T> = todo

    fn lower_idx(self, u: Int, x: T) -> Int = todo

    fn upper_idx(self, u: Int, x: T) -> Int = todo

    /// The number of positions in `r` whose value is `< x`.
    ///
    /// cost: O(log^2 |D|)
    /// empty: 0
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).count_lt(0..5, 3) == 2`
    pub fn count_lt(self, r: Range<D>, x: T) -> Int = todo

    /// The number of positions in `r` whose value is `<= x`.
    ///
    /// cost: O(log^2 |D|)
    /// empty: 0
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).count_le(0..5, 3) == 3`
    pub fn count_le(self, r: Range<D>, x: T) -> Int = todo

    /// The number of positions in `r` whose value lies in `lo_v..=hi_v`; 0 when `lo_v > hi_v`.
    ///
    /// cost: O(log^2 |D|)
    /// empty: 0
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).count_in(1..4, 1, 4) == 3`
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).count_in(0..5, 4, 3) == 0`
    pub fn count_in(self, r: Range<D>, lo_v: T, hi_v: T) -> Int = todo

    /// The number of positions in `r` whose value is `> x`.
    ///
    /// cost: O(log^2 |D|)
    /// empty: 0
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).count_gt(0..5, 3) == 2`
    pub fn count_gt(self, r: Range<D>, x: T) -> Int = todo

    /// The number of positions in `r` whose value is `>= x`.
    ///
    /// cost: O(log^2 |D|)
    /// empty: 0
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).count_ge(0..5, 4) == 2`
    pub fn count_ge(self, r: Range<D>, x: T) -> Int = todo

    /// The smallest value in `r` that is `>= x`, or `None`.
    ///
    /// cost: O(log^2 |D|)
    /// empty: `None`
    /// ties: equal values are interchangeable; the value is returned, not a position
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).first_ge(0..3, 2) == Some(3)`
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).first_ge(0..3, 5) == None`
    pub fn first_ge(self, r: Range<D>, x: T) -> ?T = todo

    /// The largest value in `r` that is `<= x`, or `None`.
    ///
    /// cost: O(log^2 |D|)
    /// empty: `None`
    /// ties: equal values are interchangeable; the value is returned, not a position
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).last_le(2..5, 3) == Some(1)`
    /// example: `MergeSortTree([3, 1, 4, 1, 5]).last_le(2..5, 0) == None`
    pub fn last_le(self, r: Range<D>, x: T) -> ?T = todo

/// O(|R| * |C|) space. Iterative 2D segment tree: point update and rectangle fold in O(log |R| * log |C|).
pub struct SegTree2D<R, C, T>(m: Monoid<T>):
    t: List<T>

    /// A grid tree over `(R, C)` folding with `op`: built from `a`, filled with `fill`, or holding
    /// the identity everywhere. `op` must be `Commutative`, since a rectangle is folded in
    /// row-blocks and column-blocks rather than in one order.
    ///
    /// cost: O(|R| * |C|)
    /// empty: an empty `R` or `C` gives a tree whose every fold is the identity
    /// order: cells keep the domains `R` and `C` of `a`
    /// traps: when both `a` and `fill` are given
    /// example: `SegTree2D<0..3, 0..3, Int>(+, fill: 1).all == 9`
    pub fn new<M>(op: M, a: ?[R, C]T = None, fill: ?T = None) -> SegTree2D<R, C, T> = todo

    /// The value of cell `(r, c)`.
    ///
    /// cost: O(1)
    /// traps: when `r` is outside `R` or `c` is outside `C`
    /// example: `var g = SegTree2D<0..3, 0..3, Int>(+, fill: 1); g.set(1, 2, 7); g.get(1, 2) == 7`
    pub fn get(self, r: R, c: C) -> T = todo

    /// Make cell `(r, c)` hold `x`.
    ///
    /// cost: O(log |R| * log |C|)
    /// traps: when `r` is outside `R` or `c` is outside `C`
    /// example: `var g = SegTree2D<0..3, 0..3, Int>(+, fill: 1); g.set(0, 0, 5); g.all == 13`
    pub fn set(mut self, r: R, c: C, x: T) = todo

    /// Replace the value `v` of cell `(r, c)` with `f(v)`.
    ///
    /// cost: O(log |R| * log |C|) plus one call of `f`
    /// traps: when `r` is outside `R` or `c` is outside `C`
    /// example: `var g = SegTree2D<0..3, 0..3, Int>(+, fill: 1); g.update(2, 2, x => x + 1); g.all == 10`
    pub fn update(mut self, r: R, c: C, f: T -> T) = todo

    /// The fold of every cell in rows `rows` and columns `cols`.
    ///
    /// cost: O(log |R| * log |C|)
    /// empty: the identity when `rows` or `cols` is empty
    /// traps: when a non-empty range reaches outside its domain
    /// example: `SegTree2D<0..3, 0..3, Int>(+, fill: 1).fold(0..2, 1..3) == 4`
    /// example: `SegTree2D<0..3, 0..3, Int>(+, fill: 1).fold(0..2, 1..1) == 0`
    pub fn fold(self, rows: Range<R>, cols: Range<C>) -> T = todo

    /// The fold of every cell.
    ///
    /// cost: O(1)
    /// empty: the identity for an empty grid
    /// traps: never
    /// example: `var g = SegTree2D<0..2, 0..2, Int>(max, fill: 0); g.set(1, 0, 8); g.all == 8`
    @get pub fn all(self) -> T = todo

/// O(|D| log V) build and memory. Wavelet matrix over non-negative integer values:
/// O(log V) range k-th smallest, frequency, value-range count, predecessor/successor, and quantile sum.
pub struct WaveletMatrix<D>:
    bits: Int

    /// A wavelet matrix over the non-negative values of `a`, with one bit level per bit of the
    /// largest value (`V` below is that value plus one).
    ///
    /// cost: O(|D| log V) time and memory
    /// empty: an empty `a` gives a matrix where every count is 0
    /// order: positions keep the domain `D` of `a`
    /// traps: when some value is negative
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).kth(0..5, 2) == Some(3)`
    pub fn new(a: [D]Int) -> WaveletMatrix<D> = todo

    /// The `k`-th smallest value in `r`, counting from 0 and with repeats, or `None` when
    /// `k >= r.len`.
    ///
    /// cost: O(log V)
    /// empty: `None`
    /// ties: equal values are counted once each
    /// traps: when `k < 0`, or a non-empty `r` reaches outside `D`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).kth(0..5, 0) == Some(1)`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).kth(0..5, 2) == Some(3)`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).kth(0..5, 5) == None`
    pub fn kth(self, r: Range<D>, k: Int) -> ?Int = todo

    /// The number of positions in `r` holding exactly `x`; 0 for any `x` outside `0..V`.
    ///
    /// cost: O(log V)
    /// empty: 0
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).rank(0..5, 1) == 2`
    pub fn rank(self, r: Range<D>, x: Int) -> Int = todo

    /// The number of positions in `r` whose value is `< x`.
    ///
    /// cost: O(log V)
    /// empty: 0
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).count_lt(0..5, 4) == 3`
    pub fn count_lt(self, r: Range<D>, x: Int) -> Int = todo

    /// The number of positions in `r` whose value lies in `lo_v..=hi_v`; 0 when `lo_v > hi_v`.
    ///
    /// cost: O(log V)
    /// empty: 0
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).count_in(0..5, 1, 3) == 3`
    pub fn count_in(self, r: Range<D>, lo_v: Int, hi_v: Int) -> Int = todo

    /// The largest value in `r` that is `< x`, or `None`.
    ///
    /// cost: O(log V)
    /// empty: `None`
    /// ties: equal values are interchangeable; the value is returned, not a position
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).prev_val(0..5, 3) == Some(1)`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).prev_val(0..5, 1) == None`
    pub fn prev_val(self, r: Range<D>, x: Int) -> ?Int = todo

    /// The smallest value in `r` that is `> x`, or `None`.
    ///
    /// cost: O(log V)
    /// empty: `None`
    /// ties: equal values are interchangeable; the value is returned, not a position
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).next_val(0..5, 3) == Some(4)`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).next_val(0..5, 5) == None`
    pub fn next_val(self, r: Range<D>, x: Int) -> ?Int = todo

    /// The sum of the `k` smallest values in `r`, with repeats.
    ///
    /// cost: O(log V)
    /// empty: 0 (only `k == 0` is allowed)
    /// ties: equal values are counted once each
    /// traps: when `k` is outside `0..=r.len`, or a non-empty `r` reaches outside `D`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).quantile_sum(0..5, 3) == 5`
    /// example: `WaveletMatrix([3, 1, 4, 1, 5]).quantile_sum(1..3, 0) == 0`
    pub fn quantile_sum(self, r: Range<D>, k: Int) -> Int = todo

/// O(|D| log^2 |D|) space. Dynamic point-update + 2D range value-count and k-th structure
/// (Fenwick of order trees) over `[D]T`.
pub struct DynamicOrderTree<D, T>:
    n: Int

    /// A structure over `a` answering value counts and order statistics on ranges while positions
    /// are reassigned.
    ///
    /// cost: O(|D| log^2 |D|) time and memory
    /// empty: an empty `a` gives a tree where every count is 0
    /// order: positions keep the domain `D` of `a`
    /// traps: never
    /// example: `DynamicOrderTree([3, 1, 4]).count_lt(0..3, 4) == 2`
    pub fn new(a: [D]T) -> DynamicOrderTree<D, T> = todo

    /// Make position `i` hold `x`.
    ///
    /// cost: O(log^2 |D|)
    /// traps: when `i` is outside `D`
    /// example: `var t = DynamicOrderTree([3, 1, 4]); t.set(0, 0); t.kth(0..3, 0) == Some(0)`
    pub fn set(mut self, i: D, x: T) = todo

    /// The value at position `i`.
    ///
    /// cost: O(1)
    /// traps: when `i` is outside `D`
    /// example: `var t = DynamicOrderTree([3, 1, 4]); t.set(2, 9); t.get(2) == 9`
    pub fn get(self, i: D) -> T = todo

    /// The number of positions in `r` whose value is `< x`.
    ///
    /// cost: O(log^2 |D|)
    /// empty: 0
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `DynamicOrderTree([3, 1, 4]).count_lt(1..3, 4) == 1`
    pub fn count_lt(self, r: Range<D>, x: T) -> Int = todo

    /// The number of positions in `r` whose value lies in `lo_v..=hi_v`; 0 when `lo_v > hi_v`.
    ///
    /// cost: O(log^2 |D|)
    /// empty: 0
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `DynamicOrderTree([3, 1, 4]).count_in(0..3, 1, 3) == 2`
    pub fn count_in(self, r: Range<D>, lo_v: T, hi_v: T) -> Int = todo

    /// The `k`-th smallest value in `r`, counting from 0 and with repeats, or `None` when
    /// `k >= r.len`.
    ///
    /// cost: O(log^2 |D|)
    /// empty: `None`
    /// ties: equal values are counted once each
    /// traps: when `k < 0`, or a non-empty `r` reaches outside `D`
    /// example: `DynamicOrderTree([3, 1, 4]).kth(0..3, 1) == Some(3)`
    /// example: `DynamicOrderTree([3, 1, 4]).kth(0..3, 3) == None`
    pub fn kth(self, r: Range<D>, k: Int) -> ?T = todo

/// O(q log |D|) space. Mergeable sparse binary segment tree over domain `D` for lazily pruned
/// minimum-alive queries (`MergeSeg.singleton`, `merge`, `first_alive`).
pub struct MergeSeg<D>:
    root: Int

    /// A set of positions of `D` holding only `x`, stored as a sparse binary tree over `D`.
    ///
    /// cost: O(log |D|)
    /// traps: when `x` is outside `D`
    /// example: `MergeSeg<0..10>.singleton(3).first_alive(x => true) == Some(3)`
    pub fn singleton(x: D) -> MergeSeg<D> = todo

    /// The union of `self` and `other`. The result reuses the nodes of both, so neither input may be
    /// used afterwards.
    ///
    /// cost: O(log |D|) amortized per node absorbed
    /// traps: never
    /// example: `var m = MergeSeg<0..10>.singleton(7).merge(MergeSeg<0..10>.singleton(3)); m.first_alive(x => true) == Some(3)`
    pub fn merge(self, other: MergeSeg<D>) -> MergeSeg<D> = todo

    /// The smallest position `x` in the set with `alive(x)`, or `None`. Every position found dead on
    /// the way is removed for good, so `alive` must be monotone over time: a position never comes
    /// back to life.
    ///
    /// cost: O(log |D|) amortized
    /// ties: the smallest position in key order
    /// traps: never
    /// example: `var m = MergeSeg<0..10>.singleton(3).merge(MergeSeg<0..10>.singleton(7)); m.first_alive(x => x > 5) == Some(7)`
    /// example: `var m = MergeSeg<0..10>.singleton(3).merge(MergeSeg<0..10>.singleton(7)); m.first_alive(x => x > 5); m.first_alive(x => true) == Some(7)`
    pub fn first_alive(mut self, alive: D -> Bool) -> ?D = todo

@test
fn merge_sort_and_sparse_segtree_match_brute() = todo
