//! Operations as values: what a range structure folds with.
//!
//! start: `desc`, `pow`, `Monoid`, `Action`, `MaxSub`
//! example: `[3, 1, 2].max(by: x => desc(x)) == Some(1)`
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
pub struct Monoid<T>(pub id: T, pub op: (T, T) -> T, pub rev: T -> T = unreversed):
    law assoc(a: T, b: T, c: T) where T: Eq = op(op(a, b), c) == op(a, op(b, c))
    law ident(a: T) where T: Eq = op(id, a) == a and op(a, id) == a

    /// The monoid a structure was handed: what its [`FromOp`] impl gives. A `Monoid` is itself,
    /// and a bare operator brings its identity: zero for `+`, `^`, `|` and `gcd`, one for `*`,
    /// all bits for `&`.
    ///
    /// Any other fn has no known identity and needs `Monoid(id, op)`.
    ///
    /// cost: \(O(1)\)
    /// traps: never (an operator without a known identity is a compile-time error)
    /// example: `Monoid<Int>.of(+).id == 0`
    /// example: `Monoid<Int>.of(*).op(3, 4) == 12`
    @from_op
    pub fn of<M: FromOp<T>>(m: M) -> Monoid<T> = m.monoid()

    /// `x`, a fold of some elements, as the fold of the same elements read backwards: by `rev`,
    /// which `Monoid.of` gives every stock op (all commutative), `.commutative()` and a
    /// `Reversible` value's own `monoid()`.
    ///
    /// cost: \(O(1)\) plus one call of `rev`
    /// traps: when the monoid was made without `rev:`
    /// example: `Monoid<Int>.of(+).reversed(5) == 5`
    /// trap: `Monoid<Int>(0, +).reversed(5)`
    pub fn reversed(self, x: T) -> T = rev(x)

    /// `self` with the law `op(a, b) == op(b, a)` stated, so folds may reorder and lazy tags
    /// skip push-down; `cpl stress` samples the law like any other.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Monoid<Int>(0, (a, b) => a + b).commutative().op(2, 3) == 5`
    pub fn commutative(self) -> Comm<T> = Comm(self)

/// A monoid whose `op(a, b) == op(b, a)` is stated; [`Monoid.commutative`] makes one.
pub struct Comm<T>(pub m: Monoid<T>):
    law comm(a: T, b: T) where T: Eq = m.op(a, b) == m.op(b, a)


impl<T> Op<T> for Comm<T>:
    type Fold = T
    fn op(self, a: T, b: T) -> T = m.op(a, b)
    fn folds(self) -> Monoid<T> = m
    fn lift(self, x: T) -> T = x

impl<T> HasId<T> for Comm<T>:
    @get fn id(self) = m.id

impl<T> Commutative<T> for Comm<T>

/// `T.ones`: every bit set, the identity of `T`'s `&`.
pub trait Ones:
    @get fn ones() -> Self
impl Ones for Int:
    @get fn ones() -> Int = -1
impl Ones for U64:
    @get fn ones() -> U64 = U64(0) - U64(1)

/// What a structure handed an operation folds with: the operation's monoid. `+`, `*`, `^`,
/// `|`, `&` and `gcd` bring theirs, a `Monoid` is its own, and a type of your own joins with an
/// impl, so `SegTree(op, a)` takes any of them.
pub trait FromOp<T>:
    /// The monoid this operation folds with.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Monoid<Int>(0, (a, b) => a + b).monoid().op(2, 3) == 5`
    /// example: `Monoid<Int>(0, (a, b) => a + b).commutative().monoid().id == 0`
    fn monoid(self) -> Monoid<T>
    /// Whether the operation is `+`, so an addition action scales a tag by the span it covers.
    fn sums(self) -> Bool = false

impl<T> FromOp<T> for Monoid<T>:
    fn monoid(self) -> Monoid<T> = self

impl<T> FromOp<T> for Comm<T>:
    fn monoid(self) -> Monoid<T> = Monoid(m.id, m.op, x => x)

impl<T: Add + Zero> FromOp<T> for type_of(+):
    fn monoid(self) -> Monoid<T> = Monoid(T.zero, (a, b) => a + b, x => x)
    fn sums(self) -> Bool = true

impl<T: Mul + One> FromOp<T> for type_of(*):
    fn monoid(self) -> Monoid<T> = Monoid(T.one, (a, b) => a * b, x => x)

impl<T: BitXor + Zero> FromOp<T> for type_of(^):
    fn monoid(self) -> Monoid<T> = Monoid(T.zero, (a, b) => a ^ b, x => x)

impl<T: BitOr + Zero> FromOp<T> for type_of(|):
    fn monoid(self) -> Monoid<T> = Monoid(T.zero, (a, b) => a | b, x => x)

impl<T: BitAnd + Ones> FromOp<T> for type_of(&):
    fn monoid(self) -> Monoid<T> = Monoid(T.ones, (a, b) => a & b, x => x)

impl FromOp<Bool> for type_of(and):
    fn monoid(self) -> Monoid<Bool> = Monoid(true, (a, b) => a and b, x => x)

impl FromOp<Bool> for type_of(or):
    fn monoid(self) -> Monoid<Bool> = Monoid(false, (a, b) => a or b, x => x)

impl FromOp<Int> for type_of(gcd):
    fn monoid(self) -> Monoid<Int> = Monoid(0, (a, b) => gcd(a, b), x => x)

impl<T: Ord + Bounded> FromOp<T> for type_of(max):
    fn monoid(self) -> Monoid<T> = Monoid(T.MIN, (a, b) => max(a, b), x => x)

impl<T: Ord + Bounded> FromOp<T> for type_of(min):
    fn monoid(self) -> Monoid<T> = Monoid(T.MAX, (a, b) => min(a, b), x => x)

/// What a structure handed an invertible operation folds with: its group. `+` brings zero and
/// negation, `^` zero and itself as inverse, and a `Group` is its own.
pub trait GroupOp<T>:
    /// The group this operation folds with.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Group<Int>(0, (a, b) => a + b, a => -a).group().inv(2) == -2`
    fn group(self) -> Group<T>

impl<T> GroupOp<T> for Group<T>:
    fn group(self) -> Group<T> = self

impl<T: Add + Sub + Zero> GroupOp<T> for type_of(+):
    fn group(self) -> Group<T> = Group(T.zero, (a, b) => a + b, a => T.zero - a)

impl<T: BitXor + Zero> GroupOp<T> for type_of(^):
    fn group(self) -> Group<T> = Group(T.zero, (a, b) => a ^ b, a => a)

