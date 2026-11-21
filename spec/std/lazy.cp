//! Segment tree with range updates.
use std.algebra.{Action, MaxCount, MinCount, Monoid}
use std.rand.Rng
use std.seq.{ends}

/// Range update, range fold: `LazySegTree<V, Int>(AddMax, a)` (the tag type `F` is inferred
/// from the `Action`). The stock actions are `AddSum`, `AddMin`, `AddMax`, `AssignSum`,
/// `AssignMin`, and `AssignMax`; a custom action is
/// `Action(id: ..., compose: (new_f, old_f) => ..., apply: (f, val, span) => ..., over: op)`
/// where `compose` applies `old_f` first then `new_f` and `apply` updates a node fold
/// covering `span` leaves.
/// `apply`, `set`, `update`, `get`, `fold`, `max_right`, and `min_left` are O(log n); `all` is O(1).
/// Memory is 2 * bit_ceil(n) values and spans plus bit_ceil(n) pending tags.
pub struct LazySegTree<D, T, F>(act: Action<T, F>):
    t: List<T>
    lz: List<F>
    has_lz: List<Bool>
    /// How many positions each node covers; the padding past the domain covers none.
    span: List<0..=2_000_000_000>
    size: 0..=2_000_000_000
    log: 0..=64

    /// A tree over `a`, filled with `fill`, or with the identity everywhere. O(n).
    pub fn new(act: Action<T, F>, a: ?[D]T = none, fill: ?T = none) -> LazySegTree<D, T, F> = todo

    fn pull(mut self, p: Int) = todo

    fn put(mut self, p: Int, f: F) = todo

    fn push(mut self, p: Int) = todo

    fn open(mut self, l: Int, h: Int) = todo

    /// Assign position `i` to `x`, pushing pending tags on its path. O(log |D|).
    pub fn set(mut self, i: D, x: T) = todo

    /// Push down to leaf `i`, replace its value with `f(cur)`, and pull back up in one O(log n) traversal.
    pub fn update(mut self, i: D, f: T -> T) = todo

    /// Read the current value at position `i`, pushing pending tags on its path. O(log |D|).
    pub fn get(mut self, i: D) -> T = todo

    /// Position `i` as a writable place: `st.at(i) += x`. Pending tags on its path are
    /// pushed first and the ancestors pulled when the write ends. O(log |D|).
    pub fn at(mut self, i: D) lends mut T:
        let p = i.to_idx + size
        for k in (1..=log).rev: self.push(p >> k)
        lend mut t[p]
        for k in 1..=log: self.pull((i.to_idx + size) >> k)

    /// The fold of every position. O(1).
    pub @get fn all(self) -> T = todo

    /// The operation the tree folds with, as its `(id, op)` pair.
    pub @get fn monoid(self) -> Monoid<T> = todo

    /// The fold of a range, left to right: `st.fold(l..=r)`, `st.fold(..i)`, `st.fold(i..)`.
    /// An empty range gives the identity.
    pub fn fold(mut self, r: Range<D>) -> T = todo

    /// Every position in the range takes the update `f`.
    pub fn apply(mut self, r: Range<D>, f: F) = todo

    /// The furthest boundary `b` right of `from` such that `ok(fold(from..b))`.
    /// `ok` must hold for the identity and switch from true to false at most once.
    pub fn max_right(mut self, from: Bounds<D>, ok: T -> Bool) -> Bounds<D> = todo

    /// The furthest boundary `b` left of `to` such that `ok(fold(b..to))`.
    pub fn min_left(mut self, to: Bounds<D>, ok: T -> Bool) -> Bounds<D> = todo

    /// O(1). Apply tag `f` to every position in the tree.
    pub fn apply_all(mut self, f: F) = todo

    /// O(|D|). Reset all leaves to `fill` (or `act.over.id`) and clear pending tags without reallocating.
    pub fn clear(mut self, fill: ?T = none) = todo

    /// O(|D|). Push all pending tags once and materialize every leaf as `[D]T`.
    pub fn to_array(mut self) -> [D]T = todo

