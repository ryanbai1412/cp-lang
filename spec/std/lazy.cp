//! Range-update helpers that are not segment trees: `Flip`, `LazyTreap`, `Rope`. Range apply on a segment tree is `SegTree(op, a, act:)` (`segtree.cp`).
use std.algebra.{Action, MaxCount, MinCount, Monoid}
use std.rand.Rng
use std.seq.{ends}


/// Paired aggregate `(now, flipped)` for range-toggle / complement updates (`SegTree(op, a, act: flip)`).
/// Build the toggle action with `flip_action(m)` (tag `Bool`: `true` swaps `now` and `flipped`).
pub struct Flip<T>(now: T, flipped: T):
    /// The leaf for one element that is currently `on` or off: `Flip(x, id)` when `on`, else
    /// `Flip(id, x)`. `now` is the value the element contributes as it stands and `flipped` what it
    /// would contribute after one toggle.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Flip.of(true, 5, 0).now == 5`
    /// example: `Flip.of(false, 5, 0).flipped == 5`
    pub fn of(on: Bool, x: T, id: T) -> Flip<T> = todo

    /// The monoid on pairs that folds both halves with `m` independently:
    /// `op(a, b) == Flip(m(a.now, b.now), m(a.flipped, b.flipped))`, identity `Flip(id, id)` with
    /// `id` from `Monoid.of(m)`.
    ///
    /// cost: O(1)
    /// traps: never (an `m` without a known identity is a compile-time error)
    /// example: `Flip.op(+).op(Flip(1, 0), Flip(0, 1)).now == 1`
    /// example: `Flip.op(+).id.flipped == 0`
    pub fn op<M>(m: M) -> Monoid<Flip<T>> = todo