fn unreversed<T>(x: T) -> T:
    assert false, "this fold is read backwards but its op has no `rev`: pass `rev:` to `Monoid`, use `.commutative()`, or implement `Reversible`"
    x

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
/// - `commutes: true` promises tags commute (`compose(f, g)` acts as `compose(g, f)`), so a
///   `SegTree` applies a range tag without pushing tags down first.
/// - `replaces: f => ...` says which tags set the value outright: when `replaces(f)` holds,
///   `apply(f, val, span)` is the same for every `val`, as with assignment. A `SegTree` fold
///   then answers from the first such tag it meets instead of reading below it. Leave it out
///   (the default) when no tag does; `id` never does.
/// - `compose_in(mut old_f, new_f)`, when given, sets `old_f` to `compose(new_f, old_f)` in place.
///   Trees compose a pending tag with it (`compose_onto`), so a large tag is updated field by
///   field instead of rebuilt and copied on every update.
///
/// The positional form `Action(id, compose, apply, over: op)` is the same call. A flipped
/// `compose` still type-checks, so check a hand-written action once with
/// `laws_hold(act, T: [...], F: [...])`.
pub struct Action<T, F>(
    pub id: F,
    pub compose: (F, F) -> F,
    pub apply: (F, T, Int) -> T,
    pub over: Monoid<T>,
    pub commutes: Bool,
    pub replaces: ?(F -> Bool) = None,
    pub compose_in: ?((mut F, F) -> ()) = None,
):
    law ident(a: T) where T: Eq = apply(id, a, 1) == a
    law replace(f: F, a: T, b: T) where T: Eq = not (replaces ?? (_ => false))(f) or apply(f, a, 1) == apply(f, b, 1)
    law commute(f: F, g: F, a: T) where T: Eq = not commutes or apply(compose(f, g), a, 1) == apply(compose(g, f), a, 1)
    // `compose(f, g)` applies `g` first, then `f`.
    law composes(f: F, g: F, a: T) where T: Eq = apply(compose(f, g), a, 1) == apply(f, apply(g, a, 1), 1)
    law composes_in_place(f: F, g: F, a: T) where T: Eq = apply(self.composed_in(f, g), a, 1) == apply(compose(f, g), a, 1)

    /// A lazy action of tags `F` on values `T` folded with `over`.
    ///
    /// `id` is the no-op tag, `compose(new_f, old_f)` applies `old_f` first and then `new_f`, and
    /// `apply(f, val, span)` is the folded value of `span` leaves, `val`, after `f`. `id`,
    /// `compose` and `over` default to `F.zero`, `F`'s `+` and `T`'s `+`, so an additive tag on
    /// sums needs only `apply`.
    ///
    /// cost: \(O(1)\)
    /// traps: never (laws are not checked here; use `laws_hold`)
    /// example: `Action<Int, Int>(apply: (d, x, span) => x + d * span).apply(2, 10, 3) == 16`
    /// example: `Action<Int, Int>(apply: (d, x, span) => x + d * span).compose(2, 5) == 7`
    pub fn new<M: FromOp<T>>(
        id: F = F.zero,
        compose: (F, F) -> F = (+),
        apply: (F, T, Int) -> T,
        over: M = (+),
        commutes: Bool = false,
        replaces: ?(F -> Bool) = None,
        compose_in: ?((mut F, F) -> ()) = None,
    ) -> Action<T, F>:
        Action(id, compose, apply, Monoid<T>.of(over), commutes, replaces, compose_in)

    /// Set `old` to `compose(new, old)`: in place through `compose_in` when given.
    ///
    /// cost: \(O(1)\) plus one `compose_in` or `compose` call
    /// traps: never
    /// example: `var f = 2; Action<Int, Int>(apply: (d, x, span) => x + d * span).compose_onto(mut f, 5); f == 7`
    pub fn compose_onto(self, mut old: F, new: F):
        if let c = compose_in: c(mut old, new) else: old = compose(new, old)

    fn composed_in(self, new: F, old: F) -> F:
        var f = old
        self.compose_onto(mut f, new)
        f

    /// Resolve a stock action (`add`, `assign`, `affine`, `flip`) or an `Action` against fold `op`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Action<Int, Int>.of(add, +).apply(2, 10, 3) == 16`
    pub fn of<A: Acts<T, F>, M: FromOp<T>>(a: A, op: M) -> Action<T, F> = a.action(op.monoid(), op.sums())

/// What an `act:` argument is: a stock action or an `Action`, acting with tags `F` on values `T`.
/// The impl fixes `F` from `T`, so `SegTree(+, a, act: assign)` has `?T` tags without naming them.
pub trait Acts<T, F>:
    /// The `Action` this stands for over the fold `over`; `sums` says `over` is `+`, so a tag
    /// on a span of `n` leaves counts `n` times.
    fn action(self, over: Monoid<T>, sums: Bool) -> Action<T, F>

impl<T, F> Acts<T, F> for Action<T, F>:
    fn action(self, over: Monoid<T>, sums: Bool) -> Action<T, F> = self

impl<T> Acts<T, T> for AddAction:
    fn action(self, over: Monoid<T>, sums: Bool) -> Action<T, T> where T: Zero:
        if sums: return Action(T.zero, (nf, of) => nf + of, (f, v, span) => v + f * span, over, true)
        Action(T.zero, (nf, of) => nf + of, (f, v, span) => v + f, over, true)

impl<T> Acts<T, ?T> for AssignAction:
    fn action(self, over: Monoid<T>, sums: Bool) -> Action<T, ?T>:
        let set: ?((?T) -> Bool) = Some(f => f != None)
        if sums: return Action(None, (nf, of) => nf ?? of, (f, v, span) => if let x = f: x * span else: v, over, false, set)
        Action(None, (nf, of) => nf ?? of, (f, v, span) => f ?? v, over, false, set)

impl<T> Acts<T, (T, T)> for AffineAction:
    fn action(self, over: Monoid<T>, sums: Bool) -> Action<T, (T, T)> where T: One + Zero = Action(
        (T.one, T.zero),
        (nf, of) => (nf.0 * of.0, nf.0 * of.1 + nf.1),
        (f, v, span) => f.0 * v + f.1 * span,
        over,
    )

impl<T> Acts<T, (set: ?T, add: T)> for AssignAddAction:
    fn action(self, over: Monoid<T>, sums: Bool) -> Action<T, (set: ?T, add: T)> where T: Zero:
        let keep: ?T = None
        let id = (set: keep, add: T.zero)
        let set: ?((set: ?T, add: T) -> Bool) = Some(f => f.set != None)
        if sums: return Action(id, (nf, of) => if let _ = nf.set: nf else: (set: of.set, add: of.add + nf.add), (f, v, span) => if let x = f.set: x * span + f.add * span else: v + f.add * span, over, false, set)
        Action(id, (nf, of) => if let _ = nf.set: nf else: (set: of.set, add: of.add + nf.add), (f, v, span) => if let x = f.set: x + f.add else: v + f.add, over, false, set)

impl<T> Acts<T, Bool> for FlipAction:
    fn action(self, over: Monoid<T>, sums: Bool) -> Action<T, Bool> =
        Action(false, (nf, of) => nf != of, (f, v, span) => if f: T(v.flipped, v.now) else: v, over, true)

