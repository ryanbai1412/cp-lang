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

    /// The monoid a structure was handed: a `Monoid` is itself, and a bare operator brings its
    /// identity: zero for `+`, `^`, `|` and `gcd`, one for `*`, all bits for `&`, and for `min` /
    /// `max` the far end of `T` (`None` on a `?T`). Any other fn has no known identity and needs
    /// `Monoid(id, op)`.
    ///
    /// cost: O(1)
    /// traps: never (an operator without a known identity is a compile-time error)
    /// example: `Monoid<Int>.of(+).id == 0`
    /// example: `Monoid<Int>.of(*).op(3, 4) == 12`
    /// example: `Monoid<?Int>.of(min).id == None`
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

    /// A lazy action of tags `F` on values `T` folded with `over`. `id` is the no-op tag,
    /// `compose(new_f, old_f)` applies `old_f` first and then `new_f`, and `apply(f, val, span)`
    /// is the folded value of `span` leaves, `val`, after `f`. `id`, `compose` and `over` default
    /// to `F.zero`, `F`'s `+` and `T`'s `+`, so an additive tag on sums needs only `apply`.
    ///
    /// cost: O(1)
    /// traps: never (laws are not checked here; use `laws_hold`)
    /// example: `Action<Int, Int>(apply: (d, x, span) => x + d * span).apply(2, 10, 3) == 16`
    /// example: `Action<Int, Int>(apply: (d, x, span) => x + d * span).compose(2, 5) == 7`
    pub fn new<M>(id: F = F.zero, compose: (F, F) -> F = (+), apply: (F, T, Int) -> T, over: M = (+)) -> Action<T, F> = todo

/// `x` with its order reversed. Build it with `desc(x)`.
pub struct Desc<T>(x: T)

impl Ord for Desc<T>:
    fn lt(a, b) = todo

impl Show for Desc<T>:
    fn show(a) = todo

/// `x` wrapped so that it compares the other way round: `desc(a) < desc(b)` exactly when
/// `b < a`. For one component of a sort key or a heap entry; shows as `x` does.
///
/// cost: O(1)
/// traps: never
/// example: `[3, 1, 2].max(by: x => desc(x)) == Some(1)`
/// example: `(desc(3) < desc(5)) == false`
pub fn desc<T>(x: T) -> Desc<T> = todo

/// A folded value that can be read from the other end. A fold along a tree path walks some
/// stretches upwards; `Rooted.fold_path` reverses those with `rev`. A type that does not
/// implement it is its own reverse, which is right for every commutative operation.
pub trait Reversible:
    fn rev(self) -> Self = todo

/// `xs` folded left to right with `m`: `op(op(op(id, x0), x1), x2)`, so a non-commutative `m`
/// sees elements in sequence order. `m` is anything `Monoid.of` accepts.
///
/// cost: O(n) calls of `op`
/// empty: the identity of `m`
/// traps: never (an `m` without a known identity is a compile-time error)
/// example: `fold(+, [1, 2, 3]) == 6`
/// example: `fold(+, 0..0) == 0`
/// example: `fold(Affine.monoid(), [Affine(2, 0), Affine(1, 1)]).eval(5) == 11`
pub fn fold<T, M, S: Seq>(m: M, xs: S) -> T = todo

/// Maximum non-empty subarray summary (Kadane monoid).
/// Concatenation (`+` or `MaxSub.join`) is associative with identity `MaxSub.empty` in O(1);
/// `s.scale(k)` scales every element by non-negative `k` in O(1).
/// Implements `Reversible` (swapping `pre` and `suf`) for tree-path folds.
pub struct MaxSub(has: Bool, best: Int, total: Int, pre: Int, suf: Int):
    /// The summary of no elements: the identity of `join`. It has no best subarray.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `MaxSub.empty().try_best == None`
    pub fn empty() -> MaxSub = todo

    /// The summary of the single element `v`: best, total, best prefix and best suffix are all `v`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `MaxSub.of(-3).try_best == Some(-3)`
    pub fn of(v: Int) -> MaxSub = todo

    /// The summary of `a`'s elements followed by `b`'s (the same as `a + b`). Associative, with
    /// `MaxSub.empty()` as identity; the best subarray may cross the seam (`a.suf + b.pre`).
    ///
    /// cost: O(1)
    /// traps: never (overflow wraps in release, traps in debug)
    /// example: `MaxSub.join(MaxSub.of(2), MaxSub.of(-1)).try_best == Some(2)`
    /// example: `fold(MaxSub.monoid(), [MaxSub.of(v) for v in [2, -5, 3, 1]]).try_best == Some(4)`
    pub fn join(a: MaxSub, b: MaxSub) -> MaxSub = todo

    /// The summary of the same elements each multiplied by `k` (the same as `self * k`). Exact
    /// because every maximum scales by a non-negative factor; the empty summary stays empty.
    ///
    /// cost: O(1)
    /// traps: when `k < 0`
    /// example: `MaxSub.of(-3).scale(2).try_best == Some(-6)`
    /// example: `MaxSub.of(-3).scale(0).try_best == Some(0)`
    pub fn scale(self, k: Int) -> MaxSub = todo

    /// The largest sum of a non-empty contiguous run of the summarized elements, or `None` for
    /// the empty summary.
    ///
    /// cost: O(1)
    /// empty: `None`
    /// traps: never
    /// example: `MaxSub.join(MaxSub.of(-1), MaxSub.of(-2)).try_best == Some(-1)`
    pub @get fn try_best(self) -> ?Int = todo

    /// `Monoid(MaxSub.empty(), MaxSub.join)`, for `SegTree` and `fold`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `MaxSub.monoid().id.try_best == None`
    pub fn monoid() -> Monoid<MaxSub> = todo