/// Add to every position, fold with `+`.
pub const AddSum = Action<Int, Int>(0, +, (f, x, len) => x + f * len, over: +)
/// Add to every position, fold with `min`.
pub const AddMin = Action<Int, Int>(0, +, (f, x, len) => x + f, over: min)
/// Add to every position, fold with `max`.
pub const AddMax = Action<Int, Int>(0, +, (f, x, len) => x + f, over: max)
/// Overwrite every position, fold with `+`. The pending update is a `?Int`: `none` is "nothing pending".
pub const AssignSum = Action<Int, ?Int>(
    none,
    (f, g) => f ?? g,
    (f, x, len) => if f == none: x else: f! * len,
    over: +,
)
/// Overwrite every position, fold with `min`.
pub const AssignMin = Action<Int, ?Int>(none, (f, g) => f ?? g, (f, x, len) => f ?? x, over: min)
/// Overwrite every position, fold with `max`.
pub const AssignMax = Action<Int, ?Int>(none, (f, g) => f ?? g, (f, x, len) => f ?? x, over: max)
/// Affine update `(mul, add)` over `+` for `Int`: `x -> mul * x + add`.
pub const AffineSum = Action<Int, (Int, Int)>(
    (1, 0),
    (new_f, old_f) => (new_f.0 * old_f.0, new_f.0 * old_f.1 + new_f.1),
    (f, val, span) => f.0 * val + f.1 * span,
    over: +,
)

/// Generic affine update `(mul, add)` over `+` for any ring `T` (`Int`, `Mod<P>`). O(1).
pub fn affine_sum<T>() -> Action<T, (T, T)> = todo

/// O(1). Range add over `(min, count of min)` for counting zero/minimum-covered cells.
pub fn add_min_count<T>() -> Action<MinCount<T>, T> = todo

/// O(1). Range add over `(max, count of max)`.
pub fn add_max_count<T>() -> Action<MaxCount<T>, T> = todo

/// O(1). Range bit-flip (`Bool` tag) over `(ones, len)` popcount sum.
pub fn xor_sum() -> Action<(ones: Int, len: Int), Bool> = todo

/// Paired aggregate `(now, flipped)` for range-toggle / complement updates on a `LazySegTree`.
/// Build the toggle action with `flip_action(m)` (tag `Bool`: `true` swaps `now` and `flipped`).
pub struct Flip<T>(now: T, flipped: T):
    /// Leaf value for an element that is currently `on`: `(x, id)` if `on`, else `(id, x)`. O(1).
    pub fn of(on: Bool, x: T, id: T) -> Flip<T> = todo

/// Range-toggle `Action<Flip<T>, Bool>` over monoid `m`: `true` swaps `now` and `flipped`. O(1).
pub fn flip_action<T, M>(m: M) -> Action<Flip<T>, Bool> = todo

/// Range update, point query: no fold monoid, no pulls, no span array.
/// `compose(new_f, old_f)` applies `old_f` first then `new_f`; `apply(f, x)` updates a single leaf.
/// `apply`, `get`, `set`, `at` and `update` are O(log n); memory is n leaves plus bit_ceil(n) tags.
pub struct DualSegTree<D, T, F>(id: F, compose: (F, F) -> F, apply_fn: (F, T) -> T):
    val: List<T>
    lz: List<F>
    size: Int
    log: Int

    /// A dual tree over `a` or filled with `fill`. O(n).
    pub fn new(
        id: F,
        compose: (F, F) -> F,
        apply: (F, T) -> T,
        a: ?[D]T = none,
        fill: ?T = none,
    ) -> DualSegTree<D, T, F> = todo

    fn put(mut self, p: Int, f: F) = todo

    fn push(mut self, p: Int) = todo

    fn open(mut self, l: Int, h: Int) = todo

    /// Apply tag `f` to every position in `r`. O(log n).
    pub fn apply(mut self, r: Range<D>, f: F) = todo

    /// Current value at position `i`. O(log n).
    pub fn get(mut self, i: D) -> T = todo

    /// Overwrite position `i` with `x`. O(log n).
    pub fn set(mut self, i: D, x: T) = todo

    /// Position `i` as a writable place: `st.at(i) += x`, after pushing the tags above it. O(log n).
    pub fn at(mut self, i: D) lends mut T:
        let p = i.to_idx + size
        for k in (1..=log).rev: self.push(p >> k)
        lend mut val[i.to_idx]

    /// Replace position `i` with `f(cur)` in one O(log n) pass.
    pub fn update(mut self, i: D, f: T -> T) = todo

    /// O(1). Apply tag `f` to every position in the tree.
    pub fn apply_all(mut self, f: F) = todo

    /// O(|D|). Push all pending tags once and materialize every leaf as `[D]T`.
    pub fn to_array(mut self) -> [D]T = todo