/// The range-toggle action on `Flip<T>` values over `m`: the tag is a `Bool`, `true` swaps
/// `now` and `flipped`, `false` is the identity, and tags compose by exclusive or. The swap
/// needs no span, since both halves were folded with the same `m`.
///
/// cost: O(1)
/// traps: never
/// example: `flip_action(+).apply(true, Flip(3, 1), 4).now == 1`
/// example: `flip_action(+).compose(true, true) == false`
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

    /// An empty treap. `apply(v, k, f)` is element `v` after tag `f`, where `k` is the number of
    /// elements before it in the run the tag was applied to (so tags may depend on rank);
    /// `compose(new_f, old_f)` is `old_f` then `new_f`; `split(f, k)` gives the tags for the left
    /// child and for the right child of a node with `k` elements on its left (default `(f, f)`, for
    /// tags that ignore rank; a rank-dependent tag needs a `split` that shifts the right child's ranks
    /// by `k + 1`); `before(x, y)` orders value insertion (default never, so `insert` appends).
    ///
    /// cost: O(1)
    /// empty: the new treap is empty
    /// traps: never
    /// example: `LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f).len == 0`
    pub fn new(
        apply: (T, Int, F) -> T,
        compose: (F, F) -> F,
        split: (F, Int) -> (F, F) = (f, _) => (f, f),
        before: (T, T) -> Bool = (_, _) => false,
    ) -> LazyTreap<T, F> = todo

    fn priority(mut self) -> Int = todo

    fn sz_of(self, u: ?Int) -> Int = todo

    /// The number of elements stored.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(4); t.len == 1`
    pub @get fn len(self) -> Int = todo

    /// Make room for `n` elements in total, so pushes up to that size never regrow the node arena.
    /// A smaller `n` than the current capacity changes nothing.
    ///
    /// cost: O(n) when it grows, else O(1)
    /// traps: when `n >= 2^31` (the size bound)
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.reserve(10); t.len == 0`
    pub fn reserve(mut self, n: Int) = todo

    /// Whether the treap holds no elements.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f).is_empty == true`
    pub @get fn is_empty(self) -> Bool = todo

    fn put(mut self, u: ?Int, f: F) = todo

    fn down(mut self, i: Int) = todo

    fn pull(mut self, i: Int) = todo

    fn split_val(mut self, u: ?Int, x: T) -> (?Int, ?Int) = todo

    fn ins(mut self, u: ?Int, idx: Int, x: T, p: Int) -> Int = todo

    fn split_pos(mut self, u: ?Int, k: Int) -> (?Int, ?Int) = todo

    fn merge(mut self, a: ?Int, b: ?Int) -> ?Int = todo

    /// Insert `x` before the first element `e` with `before(x, e)`, or at the end when there is
    /// none. With a strict order for `before`, a sorted treap stays sorted and `x` goes after the
    /// elements equal to it.
    ///
    /// cost: O(log n) expected
    /// ties: after every equal element
    /// traps: when the treap already holds `2^31 - 1` elements
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f, before: (x, y) => x < y); t.insert(3); t.insert(1); t.insert(2); t.to_list() == [1, 2, 3]`
    pub fn insert(mut self, x: T) = todo

    /// Append `x` at the end of the sequence.
    ///
    /// cost: O(log n) expected
    /// traps: when the treap already holds `2^31 - 1` elements
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(2); t.to_list() == [1, 2]`
    pub fn push(mut self, x: T) = todo

    /// Insert `x` so that it becomes the element at 0-based position `pos`; `pos == len` appends.
    ///
    /// cost: O(log n) expected
    /// traps: when `pos < 0` or `pos > len`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(3); t.insert_at(1, 2); t.to_list() == [1, 2, 3]`
    pub fn insert_at(mut self, pos: Int, x: T) = todo

    /// Apply tag `f` to every element, as one tag on the root; ranks `k` count from the first element.
    ///
    /// cost: O(1)
    /// empty: changes nothing
    /// traps: never
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(2); t.apply_all(10); t.to_list() == [11, 12]`
    pub fn apply_all(mut self, f: F) = todo

    /// Apply tag `f` to the elements at positions `lo..hi`; ranks `k` count from position `lo`.
    ///
    /// cost: O(log n) expected
    /// empty: `lo == hi` changes nothing
    /// traps: when `lo < 0`, `hi > len` or `lo > hi`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(0); t.push(0); t.push(0); t.apply_range(1, 3, 5); t.to_list() == [0, 5, 5]`
    pub fn apply_range(mut self, lo: Int, hi: Int, f: F) = todo

    /// Apply tag `f` to the elements at the positions of `r`; `apply_range` with a range.
    ///
    /// cost: O(log n) expected
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `0..len`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(2); t.apply(1..2, 10); t.to_list() == [1, 12]`
    pub fn apply(mut self, r: Range<Int>, f: F) = todo

    /// Reverse the order of the elements at positions `r`. Tags applied earlier keep their effect
    /// on the elements they reached; a later tag sees the new order.
    ///
    /// cost: O(log n) expected
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `0..len`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(2); t.push(3); t.reverse(0..2); t.to_list() == [2, 1, 3]`
    pub fn reverse(mut self, r: Range<Int>) = todo

    /// Remove the element at 0-based position `pos` and return it with every pending tag applied.
    ///
    /// cost: O(log n) expected
    /// traps: when `pos < 0` or `pos >= len`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(2); t.apply_all(5); t.erase_at(0) == 6 and t.to_list() == [7]`
    pub fn erase_at(mut self, pos: Int) -> T = todo

    /// Remove the elements at positions `r`; later elements move down by `r.len`.
    ///
    /// cost: O(log n) expected
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `0..len`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(2); t.push(3); t.erase_range(0..2); t.to_list() == [3]`
    pub fn erase_range(mut self, r: Range<Int>) = todo

    /// Split off the elements at positions `k..len` into a new treap with the same `apply`,
    /// `compose`, `split` and `before`, and return it; `self` keeps positions `0..k`. Pending
    /// tags travel with the elements they cover.
    ///
    /// cost: O(log n) expected
    /// traps: when `k < 0` or `k > len`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(2); t.push(3); let u = t.split_at(1); t.to_list() == [1] and u.to_list() == [2, 3]`
    pub fn split_at(mut self, k: Int) -> LazyTreap<T, F> = todo

    /// Move every element of `other` onto the end of `self`, in order, leaving `other` empty. Both
    /// treaps must use the same functions; `self`'s are kept.
    ///
    /// cost: O(log n) expected
    /// empty: an empty `other` changes nothing
    /// traps: when the total reaches `2^31` elements
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); var u = t.split_at(1); u.push(5); t.append(u); t.to_list() == [1, 5] and u.is_empty`
    pub fn append(mut self, mut other: LazyTreap<T, F>) = todo

    /// The element at 0-based position `pos` with every pending tag on its path applied. Tags are
    /// pushed down along the path, which is why it takes `mut self`; the sequence does not change.
    ///
    /// cost: O(log n) expected
    /// traps: when `pos < 0` or `pos >= len`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(2); t.apply(0..1, 4); t.get(0) == 5`
    pub fn get(mut self, pos: Int) -> T = todo

    // A read leaves the tree as it is: the tags still pending above a node
    // ride along as `pend` instead of being pushed into it.
    fn walk(self, u: ?Int, pend: ?F, lim: Int, mut out: List<T>) = todo

    /// The first `min(k, len)` elements in order, with pending tags applied. Reads without pushing
    /// tags, so the tree is unchanged.
    ///
    /// cost: O(k + log n)
    /// empty: `[]` when `k == 0` or the treap is empty
    /// order: sequence order
    /// traps: when `k < 0`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.push(2); t.push(3); t.take(2) == [1, 2]`
    /// example: `var t = LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f); t.push(1); t.take(5) == [1]`
    pub fn take(self, k: Int) -> List<T> = todo

    /// Every element in order, with pending tags applied; `take(len)`.
    ///
    /// cost: O(n)
    /// empty: `[]`
    /// order: sequence order
    /// traps: never
    /// example: `LazyTreap<Int, Int>(apply: (v, k, f) => v + f, compose: (g, f) => g + f).to_list() == []`
    pub fn to_list(self) -> List<T> = todo