impl Add for MaxSub:
    fn add(a, b) = todo

/// O(1). Scale every underlying value by `k >= 0`.
impl Mul<Int> for MaxSub:
    fn mul(self, k: Int) -> MaxSub = todo

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

    /// The group a structure was handed: a `Group` is itself, `+` brings zero and negation, `^`
    /// brings zero and itself as inverse. Any other fn needs `Group(id, op, inv)`.
    ///
    /// cost: O(1)
    /// traps: never (an operator without a known inverse is a compile-time error)
    /// example: `Group<Int>.of(+).inv(5) == -5`
    /// example: `Group<Int>.of(^).inv(5) == 5`
    pub fn of<G>(g: G) -> Group<T> = todo

/// An `Action` whose `apply` ignores the span: `apply(f, val, span) == apply(f, val)`. Right
/// for tags on idempotent folds such as `min`, `max`, `gcd`, where `n` changed leaves fold to
/// one changed value.
///
/// cost: O(1)
/// traps: never
/// example: `point_action(id: 0, compose: (a, b) => a + b, apply: (f, x) => x + f, over: max).apply(3, 10, 4) == 13`
pub fn point_action<T, F, M>(
    id: F,
    compose: (F, F) -> F,
    apply: (F, T) -> T,
    over: M,
) -> Action<T, F> = todo

/// O(1) per merge. Minimum value together with its multiplicity, for counting zeros or
/// minimum-covered cells under range add (`AddMinCount`).
pub struct MinCount<T>(val: T, count: Int):
    /// The summary of the single element `v`: value `v`, count 1.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `MinCount.of(5).count == 1`
    pub fn of(v: T) -> MinCount<T> = todo

    /// The smaller value with its count; on equal values, the value with both counts summed.
    ///
    /// cost: O(1)
    /// ties: equal values merge; their counts add
    /// traps: never
    /// example: `MinCount.join(MinCount.of(2), MinCount.of(2)).count == 2`
    /// example: `MinCount.join(MinCount(1, 3), MinCount(2, 5)).val == 1`
    pub fn join(a: MinCount<T>, b: MinCount<T>) -> MinCount<T> = todo

    /// `Monoid(id, MinCount.join)` with identity value the far end of `T` (as `Monoid.of(min)`)
    /// and count 0, for `SegTree` and `fold`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `fold(MinCount.monoid(), [MinCount.of(v) for v in [3, 1, 1]]).count == 2`
    pub fn monoid() -> Monoid<MinCount<T>> = todo

/// O(1) per merge. Maximum value together with its multiplicity (range add: `SegTree(MaxCount.monoid, a, act: add)`).
pub struct MaxCount<T>(val: T, count: Int):
    /// The summary of the single element `v`: value `v`, count 1.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `MaxCount.of(5).count == 1`
    pub fn of(v: T) -> MaxCount<T> = todo

    /// The larger value with its count; on equal values, the value with both counts summed.
    ///
    /// cost: O(1)
    /// ties: equal values merge; their counts add
    /// traps: never
    /// example: `MaxCount.join(MaxCount(4, 1), MaxCount(4, 2)).count == 3`
    pub fn join(a: MaxCount<T>, b: MaxCount<T>) -> MaxCount<T> = todo

    /// `Monoid(id, MaxCount.join)` with identity value the low end of `T` (as `Monoid.of(max)`)
    /// and count 0, for `SegTree` and `fold`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `fold(MaxCount.monoid(), [MaxCount.of(v) for v in [3, 1, 3]]).count == 2`
    pub fn monoid() -> Monoid<MaxCount<T>> = todo

