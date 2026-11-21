//! Operations as values: what a range structure folds with.
//!
//! A structure never asks "what is the sum of two `T`s"; it is handed the
//! operation. Most of the time that is a bare operator: `SegTree<V, Int>(max, a)`
//! and `SegTree<V, Int>(+, a)` are the same tree with two operations. An operation
//! of your own is the pair `Monoid(id, op)`; a lazy update is an `Action`.
//! `desc(x)` reverses an order inside a sort key or a heap entry, and a
//! non-commutative value says how it reads backwards with `Reversible`.
use std.math.{gcd}

/// An associative operation with an identity: the pair `(id, op)`, written
/// `Monoid<T>(id, op)` or, naming both, `Monoid<T>(id: e, op: (a, b) => ...)`.
/// `op(a, b)` folds `a` on the left of `b`, which matters when it does not commute.
/// The laws are checked by `laws_hold(m, samples)`.
pub struct Monoid<T>(id: T, op: (T, T) -> T):
    law assoc(a: T, b: T, c: T) = op(op(a, b), c) == op(a, op(b, c))
    law ident(a: T) = op(id, a) == a and op(a, id) == a

    /// What a structure was handed, as a monoid: a `Monoid` is itself, and a bare operator
    /// brings its identity (`+ ^ | gcd`: zero, `*`: one, `&`: all bits, `min` / `max`: the far
    /// end of `T`, which is `none` for a `?T`). Any other fn needs `Monoid(id, op)`.
    pub fn of<M>(m: M) -> Monoid<T> = todo

fn same<A, B>(a: A, b: B) -> Bool = todo

/// A lazy update `F` on values `T` folded with `over`. Name every argument, in any order, so
/// the roles read at the call site:
///
///     Action<Int, (Int, Int)>(
///         id: (1, 0),
///         compose: (new_f, old_f) => (new_f.0 * old_f.0, new_f.0 * old_f.1 + new_f.1),
///         apply: (f, val, span) => f.0 * val + f.1 * span,
///         over: +,
///     )
///
/// - `id` is the no-op tag (`apply(id, x, len) == x`).
/// - `compose(new_f, old_f)` applies `old_f` first and then `new_f`.
/// - `apply(f, val, span)` returns the folded value `val` of `span` covered leaves after applying `f`.
///
/// The positional form `Action(id, compose, apply, over: op)` is the same call. A flipped
/// `compose` still type-checks, so check a hand-written action once with
/// `laws_hold(act, T: [...], F: [...])`.
pub struct Action<T, F>(id: F, compose: (F, F) -> F, apply: (F, T, Int) -> T, over: Monoid<T>):
    law ident(a: T) = apply(id, a, 1) == a
    // `compose(f, g)` applies `g` first, then `f`.
    law composes(f: F, g: F, a: T) = apply(compose(f, g), a, 1) == apply(f, apply(g, a, 1), 1)

    /// Construct a lazy `Action<T, F>` from `id`, `compose`, `apply`, and monoid `over`. O(1).
    pub fn new<M>(id: F, compose: (F, F) -> F, apply: (F, T, Int) -> T, over: M) -> Action<T, F> = todo

/// `x` with its order reversed. Build it with `desc(x)`.
pub struct Desc<T>(x: T)

impl Ord for Desc<T>:
    fn lt(a, b) = todo

impl Show for Desc<T>:
    fn show(a) = todo

/// `x` ordered the other way round, for one part of a sort key or a heap entry:
/// `sort(mut es, by: e => (e.w, desc(e.name)))`, `MinHeap<(Int, Desc<V>)>()`.
pub fn desc<T>(x: T) -> Desc<T> = todo

/// A folded value that can be read from the other end. A fold along a tree path walks some
/// stretches upwards; `Rooted.fold_path` reverses those with `rev`. A type that does not
/// implement it is its own reverse, which is right for every commutative operation.
pub trait Reversible:
    fn rev(self) -> Self = todo

/// `xs` folded left to right with `m`. Works on any `Seq`. O(n).
pub fn fold<T, M, S: Seq>(m: M, xs: S) -> T = todo

/// Maximum non-empty subarray summary (Kadane monoid).
/// Concatenation (`+` or `MaxSub.join`) is associative with identity `MaxSub.empty` in O(1);
/// `s.scale(k)` scales every element by non-negative `k` in O(1).
/// Implements `Reversible` (swapping `pre` and `suf`) for tree-path folds.
pub struct MaxSub(has: Bool, best: Int, total: Int, pre: Int, suf: Int):
    /// Identity element representing an empty subarray. O(1).
    pub fn empty() -> MaxSub = todo

    /// Single-element summary for `v`. O(1).
    pub fn of(v: Int) -> MaxSub = todo

    /// Concatenate two adjacent summaries left to right. O(1).
    pub fn join(a: MaxSub, b: MaxSub) -> MaxSub = todo

    /// Scale every underlying value by `k >= 0`. O(1).
    pub fn scale(self, k: Int) -> MaxSub = todo

    /// Best non-empty subarray sum, or `none` when empty. O(1).
    pub @get fn try_best(self) -> ?Int = todo

    /// The `Monoid<MaxSub>` value for `SegTree` and `fold`. O(1).
    pub fn monoid() -> Monoid<MaxSub> = todo

impl Add for MaxSub:
    fn add(a, b) = todo

impl Reversible for MaxSub:
    fn rev(self) -> MaxSub = todo