/// `x` with its order reversed. Build it with `desc(x)`.
pub struct Desc<T>(pub x: T)

impl<T: Ord> Ord for Desc<T>:
    fn lt(a, b) = b.x < a.x

impl Show for Desc<T: Show>:
    fn show(a) = "desc({a.x})"

/// `x` wrapped so that it compares the other way round: `desc(a) < desc(b)` exactly when
/// `b < a`. For one component of a sort key or a heap entry; shows as `x` does.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `[3, 1, 2].max(by: x => desc(x)) == Some(1)`
/// example: `(desc(3) < desc(5)) == false`
@lint(sort_desc_by_negation, from: "xs.sorted(by: k => -x)", to: "xs.sorted(by: k => desc(x))", holes: "xs")
@lint(sort_desc_by_negation, from: "xs.sort(by: k => -x)", to: "xs.sort(by: k => desc(x))", holes: "xs")
pub fn desc<T>(x: T) -> Desc<T> = Desc(x)

/// A folded value that can be read from the other end. A fold along a tree path walks some
/// stretches upwards; `Rooted.fold_path` reverses those with its monoid's `rev`, which a
/// `Reversible` value's `monoid()` passes as `x => x.rev()`. Reversal belongs to the op: a
/// commutative one (`.commutative()`, every stock op) reverses as the identity.
pub trait Reversible:
    fn rev(self) -> Self

/// `x` folded with itself `k` times by squaring: `op(x, op(x, ...))`; a plain power is `x ** k`.
///
/// `op` is anything `Monoid.of` accepts, and `k = 0` gives its identity. Powers of one
/// value commute, so any associative `op` works: a number, `Mod<P>`, an `Affine` map
/// (`op: Affine.monoid()`), a permutation (`op: Monoid(id, compose)`).
///
/// cost: \(O(\log k)\) calls of `op`
/// empty: `pow(3, 0, op: *) == 1`
/// traps: when `k < 0`
/// example: `pow(3, 0, op: +) == 0`
/// example: `pow(3, 5, op: +) == 15`
/// example: `pow(Affine(2, 1), 3, op: Affine.monoid()).eval(0) == 7`
pub fn pow<T, M: FromOp<T>>(x: T, k: 0..; op: M) -> T:
    assert k >= 0, "pow: a negative exponent"
    let m = Monoid<T>.of(op)
    var acc = m.id
    var base = x
    var e = k
    while e > 0:
        if e % 2 == 1: acc = m.op(acc, base)
        e /= 2
        if e > 0: base = m.op(base, base)
    acc

/// Maximum non-empty subarray summary (Kadane monoid).
/// Concatenation (`+` or `MaxSub.join`) is associative with identity `MaxSub.zero` in \(O(1)\);
/// `s.scale(k)` scales every element by non-negative `k` in \(O(1)\).
/// Implements `Reversible` (swapping `pre` and `suf`) for tree-path folds.
pub struct MaxSub(has: Bool, pub best: Int, pub total: Int, pub pre: Int, pub suf: Int):
    /// The summary of the single element `v`: best, total, best prefix and best suffix are all `v`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `MaxSub.of(-3).try_best == Some(-3)`
    pub fn of(v: Int) -> MaxSub = MaxSub(true, v, v, v, v)

    /// The summary of `a`'s elements followed by `b`'s (the same as `a + b`). Associative, with
    /// `MaxSub.zero` as identity; the best subarray may cross the seam (`a.suf + b.pre`).
    ///
    /// cost: \(O(1)\)
    /// traps: never (overflow wraps in release, traps in debug)
    /// example: `MaxSub.join(MaxSub.of(2), MaxSub.of(-1)).try_best == Some(2)`
    /// example: `[MaxSub.of(v) for v in [2, -5, 3, 1]].fold(MaxSub.monoid()).try_best == Some(4)`
    pub fn join(a: MaxSub, b: MaxSub) -> MaxSub:
        if not a.has: return b
        if not b.has: return a
        MaxSub(
            true,
            max(a.best, b.best, a.suf + b.pre),
            a.total + b.total,
            max(a.pre, a.total + b.pre),
            max(b.suf, a.suf + b.total),
        )

    /// The summary of the same elements in reverse order: the best prefix and suffix swap.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `(MaxSub.of(3) + MaxSub.of(-5)).reversed() == MaxSub.of(-5) + MaxSub.of(3)`
    pub fn reversed(self) -> MaxSub = MaxSub(has, best, total, suf, pre)

    /// The summary of `k` copies of this run, one after another, in \(O(1)\) where `pow` takes
    /// \(O(\log k)\) joins: what a range assignment of `k` equal elements stores.
    ///
    /// cost: \(O(1)\)
    /// empty: `MaxSub.of(4).repeat(0) == MaxSub.zero`
    /// traps: when `k < 0`
    /// example: `MaxSub.of(-2).repeat(3) == MaxSub.of(-2) + MaxSub.of(-2) + MaxSub.of(-2)`
    /// example: `let s = MaxSub.of(-1) + MaxSub.of(3); s.repeat(3) == s + s + s and s.repeat(3).best == 7`
    pub fn repeat(self, k: 0..) -> MaxSub:
        assert k >= 0, "`repeat` count is negative"
        if k == 0 or not has: return MaxSub.zero
        if k == 1: return self
        let gain = max(total, 0)
        MaxSub(true, max(best, suf + pre + (k - 2) * gain), k * total, pre + (k - 1) * gain, suf + (k - 1) * gain)

    /// The summary of the same elements each multiplied by `k` (the same as `self * k`). Exact
    /// because every maximum scales by a non-negative factor; the empty summary stays empty.
    ///
    /// cost: \(O(1)\)
    /// traps: when `k < 0`
    /// example: `MaxSub.of(-3).scale(2).try_best == Some(-6)`
    /// example: `MaxSub.of(-3).scale(0).try_best == Some(0)`
    pub fn scale(self, k: 0..) -> MaxSub:
        assert k >= 0
        if not has: self
        else: MaxSub(true, best * k, total * k, pre * k, suf * k)

    /// The largest sum of a non-empty contiguous run of the summarized elements, or `None` for
    /// the empty summary.
    ///
    /// cost: \(O(1)\)
    /// empty: `MaxSub.monoid().id.try_best == None`
    /// traps: never
    /// example: `MaxSub.join(MaxSub.of(-1), MaxSub.of(-2)).try_best == Some(-1)`
    @get pub fn try_best(self) -> ?Int = if has: Some(best) else: None

    /// `Monoid(MaxSub.zero, MaxSub.join)`, for `SegTree` and `fold`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `MaxSub.monoid().id.try_best == None`
    pub fn monoid() -> Monoid<MaxSub> = Monoid(MaxSub.zero, (a, b) => MaxSub.join(a, b), x => x.rev())

