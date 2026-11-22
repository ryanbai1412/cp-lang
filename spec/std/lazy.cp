//! Range-update helpers that are not segment trees: `Flip`, `LazyTreap`, `Rope`. Range apply on a segment tree is `SegTree(op, a, act:)` (`segtree.cp`).
use std.algebra.{Action, MaxCount, MinCount, Monoid}
use std.rand.Rng
use std.seq.{ends}


/// Paired aggregate `(now, flipped)` for range-toggle / complement updates (`SegTree(op, a, act: flip)`).
/// Build the toggle action with `flip_action(m)` (tag `Bool`: `true` swaps `now` and `flipped`).
pub struct Flip<T>(now: T, flipped: T):
    /// Leaf value for an element that is currently `on`: `(x, id)` if `on`, else `(id, x)`. O(1).
    pub fn of(on: Bool, x: T, id: T) -> Flip<T> = todo

    /// The pairwise `Monoid<Flip<T>>` of a monoid on `T`, for `SegTree(Flip.op(m), a, act: flip)`. O(1).
    pub fn op<M>(m: M) -> Monoid<Flip<T>> = todo

/// Range-toggle `Action<Flip<T>, Bool>` over monoid `m`: `true` swaps `now` and `flipped`. O(1).
pub fn flip_action<T, M>(m: M) -> Action<Flip<T>, Bool> = todo

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
fn right_while_and_left_while_see_pending_updates() = todo

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