/// O(1) construction. Associative operation without an identity, for non-empty folds
/// (`SparseTable`, `DisjointSparseTable`, `SegTree.try_fold`).
pub struct Semigroup<T>(op: (T, T) -> T):
    law assoc(a: T, b: T, c: T) = op(op(a, b), c) == op(a, op(b, c))

/// O(1) construction. Associative operation with identity and inverse (`inv`),
/// for generalized Fenwick trees and difference structures.
pub struct Group<T>(id: T, op: (T, T) -> T, inv: T -> T):
    law assoc(a: T, b: T, c: T) = op(op(a, b), c) == op(a, op(b, c))
    law ident(a: T) = op(id, a) == a and op(a, id) == a
    law inverse(a: T) = op(a, inv(a)) == id and op(inv(a), a) == id

    /// O(1). Lift a `Group` or bare operator (`+`, `^`) to a `Group<T>`.
    pub fn of<G>(g: G) -> Group<T> = todo

/// O(1). Build an `Action<T, F>` whose `apply(f, val)` does not depend on node span.
pub fn point_action<T, F, M>(
    id: F,
    compose: (F, F) -> F,
    apply: (F, T) -> T,
    over: M,
) -> Action<T, F> = todo

/// O(1) per merge. Minimum value together with its multiplicity, for counting zeros or
/// minimum-covered cells under range add (`AddMinCount`).
pub struct MinCount<T>(val: T, count: Int):
    /// O(1). Single-element summary for `v`.
    pub fn of(v: T) -> MinCount<T> = todo

    /// O(1). Combine two summaries, summing counts when minima tie.
    pub fn join(a: MinCount<T>, b: MinCount<T>) -> MinCount<T> = todo

    /// O(1). The `Monoid<MinCount<T>>` value for `SegTree` and `LazySegTree`.
    pub fn monoid() -> Monoid<MinCount<T>> = todo

/// O(1) per merge. Maximum value together with its multiplicity (`AddMaxCount`).
pub struct MaxCount<T>(val: T, count: Int):
    /// O(1). Single-element summary for `v`.
    pub fn of(v: T) -> MaxCount<T> = todo

    /// O(1). Combine two summaries, summing counts when maxima tie.
    pub fn join(a: MaxCount<T>, b: MaxCount<T>) -> MaxCount<T> = todo

    /// O(1). The `Monoid<MaxCount<T>>` value for `SegTree` and `LazySegTree`.
    pub fn monoid() -> Monoid<MaxCount<T>> = todo

/// O(1) per merge. Value paired with its witness position `at: D` for leftmost-tie argmin folds.
pub struct MinIdx<D, T>(val: T, at: D):
    /// O(1). Combine two `(val, at)` entries, keeping the smaller value (leftmost on tie).
    pub fn join(a: MinIdx<D, T>, b: MinIdx<D, T>) -> MinIdx<D, T> = todo

/// O(1) per merge. Value paired with its witness position `at: D` for leftmost-tie argmax folds.
pub struct MaxIdx<D, T>(val: T, at: D):
    /// O(1). Combine two `(val, at)` entries, keeping the larger value (leftmost on tie).
    pub fn join(a: MaxIdx<D, T>, b: MaxIdx<D, T>) -> MaxIdx<D, T> = todo

/// O(1) per composition/eval. Affine map `x -> a * x + b` over any ring `T` (`Int`, `Mod<P>`).
pub struct Affine<T>(a: T, b: T):
    /// O(1). Identity map `x -> x`.
    pub fn id() -> Affine<T> = todo

    /// O(1). Evaluate `a * x + b`.
    pub fn eval(self, x: T) -> T = todo

    /// O(1). Compose `g` after `f`: `x -> g.eval(f.eval(x))`.
    pub fn then(f: Affine<T>, g: Affine<T>) -> Affine<T> = todo

    /// O(1). The left-to-right composition `Monoid<Affine<T>>` (`SegTree` point-set + range-composite).
    pub fn monoid() -> Monoid<Affine<T>> = todo

@test
fn bare_operators_bring_their_identity() = todo

@test
fn a_monoid_of_your_own_is_a_pair() = todo

@test
fn affine_action_laws() = todo

@test
fn named_arguments_build_the_same_monoid_and_action() = todo

@test
fn desc_reverses_one_part_of_a_key() = todo

struct Word(s: Str)

impl Reversible for Word:
    fn rev(self) -> Word = todo

@test
fn plain_values_are_their_own_reverse() = todo

@test
fn max_sub_monoid_laws_and_kadane() = todo

/// O(1). Three-way ordering result returned by `cmp`.
pub enum Ordering:
    Less
    Equal
    Greater

/// O(1). Three-way comparison of `a` and `b`.
pub fn cmp<T: Ord>(a: T, b: T) -> Ordering = todo

/// O(1). Semiring `(zero, one, add, mul)` for generic matrix products and path algebras.
pub struct Semiring<T>(zero: T, one: T, add: (T, T) -> T, mul: (T, T) -> T):
    /// O(1). Tropical `(min, +)` semiring on `Int`.
    pub fn min_plus() -> Semiring<Int> = todo

    /// O(1). Tropical `(max, +)` semiring on `Int`.
    pub fn max_plus() -> Semiring<Int> = todo

    /// O(1). Boolean `(or, and)` semiring.
    pub fn bool_ring() -> Semiring<Bool> = todo