/// The summary of no elements: the identity of `join`. It has no best subarray.
impl Zero for MaxSub:
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `MaxSub.zero.try_best == None`
    @get fn zero() -> MaxSub = MaxSub(false, 0, 0, 0, 0)
impl Add for MaxSub:
    fn add(a, b) = MaxSub.join(a, b)

/// \(O(1)\). Scale every underlying value by `k >= 0`.
impl Mul<Int> for MaxSub:
    fn mul(self, k: Int) -> MaxSub = self.scale(k)

impl Reversible for MaxSub:
    fn rev(self) -> MaxSub = MaxSub(has, best, total, suf, pre)

/// `MaxSub` that also knows where its best subarray is: build one per element with
/// `MaxSubAt.of(i, a[i])` and join with `+` (or `MaxSubAt.join`), associative with identity
/// `MaxSubAt<D>.zero` in \(O(1)\). The same summary answers range queries in a `SegTree`.
///
/// `join` resolves ties itself, so every grouping of the same elements gives the same `at`.
///
/// ties: `domain D = 0..5; let s = MaxSubAt.of(D(0), 0) + MaxSubAt.of(D(1), 1) + MaxSubAt.of(D(2), -5) + MaxSubAt.of(D(3), 1) + MaxSubAt.of(D(4), 0); s.at == D(0).l..D(1).r`
pub struct MaxSubAt<D>(
    has: Bool,
    pub best: Int,
    pub at: Range<D>,
    total: Int,
    pre: Int,
    pre_end: Cut<D>,
    suf: Int,
    suf_start: Cut<D>,
):
    /// The summary of no elements: the identity of `join`. It has no best subarray.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `domain D = 0..3; MaxSubAt<D>.zero.try_at == None`
    @get pub fn zero() -> MaxSubAt<D>:
        let c = D.cuts.nth(0)
        MaxSubAt(false, 0, c..c, 0, 0, c, 0, c)

    /// The summary of the single element `v` at key `i`: its best subarray is `i` alone.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `domain D = 0..3; MaxSubAt.of(D(1), -3).try_best == Some(-3)`
    pub fn of(i: D, v: Int) -> MaxSubAt<D> = MaxSubAt(true, v, i.l..i.r, v, v, i.r, v, i.l)

    /// The summary of `a`'s elements followed by `b`'s (the same as `a + b`); the best
    /// subarray may cross the seam.
    ///
    /// cost: \(O(1)\)
    /// ties: `domain D = 0..5; let a = MaxSubAt.of(D(0), 0) + MaxSubAt.of(D(1), 1); let b = MaxSubAt.of(D(2), -5) + MaxSubAt.of(D(3), 1) + MaxSubAt.of(D(4), 0); MaxSubAt.join(a, b).at == D(0).l..D(1).r`
    /// traps: never (overflow wraps in release, traps in debug)
    /// example: `domain D = 0..3; MaxSubAt.join(MaxSubAt.of(D(0), 1), MaxSubAt.of(D(1), 0)).at == D(0).l..D(1).r`
    pub fn join(a: MaxSubAt<D>, b: MaxSubAt<D>) -> MaxSubAt<D>:
        if not a.has: return b
        if not b.has: return a
        var (best, at) = (a.best, a.at)
        if wider_best(b.best, b.at, best, at): (best, at) = (b.best, b.at)
        let cross = a.suf_start..b.pre_end
        if wider_best(a.suf + b.pre, cross, best, at): (best, at) = (a.suf + b.pre, cross)
        let longer_pre = a.total + b.pre >= a.pre
        let longer_suf = a.suf + b.total >= b.suf
        MaxSubAt(
            true,
            best,
            at,
            a.total + b.total,
            if longer_pre: a.total + b.pre else: a.pre,
            if longer_pre: b.pre_end else: a.pre_end,
            if longer_suf: a.suf + b.total else: b.suf,
            if longer_suf: a.suf_start else: b.suf_start,
        )

    /// The largest sum of a non-empty contiguous run, or `None` for the empty summary.
    ///
    /// cost: \(O(1)\)
    /// empty: `domain D = 0..3; MaxSubAt<D>.zero.try_best == None`
    /// traps: never
    /// example: `domain D = 0..3; MaxSubAt<D>.zero.try_best == None`
    @get pub fn try_best(self) -> ?Int = if has: Some(best) else: None

    /// Where the best subarray is (widest, then leftmost), or `None` for the empty summary.
    ///
    /// cost: \(O(1)\)
    /// empty: `domain D = 0..3; MaxSubAt<D>.zero.try_at == None`
    /// traps: never
    /// example: `domain D = 0..3; MaxSubAt.of(D(2), 5).try_at == Some(D(2).l..D(2).r)`
    /// order: the range runs left to right
    @get pub fn try_at(self) -> ?Range<D> = if has: Some(at) else: None

    /// `Monoid(MaxSubAt<D>.zero, MaxSubAt.join)`, for `SegTree` and `fold`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `domain D = 0..3; MaxSubAt<D>.monoid().id.try_best == None`
    pub fn monoid() -> Monoid<MaxSubAt<D>> = Monoid(MaxSubAt<D>.zero, (a, b) => MaxSubAt.join(a, b))

impl<D> Add for MaxSubAt<D>:
    fn add(a, b) = MaxSubAt.join(a, b)

fn wider_best<D>(s: Int, r: Range<D>, t: Int, q: Range<D>) -> Bool =
    cmp((s, r.r.ord - r.l.ord, q.l.ord), (t, q.r.ord - q.l.ord, r.l.ord)) == Ordering.Greater

/// \(O(1)\) construction. Associative operation without an identity, for non-empty folds
/// (`SparseTable`, `DisjointSparseTable`, `SegTree.try_fold`).
pub struct Semigroup<T>(pub op: (T, T) -> T):
    law assoc(a: T, b: T, c: T) where T: Eq = op(op(a, b), c) == op(a, op(b, c))

    /// \(O(1)\). The op on `?T` with `None` as identity: a `Monoid` any structure takes.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `[Some(5), Some(2)].fold(Semigroup((a: Int, b: Int) => a - b).opt()) == Some(3)`
    pub fn opt(self) -> Monoid<?T> = op.opt()

/// \(O(1)\) construction. Associative operation with identity and inverse (`inv`),
/// for generalized Fenwick trees and difference structures.
pub struct Group<T>(pub id: T, pub op: (T, T) -> T, pub inv: T -> T):
    law assoc(a: T, b: T, c: T) where T: Eq = op(op(a, b), c) == op(a, op(b, c))
    law ident(a: T) where T: Eq = op(id, a) == a and op(a, id) == a
    law inverse(a: T) where T: Eq = op(a, inv(a)) == id and op(inv(a), a) == id

    /// The group a structure was handed: a `Group` is itself, `+` brings zero and negation, `^`
    /// brings zero and itself as inverse. Any other fn needs `Group(id, op, inv)`.
    ///
    /// cost: \(O(1)\)
    /// traps: never (an operator without a known inverse is a compile-time error)
    /// example: `Group<Int>.of(+).inv(5) == -5`
    /// example: `Group<Int>.of(^).inv(5) == 5`
    pub fn of<G: GroupOp<T>>(g: G) -> Group<T> = g.group()