/// O(q log(hi - lo)) space. Dynamic (coordinate-sparse) lazy segment tree over `[lo, hi)`
/// for huge coordinate ranges (e.g. `0..1e9`) without coordinate compression.
pub struct SparseLazySegTree<T, F>(act: Action<T, F>, lo: Int, hi: Int):
    root: Int

    /// O(1). Create an empty sparse lazy segment tree over half-open `[lo, hi)` filled with `fill` or `act.over.id`.
    pub fn new(act: Action<T, F>, lo: Int, hi: Int, fill: ?T = none) -> SparseLazySegTree<T, F> = todo

    /// O(log(hi - lo)). Apply tag `f` to every coordinate in `[ql, qr)`.
    pub fn apply(mut self, ql: Int, qr: Int, f: F) = todo

    /// O(log(hi - lo)). Fold coordinates in `[ql, qr)`.
    pub fn fold(mut self, ql: Int, qr: Int) -> T = todo

    /// O(log(hi - lo)). Read the value at coordinate `x`.
    pub fn get(mut self, x: Int) -> T = todo

    /// O(log(hi - lo)). Set coordinate `x` to `v`.
    pub fn set(mut self, x: Int, v: T) = todo

    /// O(1). Fold over the entire `[lo, hi)` coordinate range.
    pub @get fn all(self) -> T = todo

    /// O(log(hi - lo)). Furthest coordinate `x >= from` with `ok(fold(from, x))`.
    pub fn max_right(mut self, from: Int, ok: T -> Bool) -> Int = todo

    /// O(log(hi - lo)). Furthest coordinate `x <= to` with `ok(fold(x, to))`.
    pub fn min_left(mut self, to: Int, ok: T -> Bool) -> Int = todo

fn brute<D>(a: [D]Int, r: Range<D>, pick: Int) -> Int = todo

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
fn max_right_and_min_left_see_pending_updates() = todo

fn ccc_compose(f: (?Int, ?Int, Int), g: (?Int, ?Int, Int)) -> (?Int, ?Int, Int) = todo