/// O(1) per composition/eval. Affine map `x -> a * x + b` over any ring `T` (`Int`, `Mod<P>`).
pub struct Affine<T>(a: T, b: T):
    /// The identity map `x -> 1 * x + 0`.
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `Affine<Int>.id().eval(7) == 7`
    pub fn id() -> Affine<T> = todo

    /// `a * x + b`.
    ///
    /// cost: O(1)
    /// traps: never (overflow wraps in release, traps in debug)
    /// example: `Affine(2, 3).eval(5) == 13`
    pub fn eval(self, x: T) -> T = todo

    /// `f` first, then `g`: the map `x -> g.eval(f.eval(x))`, which is
    /// `Affine(g.a * f.a, g.a * f.b + g.b)`.
    ///
    /// cost: O(1)
    /// traps: never (overflow wraps in release, traps in debug)
    /// example: `Affine.then(Affine(2, 0), Affine(1, 1)).eval(5) == 11`
    /// example: `Affine.then(Affine(1, 1), Affine(2, 0)).eval(5) == 12`
    pub fn then(f: Affine<T>, g: Affine<T>) -> Affine<T> = todo

    /// `Monoid(Affine.id(), Affine.then)`: a fold applies the leftmost map first, so a segment tree
    /// over maps answers "push `x` through positions `l..r` in order".
    ///
    /// cost: O(1)
    /// traps: never
    /// example: `fold(Affine.monoid(), [Affine(1, 1), Affine(2, 0)]).eval(5) == 12`
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

/// The three-way comparison of `a` and `b` under `Ord`.
///
/// cost: O(1) plus one or two comparisons of `T`
/// traps: never
/// example: `cmp(1, 2) == Ordering.Less`
/// example: `cmp("b", "a") == Ordering.Greater`
/// example: `cmp(3, 3) == Ordering.Equal`
pub fn cmp<T: Ord>(a: T, b: T) -> Ordering = todo

/// O(1). A semiring from two operation values: `Semiring(min, +)`, `Semiring(or, and)`.
/// `add` is a commutative monoid, `mul` a monoid that distributes over it; zero and one are
/// their identities. Matrix products (`Mat(over:)`), path algebras and `convolve(over:)` take
/// it. `Semiring(+, *)` is the default wherever one is accepted.
pub struct Semiring<A, M>(add: A, mul: M)

/// An associative operation, possibly without an identity (round 5). Bare operators
/// (`+ * ^ | & min max gcd and or`) implement it. Folds over an `Op` of an empty range
/// return `None`. Law (promised, sampled by stress): `op(op(a, b), c) == op(a, op(b, c))`.
pub trait Op<T>:
    fn op(self, a: T, b: T) -> T

/// The identity of an operation. Law: `op(id, a) == a == op(a, id)`.
pub trait HasId<T>:
    @get fn id(self) -> T

/// The inverse of an operation. Law: `op(a, inv(a)) == id`.
pub trait HasInv<T>:
    fn inv(self, a: T) -> T

/// Law: `op(a, b) == op(b, a)`. Lets folds reorder and tags skip push-down.
pub trait Commutative<T>: Op<T>

/// Law: `op(a, a) == a`. Lets `SparseTable` overlap halves.
pub trait Idempotent<T>: Op<T>

/// Law: `op(a, b)` is `a` or `b`. Lets sliding folds use a monotone deque.
pub trait Selective<T>: Idempotent<T>

/// One operation per component of a tuple (positional) or record (named): `op((a0, a1),
/// (b0, b1)) == (m0(a0, b0), m1(a1, b1))`. The identity is the tuple of identities, and the
/// result keeps only the laws every component has.
///
/// cost: O(1) to build; each call costs the sum of its components
/// traps: never
/// example: `fold(each(+, max), [(1, 5), (2, 3)]) == (3, 5)`
pub fn each<Ms>(..ms: Ms) -> Ms = todo

/// Stock action: add a tag to every element. On `+` a span of `n` changes by `f * n`; on
/// `min`/`max` by `f`. Tags commute, so a `SegTree` with it never pushes down. O(1).
pub const add = todo

/// Stock action: assign a tag to every element (`?T` tags, `None` is no-op). O(1) on `+`,
/// `min`, `max`; O(log n) folding `f` `n` times for other ops.
pub const assign = todo

/// Stock action: `x -> a * x + b` on `+` folds (span `n`: `a * s + b * n`). Not commutative.
pub const affine = todo

/// Stock action: swap the two halves of a `Flip<T>` value.
pub const flip = todo