/// An `Action` whose `apply` ignores the span: `apply(f, val, span) == apply(f, val)`. Right
/// for tags on idempotent folds such as `min`, `max`, `gcd`, where `n` changed leaves fold to
/// one changed value.
///
/// cost: \(O(1)\)
/// traps: never
/// example: `point_action(id: 0, compose: (a, b) => a + b, apply: (f, x) => x + f, over: max).apply(3, 10, 4) == 13`
pub fn point_action<T, F, M: FromOp<T>>(
    id: F,
    compose: (F, F) -> F,
    apply: (F, T) -> T,
    over: M,
) -> Action<T, F>:
    Action(id, compose, (f, v, span) => apply(f, v), Monoid<T>.of(over))

/// \(O(1)\) per merge. Minimum value together with its multiplicity, for counting zeros or
/// minimum-covered cells under range add (`AddMinCount`).
pub struct MinCount<T>(pub val: T, pub count: Int):
    /// The summary of the single element `v`: value `v`, count 1.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `MinCount.of(5).count == 1`
    pub fn of(v: T) -> MinCount<T> = MinCount(v, 1)

    /// The smaller value with its count; on equal values, the value with both counts summed.
    ///
    /// cost: \(O(1)\)
    /// ties: `let m = MinCount.join(MinCount(2, 3), MinCount(2, 4)); (m.val, m.count) == (2, 7)`
    /// traps: never
    /// example: `MinCount.join(MinCount.of(2), MinCount.of(2)).count == 2`
    /// example: `MinCount.join(MinCount(1, 3), MinCount(2, 5)).val == 1`
    pub fn join(a: MinCount<T>, b: MinCount<T>) -> MinCount<T> where T: Ord:
        if a.val < b.val: a elif b.val < a.val: b else: MinCount(a.val, a.count + b.count)

    /// `Monoid(id, MinCount.join)` with identity value the far end of `T` (as `Monoid.of(min)`)
    /// and count 0, for `SegTree` and `fold`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `[MinCount.of(v) for v in [3, 1, 1]].fold(MinCount.monoid()).count == 2`
    pub fn monoid() -> Monoid<MinCount<T>> = Monoid(MinCount.zero, (a, b) => MinCount.join(a, b), x => x)


/// The empty fold, `+`'s identity: the far end of `T` (as `Monoid.of(min)`) with count 0.
impl<T: Ord> Zero for MinCount<T>:
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `MinCount<Int>.zero.count == 0`
    @get fn zero() -> MinCount<T> = MinCount(Monoid<T>.of(min).id, 0)
impl<T> Add for MinCount<T>:
    fn add(a, b) = MinCount.join(a, b)

/// \(O(1)\) per merge. Maximum value together with its multiplicity (range add: `SegTree(MaxCount.monoid, a, act: add)`).
pub struct MaxCount<T>(pub val: T, pub count: Int):
    /// The summary of the single element `v`: value `v`, count 1.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `MaxCount.of(5).count == 1`
    pub fn of(v: T) -> MaxCount<T> = MaxCount(v, 1)

    /// The larger value with its count; on equal values, the value with both counts summed.
    ///
    /// cost: \(O(1)\)
    /// ties: `let m = MaxCount.join(MaxCount(4, 1), MaxCount(4, 2)); (m.val, m.count) == (4, 3)`
    /// traps: never
    /// example: `MaxCount.join(MaxCount(4, 1), MaxCount(4, 2)).count == 3`
    pub fn join(a: MaxCount<T>, b: MaxCount<T>) -> MaxCount<T> where T: Ord:
        if b.val < a.val: a elif a.val < b.val: b else: MaxCount(a.val, a.count + b.count)

    /// `Monoid(id, MaxCount.join)` with identity value the low end of `T` (as `Monoid.of(max)`)
    /// and count 0, for `SegTree` and `fold`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `[MaxCount.of(v) for v in [3, 1, 3]].fold(MaxCount.monoid()).count == 2`
    pub fn monoid() -> Monoid<MaxCount<T>> = Monoid(MaxCount.zero, (a, b) => MaxCount.join(a, b), x => x)


/// The empty fold, `+`'s identity: the low end of `T` (as `Monoid.of(max)`) with count 0.
impl<T: Ord> Zero for MaxCount<T>:
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `MaxCount<Int>.zero.count == 0`
    @get fn zero() -> MaxCount<T> = MaxCount(Monoid<T>.of(max).id, 0)
impl<T> Add for MaxCount<T>:
    fn add(a, b) = MaxCount.join(a, b)

/// \(O(1)\) per composition/eval. Affine map `x -> a * x + b` over any ring `T` (`Int`, `Mod<P>`).
pub struct Affine<T: Mul>(pub a: T, pub b: T):
    /// The identity map `x -> 1 * x + 0`.
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `Affine<Int>.id().eval(7) == 7`
    pub fn id() -> Affine<T> = Affine(Monoid<T>.of(*).id, Monoid<T>.of(+).id)

    /// `a * x + b`.
    ///
    /// cost: \(O(1)\)
    /// traps: never (overflow wraps in release, traps in debug)
    /// example: `Affine(2, 3).eval(5) == 13`
    pub fn eval(self, x: T) -> T = a * x + b

    /// `f` first, then `g`: the map `x -> g.eval(f.eval(x))`, which is
    /// `Affine(g.a * f.a, g.a * f.b + g.b)`.
    ///
    /// cost: \(O(1)\)
    /// traps: never (overflow wraps in release, traps in debug)
    /// example: `Affine.then(Affine(2, 0), Affine(1, 1)).eval(5) == 11`
    /// example: `Affine.then(Affine(1, 1), Affine(2, 0)).eval(5) == 12`
    pub fn then(f: Affine<T>, g: Affine<T>) -> Affine<T> = Affine(g.a * f.a, g.a * f.b + g.b)

    /// `Monoid(Affine.id(), Affine.then)`: a fold applies the leftmost map first, so a segment tree
    /// over maps answers "push `x` through positions `l..r` in order".
    ///
    /// cost: \(O(1)\)
    /// traps: never
    /// example: `[Affine(1, 1), Affine(2, 0)].fold(Affine.monoid()).eval(5) == 12`
    pub fn monoid() -> Monoid<Affine<T>> = Monoid(Affine<T>.id(), (f, g) => Affine.then(f, g))