fn ccc_apply(f: (?Int, ?Int, Int), x: (Int, Int, Int), len: Int) -> (Int, Int, Int) = todo

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

    /// Create an empty `LazyTreap` with node updater `apply`, tag composition `compose`,
    /// optional subtree tag splitter `split`, and optional order predicate `before(x, y)`
    /// (true when `x` belongs before `y`). O(1).
    pub fn new(
        apply: (T, Int, F) -> T,
        compose: (F, F) -> F,
        split: (F, Int) -> (F, F) = (f, _) => (f, f),
        before: (T, T) -> Bool = (_, _) => false,
    ) -> LazyTreap<T, F> = todo

    fn priority(mut self) -> Int = todo

    fn sz_of(self, u: ?Int) -> Int = todo

    /// Number of elements currently stored. O(1).
    @get
    pub fn len(self) -> Int = todo

    /// Make room for `n` elements in total without regrowing. O(n).
    pub fn reserve(mut self, n: Int) = todo

    /// Whether the treap is empty. O(1).
    @get
    pub fn is_empty(self) -> Bool = todo

    fn put(mut self, u: ?Int, f: F) = todo

    fn down(mut self, i: Int) = todo

    fn pull(mut self, i: Int) = todo

    fn split_val(mut self, u: ?Int, x: T) -> (?Int, ?Int) = todo

    fn ins(mut self, u: ?Int, idx: Int, x: T, p: Int) -> Int = todo

    fn split_pos(mut self, u: ?Int, k: Int) -> (?Int, ?Int) = todo

    fn merge(mut self, a: ?Int, b: ?Int) -> ?Int = todo

    /// Insert `x` at the first position where `before(x, elem)` holds. O(log n) expected.
    pub fn insert(mut self, x: T) = todo

    /// Append `x` at the end of the sequence. O(log n) expected.
    pub fn push(mut self, x: T) = todo

    /// Insert `x` at 0-based position `pos` (`0 <= pos <= len`). O(log n) expected.
    pub fn insert_at(mut self, pos: Int, x: T) = todo

    /// Apply lazy tag `f` to every element in the treap. O(1).
    pub fn apply_all(mut self, f: F) = todo

    /// Apply lazy tag `f` to elements at 0-based positions `lo..hi`. O(log n) expected.
    pub fn apply_range(mut self, lo: Int, hi: Int, f: F) = todo

    /// O(log n) expected. Apply lazy tag `f` to elements at positions in `r`.
    pub fn apply(mut self, r: Range<Int>, f: F) = todo

    /// O(log n) expected. Reverse the subsegment at positions `r` in place.
    pub fn reverse(mut self, r: Range<Int>) = todo

    /// O(log n) expected. Remove and return the element at 0-based position `pos`.
    pub fn erase_at(mut self, pos: Int) -> T = todo

    /// O(log n) expected. Remove all elements in range `r`.
    pub fn erase_range(mut self, r: Range<Int>) = todo

    /// O(log n) expected. Split off elements from index `k` onward into a new `LazyTreap<T, F>`.
    pub fn split_at(mut self, k: Int) -> LazyTreap<T, F> = todo

    /// O(log n) expected. Append all elements of `other` onto the end of `self`, emptying `other`.
    pub fn append(mut self, mut other: LazyTreap<T, F>) = todo

    /// Read the element at 0-based position `pos`. O(log n) expected.
    pub fn get(mut self, pos: Int) -> T = todo

    // A read leaves the tree as it is: the tags still pending above a node
    // ride along as `pend` instead of being pushed into it.
    fn walk(self, u: ?Int, pend: ?F, lim: Int, mut out: List<T>) = todo

    /// The first `min(k, len)` elements in order. O(k + log n).
    pub fn take(self, k: Int) -> List<T> = todo

    /// All elements in order. O(n).
    pub fn to_list(self) -> List<T> = todo

/// O(n) space. Implicit-key rope over a domain `D` supporting O(log n) expected cut, paste,
/// reverse, range tag `apply`, path queries, and point `get`.
pub struct Rope<D, T>:
    n: Int

    /// O(n). Build a rope over `D` initialized from `a`, `fill`, or a tag `merge` operation.
    pub fn new(a: ?[D]T = none, fill: ?T = none, merge: ?((T, T) -> T) = none) -> Rope<D, T> = todo

    /// O(log n) expected. Apply tag `tag` to subrange `r`.
    pub fn apply(mut self, r: Range<D>, tag: T) = todo

    /// O(log n) expected. Reverse subrange `r` in place.
    pub fn reverse(mut self, r: Range<D>) = todo

    /// O(log n) expected. Walk root-to-leaf to position `i` and return the first `some` result of `f` on path tags.
    pub fn query_path<R>(mut self, i: D, f: T -> ?R) -> ?R = todo

    /// O(log n) expected. Read the element currently at position `i`.
    pub fn get(mut self, i: D) -> T = todo

    /// O(log n) expected. Overwrite the element currently at position `i`.
    pub fn set(mut self, i: D, x: T) = todo

    /// O(n). Materialize the current sequence as `[D]T`.
    pub fn to_array(mut self) -> [D]T = todo

@test
fn lazy_treap_slope_trick_and_implicit_ops_match_brute() = todo

@test
fn flip_action_toggles_ranges() = todo