/// O(n) space. Implicit-key rope over a domain `D` supporting O(log n) expected cut, paste,
/// reverse, range tag `apply`, path queries, and point `get`.
pub struct Rope<D, T>:
    n: Int

    /// A rope over `D` holding the values of `a`, or `fill` at every position. `merge(tag, x)` is
    /// how a tag from `apply` combines into a value; without it, a tag overwrites the values it covers.
    /// Positions keep their domain `D`; `reverse` moves values, not positions.
    ///
    /// cost: O(|D|)
    /// empty: an empty `D` builds an empty rope
    /// order: positions keep the domain `D` (of `a`, when given)
    /// traps: when neither or both of `a` and `fill` are given
    /// example: `Rope([1, 2, 3]).to_array() == [1, 2, 3]`
    /// example: `Rope<0..2, Int>(fill: 7).get(1) == 7`
    pub fn new(a: ?[D]T = None, fill: ?T = None, merge: ?((T, T) -> T) = None) -> Rope<D, T> = todo

    /// Apply `tag` to every value at positions `r`: each becomes `merge(tag, x)`, or `tag` when
    /// no `merge` was given. Stored lazily on the covering nodes.
    ///
    /// cost: O(log n) expected
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var r = Rope([1, 2, 3], merge: (t, x) => t + x); r.apply(1..3, 10); r.to_array() == [1, 12, 13]`
    /// example: `var r = Rope([1, 2, 3]); r.apply(0..2, 7); r.to_array() == [7, 7, 3]`
    pub fn apply(mut self, r: Range<D>, tag: T) = todo

    /// Reverse the order of the values at positions `r`; pending tags stay with their values.
    ///
    /// cost: O(log n) expected
    /// empty: changes nothing
    /// traps: when a non-empty `r` reaches outside `D`
    /// example: `var r = Rope([1, 2, 3]); r.reverse(0..3); r.get(0) == 3`
    pub fn reverse(mut self, r: Range<D>) = todo

    /// Walk from the root to the node holding position `i` and return the first `Some` that `f`
    /// gives on a pending tag there, top-down, or `None` when no pending tag gives one. Pending
    /// tags on the path all cover `i`, newer above older; which tags are still pending depends on
    /// the tree's shape, so use an `f` whose first `Some` does not (such as "the latest tag").
    ///
    /// cost: O(log n) expected calls of `f`
    /// traps: when `i` is outside `D`
    /// example: `var r = Rope([1, 2, 3]); r.apply(0..3, 7); r.query_path(1, t => Some(t)) == Some(7)`
    /// example: `Rope([1, 2, 3]).query_path(0, t => Some(t)) == None`
    pub fn query_path<R>(mut self, i: D, f: T -> ?R) -> ?R = todo

    /// The value currently at position `i`, with pending tags merged in.
    ///
    /// cost: O(log n) expected
    /// traps: when `i` is outside `D`
    /// example: `var r = Rope([1, 2, 3]); r.reverse(1..3); r.get(1) == 3`
    pub fn get(mut self, i: D) -> T = todo

    /// Overwrite the value currently at position `i` with `x`, replacing any pending tags on it.
    ///
    /// cost: O(log n) expected
    /// traps: when `i` is outside `D`
    /// example: `var r = Rope([1, 2, 3]); r.set(1, 9); r.to_array() == [1, 9, 3]`
    pub fn set(mut self, i: D, x: T) = todo

    /// The current values in position order, with every pending tag merged in.
    ///
    /// cost: O(|D|)
    /// empty: an empty array for an empty `D`
    /// order: position order of `D`
    /// traps: never
    /// example: `var r = Rope([1, 2, 3]); r.reverse(0..2); r.to_array() == [2, 1, 3]`
    pub fn to_array(mut self) -> [D]T = todo

@test
fn lazy_treap_slope_trick_and_implicit_ops_match_brute() = todo

@test
fn flip_action_toggles_ranges() = todo