@test
fn bare_operators_bring_their_identity():
    assert Monoid<Int>.of(+).id == 0 and Monoid<Int>.of(*).id == 1
    assert Monoid<Int>.of(^).id == 0 and Monoid<Int>.of(|).id == 0 and Monoid<Int>.of(gcd).id == 0
    assert Monoid<Int>.of(&).id == -1
    assert Monoid<Int>.of(max).id == Int.MIN and Monoid<Int>.of(min).id == Int.MAX
    assert Monoid<?Int>.of(max).id == None and Monoid<?Int>.of(min).id == None
    let xs = [-7, -1, 0, 2, 5, 9]
    assert laws_hold(Monoid<Int>.of(+), xs)
    assert laws_hold(Monoid<Int>.of(*), [-2, -1, 0, 1, 3])
    assert laws_hold(Monoid<Int>.of(^), xs)
    assert laws_hold(Monoid<Int>.of(&), xs)
    assert laws_hold(Monoid<Int>.of(|), xs)
    assert laws_hold(Monoid<Int>.of(max), xs)
    assert laws_hold(Monoid<Int>.of(min), xs)
    assert laws_hold(Monoid<Int>.of(gcd), [0, 4, 6, 9, 12])
    assert laws_hold(Monoid<?Int>.of(max), [None, -3, 0, 8])

@test
fn a_monoid_of_your_own_is_a_pair():
    let cat = Monoid<Str>("", (a, b) => a + b)
    assert laws_hold(cat, ["", "ab", "c"])
    assert cat.op(cat.id, "ab") == "ab" and cat.op("ab", "c") == "abc"

@test
fn affine_action_laws():
    // f = (mul, add) acting on a sum of `span` values as mul * x + add * span;
    // compose(new_f, old_f) applies old_f first, then new_f.
    let act = Action<Int, (Int, Int)>(
        (1, 0),
        (new_f, old_f) => (new_f.0 * old_f.0, new_f.0 * old_f.1 + new_f.1),
        (f, val, span) => f.0 * val + f.1 * span,
        over: +,
    )
    assert laws_hold(act, T: [-3, 0, 4], F: [(1, 0), (2, 3), (-1, 5), (0, 7)])
    assert act.over.id == 0
    // Non-commutative check: old = (*2 + 1), new = (*3 + 5) => 3*(2x + 1) + 5 = 6x + 8
    let combined = act.compose((3, 5), (2, 1))
    assert combined == (6, 8) and act.apply(combined, 4, 1) == 32
    assert act.apply(combined, 10, 3) == 6 * 10 + 8 * 3

@test
fn named_arguments_build_the_same_monoid_and_action():
    let cat = Monoid<Str>(op: (a, b) => a + b, id: "")
    assert cat.id == "" and cat.op("ab", "c") == "abc"
    let positional = Action<Int, (Int, Int)>(
        (1, 0),
        (new_f, old_f) => (new_f.0 * old_f.0, new_f.0 * old_f.1 + new_f.1),
        (f, val, span) => f.0 * val + f.1 * span,
        over: +,
    )
    let named = Action<Int, (Int, Int)>(
        apply: (f, val, span) => f.0 * val + f.1 * span,
        over: +,
        compose: (new_f, old_f) => (new_f.0 * old_f.0, new_f.0 * old_f.1 + new_f.1),
        id: (1, 0),
    )
    assert laws_hold(named, T: [-3, 0, 4], F: [(1, 0), (2, 3), (-1, 5)])
    for f in [(1, 0), (2, 3), (-1, 5)]:
        for g in [(0, 7), (3, -2)]:
            assert named.compose(f, g) == positional.compose(f, g)
            assert named.apply(f, 6, 2) == positional.apply(f, 6, 2)
    assert named.id == positional.id and named.over.id == 0

@test
fn desc_reverses_one_part_of_a_key():
    var xs = [(3, 1), (1, 2), (3, 0), (2, 2)]
    xs.sort(by: p => (p.0, desc(p.1)))
    assert xs == [(1, 2), (2, 2), (3, 1), (3, 0)]
    xs.sort(by: p => desc(p))
    assert xs == [(3, 1), (3, 0), (2, 2), (1, 2)]
    assert desc(2) < desc(1) and desc(1) == desc(1)

struct Word(s: Str)

impl Reversible for Word:
    fn rev(self) -> Word = Word(Str([s[s.len - 1 - i] for i in 0..s.len]))

@test
fn a_monoid_reverses_by_its_op():
    assert Monoid<Int>.of(+).reversed(5) == 5 and Monoid<Int>.of(max).reversed(3) == 3
    assert Monoid(0, (a, b) => a * 10 + b).commutative().monoid().reversed(7) == 7
    let digits = Monoid(Word(""), (a, b) => Word(a.s + b.s), x => x.rev())
    assert digits.reversed(Word("abc")) == Word("cba")

@test
fn max_sub_monoid_laws_and_kadane():
    let samples = [
        MaxSub.zero,
        MaxSub.of(-5),
        MaxSub.of(3),
        MaxSub.of(-2) + MaxSub.of(4),
        MaxSub.of(5) + MaxSub.of(-1) + MaxSub.of(2),
    ]
    assert laws_hold(MaxSub.monoid(), samples)
    assert MaxSub.zero.try_best == None
    let xs = [-3, 4, -1, 2, -5, 3, 2, -1]
    var got = MaxSub.zero
    for x in xs: got = MaxSub.join(got, MaxSub.of(x))
    var want = Int.MIN
    for l in 0..xs.len:
        var s = 0
        for r in l..xs.len:
            s += xs[r]
            want max= s
    assert got.best == want and got.try_best == want
    assert got.scale(3).best == want * 3
    let a = MaxSub.of(2) + MaxSub.of(-1) + MaxSub.of(5)
    let b = MaxSub.of(5) + MaxSub.of(-1) + MaxSub.of(2)
    assert a.rev() == b and MaxSub.monoid().reversed(a) == b

/// \(O(1)\). Three-way ordering result returned by `cmp`.
pub enum Ordering:
    Less
    Equal
    Greater

/// The three-way comparison of `self` and `other` under `Ord`: `cmp(a, b)` or `a.cmp(b)`.
///
/// cost: \(O(1)\) plus one or two comparisons of `T`
/// traps: never
/// example: `cmp(1, 2) == Ordering.Less`
/// example: `cmp("b", "a") == Ordering.Greater`
/// example: `cmp(3, 3) == Ordering.Equal`
/// example: `3.cmp(5) == Ordering.Less`
pub fn cmp<T: Ord>(self: T, other: T) -> Ordering =
    if self < other: Ordering.Less elif other < self: Ordering.Greater else: Ordering.Equal

/// The pair `(min(a, b), max(a, b))` in the operands' own type: two `Cut`s give two `Cut`s.
///
/// cost: \(O(1)\) plus one comparison of `T`
/// traps: never
/// example: `minmax(3, 1) == (1, 3)`
/// example: `minmax(1, 3) == (1, 3)`
/// example: `minmax("b", "a") == ("a", "b")`
/// ties: `struct K(k: Int, t: Int); impl Ord for K: fn lt(a, b) = a.k < b.k; let p = minmax(K(1, 0), K(1, 1)); (p.0.t, p.1.t) == (0, 1)`
pub fn minmax<T: Ord>(a: T, b: T) -> (T, T) = if b < a: (b, a) else: (a, b)

/// \(O(1)\). A semiring from two operation values: `Semiring(min, +)`, `Semiring(or, and)`.
/// `add` is a commutative monoid, `mul` a monoid that distributes over it; zero and one are
/// their identities. Matrix products (`Mat(over:)`), path algebras and `convolve(over:)` take
/// it. `Semiring(+, *)` is the default wherever one is accepted.
pub struct Semiring<A, M>(pub add: A, pub mul: M)

/// An associative operation, possibly without an identity (round 5). A `Monoid`, a `Group`, a
/// `Semigroup` and any `(T, T) -> T` fn implement it; a bare operator (`+ * ^ | & min max gcd
/// and or`) passed as an `Op` is its `Monoid`. `Fold` is what a fold over the op returns: `T`
/// when it has an identity, `?T`, `None` for an empty range, when it has none (decision 456).
/// Law (promised, sampled by stress): `op(op(a, b), c) == op(a, op(b, c))`.
pub trait Op<T>:
    type Fold = ?T
    fn op(self, a: T, b: T) -> T
    /// The op on folds, with the fold of nothing as its identity.
    fn folds(self) -> Monoid<Self.Fold> =
        Monoid(None, (a: ?T, b: ?T) => if let x = a: (if let y = b: Some(self.op(x, y)) else: a) else: b)
    /// The fold of the single value `x`.
    fn lift(self, x: T) -> Self.Fold = Some(x)

impl<T> Op<T> for Monoid<T>:
    type Fold = T
    fn op(self, a: T, b: T) -> T = self.op(a, b)
    fn folds(self) -> Monoid<T> = self
    fn lift(self, x: T) -> T = x

impl<T> Op<T> for Group<T>:
    type Fold = T
    fn op(self, a: T, b: T) -> T = self.op(a, b)
    fn folds(self) -> Monoid<T> = Monoid(self.id, self.op)
    fn lift(self, x: T) -> T = x

impl<T> Op<T> for Semigroup<T>:
    type Fold = ?T
    fn op(self, a: T, b: T) -> T = self.op(a, b)
    fn folds(self) -> Monoid<?T> = self.opt()
    fn lift(self, x: T) -> ?T = Some(x)

impl<T> Op<T> for (T, T) -> T:
    type Fold = ?T
    fn op(self, a: T, b: T) -> T = self(a, b)
    fn folds(self) -> Monoid<?T> = self.opt()
    fn lift(self, x: T) -> ?T = Some(x)

/// The identity of an operation. Law: `op(id, a) == a == op(a, id)`.
pub trait HasId<T>:
    @get fn id(self) -> T

/// The inverse of an operation. Law: `op(a, inv(a)) == id`.
pub trait HasInv<T>:
    fn inv(self, a: T) -> T

/// Law: `op(a, b) == op(b, a)`. Lets folds reorder and tags skip push-down.
pub trait Commutative<T>: Op<T>:
    law comm(a: T, b: T) where T: Eq = self.op(a, b) == self.op(b, a)

/// Law: `op(a, a) == a`. Lets `SparseTable` overlap halves.
pub trait Idempotent<T>: Op<T>:
    law idem(a: T) where T: Eq = self.op(a, a) == a

/// Law: `op(a, b)` is `a` or `b`. Lets sliding folds use a monotone deque.
pub trait Selective<T>: Idempotent<T>:
    law select(a: T, b: T) where T: Eq = self.op(a, b) == a or self.op(a, b) == b

/// One operation per component of a tuple (positional) or record (named): `op((a0, a1),
/// (b0, b1)) == (m0(a0, b0), m1(a1, b1))`. The identity is the tuple of identities, and the
/// result keeps only the laws every component has.
///
/// cost: \(O(1)\) to build; each call costs the sum of its components
/// traps: never
/// example: `[(1, 5), (2, 3)].fold(each(+, max)) == (3, 5)`
/// example: `[(n: 1, hi: 5), (n: 2, hi: 3)].fold(each(n: +, hi: max)).hi == 5`
pub fn each<Ms>(..ms: Ms) -> Each<Ms> = Each(ms)

/// The ops of [`each`], one per component; a structure takes it through [`Monoid.of`].
pub struct Each<Ms>(ms: Ms):
    /// The monoid on `T` that applies the i-th op to the i-th components, with the tuple of
    /// their identities as its identity.
    ///
    /// cost: \(O(\mathit{components})\)
    /// traps: never (an op with no known identity is a compile-time error)
    /// example: `each(+, max).monoid<(Int, Int)>().op((1, 5), (2, 3)) == (3, 5)`
    pub fn monoid<T>(self) -> Monoid<T>:
        let ms = self.ms
        Monoid(T(identity_of(m) for m in fields(ms)), (a, b) => T(op_of(m, x, y) for (m, x, y) in fields(ms, a, b)))

impl<T, S> FromOp<T> for Each<S>:
    fn monoid(self) -> Monoid<T> = self.monoid<T>()

fn identity_of<M: FromOp<F>, F>(m: M) -> F = Monoid<F>.of(m).id

fn op_of<M: FromOp<F>, F>(m: M, x: F, y: F) -> F = Monoid<F>.of(m).op(x, y)

/// Marker type for the stock `add` action.
pub struct AddAction

/// Marker type for the stock `assign` action.
pub struct AssignAction

/// Marker type for the stock `affine` action.
pub struct AffineAction

/// Marker type for the stock `flip` action.
pub struct FlipAction

/// Marker type for the stock `assign_add` action.
pub struct AssignAddAction

/// Stock action: add a tag to every element; the tag is a `T`. On `+` a span of `n` changes by `f * n`; on
/// `min`/`max` by `f`. Tags commute, so `SegTree.apply` with it never pushes down. \(O(1)\).
pub const add: AddAction = AddAction()

/// Stock action: assign a tag to every element; the tag is a `?T` (`None` is a no-op). \(O(1)\) on `+`,
/// `min`, `max`; \(O(\log n)\) folding `f` `n` times for other ops.
pub const assign: AssignAction = AssignAction()

/// Stock action: `x -> a * x + b` on `+` folds (span `n`: `a * s + b * n`). Not commutative.
/// The tag is the tuple `(a, b)`, not `Affine`: `s.apply(r, (2, 1))` doubles and adds 1.
pub const affine: AffineAction = AffineAction()

/// Stock action: swap the two halves of a `Flip<T>` value.
pub const flip: FlipAction = FlipAction()

/// Stock action: optionally overwrite every element, then add to it. The tag is `(set: ?T, add: T)`:
/// `(set: Some(5), add: 2)` makes each element 7, `(set: None, add: 2)` adds 2. On `+` a span of
/// `n` becomes `s * n + d * n` (or `v + d * n` with no `set`); on `min`/`max`, `s + d` (or `v + d`).
/// A later tag with a `set` replaces the earlier one; one without adds to it. \(O(1)\).
pub const assign_add: AssignAddAction = AssignAddAction()

/// \(O(1)\). The op lifted to `?T` with `None` as its identity, so any associative op (a closure,
/// a `Semigroup`) becomes a `Monoid` whose folds say "nothing here" as `None` (ops-ranges.md section 2).
///
/// cost: \(O(1)\) to make; each call is one `op` call or none
/// traps: never
/// example: `let f = (a: Int, b: Int) => a * 10 + b; [Some(1), None, Some(2)].fold(f.opt()) == Some(12)`
pub fn opt<T>(self: (T, T) -> T) -> Monoid<?T> =
    Monoid(None, (a: ?T, b: ?T) => if let x = a: (if let y = b: Some(self(x, y)) else: a) else: b)

@test
fn pow_matches_repeated_op():
    for k in 0..20:
        var mul = 1
        var total = 0
        var aff = Affine<Int>.id()
        for _ in 0..k:
            mul *= 3
            total += 3
            aff = Affine.then(aff, Affine(2, 1))
        assert pow(3, k, op: *) == mul and pow(3, k, op: +) == total
        assert pow(Affine(2, 1), k, op: Affine.monoid()).eval(5) == aff.eval(5)

impl Show for Each<Ms>:
    /// How many ops it combines, as `each of n ops`.
    ///
    /// example: `"{each(+, max)}" == "each of 2 ops"`
    fn show(e):
        var k = 0
        for _ in fields(e.ms): k += 1
        "each of {k} ops"

fn affine_onto(mut old: (Int, Int), new: (Int, Int)):
    old = (new.0 * old.0, new.0 * old.1 + new.1)

@test
fn compose_onto_matches_compose_with_and_without_compose_in():
    let fs = [(1, 0), (2, 3), (-1, 5), (0, 7)]
    let comp = (nf: (Int, Int), of: (Int, Int)) => (nf.0 * of.0, nf.0 * of.1 + nf.1)
    let app = (f: (Int, Int), v: Int, span: Int) => f.0 * v + f.1 * span
    let good = Action<Int, (Int, Int)>(id: (1, 0), compose: comp, apply: app, compose_in: (mut o, nf) => affine_onto(mut o, nf))
    let plain = Action<Int, (Int, Int)>(id: (1, 0), compose: comp, apply: app)
    assert laws_hold(good, T: [-3, 0, 4], F: fs)
    for f in fs:
        for g in fs:
            var h = g
            good.compose_onto(mut h, f)
            assert h == comp(f, g)
            var k = g
            plain.compose_onto(mut k, f)
            assert k == comp(f, g)


struct Larger

impl Op<Int> for Larger:
    fn op(self, a: Int, b: Int) -> Int = max(a, b)

impl Commutative<Int> for Larger
impl Idempotent<Int> for Larger
impl Selective<Int> for Larger

fn capped(a: Int, b: Int) -> Int:
    assert a * b <= 100, "an op pow did not need"
    a * b

@test
fn laws_hold_checks_every_kind_of_operation():
    let xs = [-7, -1, 0, 2, 5]
    assert laws_hold(Monoid<Int>.of(+).commutative(), xs)
    assert laws_hold(Semigroup<Int>((a, b) => max(a, b)), xs)
    assert laws_hold(Group<Int>.of(+), xs) and laws_hold(Group<Int>.of(^), xs)
    assert laws_hold(Larger(), xs)
    assert laws_hold(Action<Int, Int>.of(add, +), T: xs, F: xs)

@test
fn stock_actions_compose_and_apply():
    let f3 = [(1, 0), (2, 3), (-1, 5)]
    assert laws_hold(Action<Int, (Int, Int)>.of(affine, +), T: [-3, 0, 4], F: f3)
    let add_sum = Action<Int, Int>.of(add, +)
    assert add_sum.compose(2, 5) == 7 and add_sum.apply(2, 10, 3) == 16 and add_sum.commutes
    let add_max = Action<Int, Int>.of(add, max)
    assert add_max.compose(2, 5) == 7 and add_max.apply(2, 10, 3) == 12 and add_max.commutes
    assert Action<Int, ?Int>.of(assign, +).apply(Some(4), 10, 3) == 12
    let aff = Action<Int, (Int, Int)>.of(affine, +)
    assert aff.compose((3, 5), (2, 1)) == (6, 8) and aff.apply((3, 5), 10, 3) == 45
    let keep: ?Int = None
    let aa = Action<Int, (set: ?Int, add: Int)>.of(assign_add, +)
    assert aa.apply((set: Some(4), add: 1), 10, 3) == 15 and aa.apply((set: keep, add: 2), 10, 3) == 16
    assert aa.compose((set: keep, add: 2), (set: Some(4), add: 1)) == (set: Some(4), add: 3)
    let am = Action<Int, (set: ?Int, add: Int)>.of(assign_add, max)
    assert am.apply((set: Some(4), add: 1), 10, 3) == 5 and am.apply((set: keep, add: 2), 10, 3) == 12
    assert am.compose((set: keep, add: 2), (set: Some(4), add: 1)) == (set: Some(4), add: 3)

@test
fn pow_squares_only_while_bits_remain():
    assert pow(10, 2, op: Monoid<Int>(1, capped)) == 100

@test
fn max_sub_scales_every_part():
    let s = (MaxSub.of(3) + MaxSub.of(-5) + MaxSub.of(2)).scale(3)
    assert s.best == 9 and s.total == 0 and s.pre == 9 and s.suf == 6

@test
fn assign_actions_replace_and_never_commute():
    let xs = [-3, 0, 4]
    let keep: ?Int = None
    let fs = [keep, Some(2), Some(5)]
    let gs = [(set: keep, add: 0), (set: keep, add: 3), (set: Some(2), add: 1), (set: Some(5), add: 0)]
    let sum_set = Action<Int, ?Int>.of(assign, +)
    let max_set = Action<Int, ?Int>.of(assign, max)
    let sum_set_add = Action<Int, (set: ?Int, add: Int)>.of(assign_add, +)
    let max_set_add = Action<Int, (set: ?Int, add: Int)>.of(assign_add, max)
    assert laws_hold(sum_set, T: xs, F: fs) and laws_hold(max_set, T: xs, F: fs)
    assert laws_hold(sum_set_add, T: xs, F: gs) and laws_hold(max_set_add, T: xs, F: gs)
    assert not sum_set.commutes and not max_set.commutes
    assert not sum_set_add.commutes and not max_set_add.commutes
    let sets = max_set.replaces!
    assert sets(Some(2)) and not sets(keep)
    let set_adds = max_set_add.replaces!
    assert set_adds((set: Some(2), add: 0)) and not set_adds((set: keep, add: 3))
    assert not Action<Int, Int>(apply: (d, x, span) => x + d * span).commutes
