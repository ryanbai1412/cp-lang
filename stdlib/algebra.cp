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
    pub fn of<M>(m: M) -> Monoid<T>:
        if same(m, +): return Monoid(T.zero, (a, b) => a + b)
        if same(m, *): return Monoid(T.one, (a, b) => a * b)
        if same(m, ^): return Monoid(T.zero, (a, b) => a ^ b)
        if same(m, |): return Monoid(T.zero, (a, b) => a | b)
        if same(m, &): return Monoid(T.zero - T.one, (a, b) => a & b)
        if same(m, max): return Monoid(T.MIN, (a, b) => max(a, b))
        if same(m, min): return Monoid(T.MAX, (a, b) => min(a, b))
        if same(m, gcd): return Monoid(T.zero, (a, b) => gcd(a, b))
        m

fn same<A, B>(a: A, b: B) -> Bool = a == b

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

    pub fn new<M>(id: F, compose: (F, F) -> F, apply: (F, T, Int) -> T, over: M) -> Action<T, F>:
        Action(id, compose, apply, Monoid<T>.of(over))

/// `x` with its order reversed. Build it with `desc(x)`.
pub struct Desc<T>(x: T)

impl Ord for Desc<T>:
    fn lt(a, b) = b.x < a.x

impl Show for Desc<T>:
    fn show(a) = "desc({a.x})"

/// `x` ordered the other way round, for one part of a sort key or a heap entry:
/// `sort(mut es, by: e => (e.w, desc(e.name)))`, `MinHeap<(Int, Desc<V>)>()`.
pub fn desc<T>(x: T) -> Desc<T> = Desc(x)

/// A folded value that can be read from the other end. A fold along a tree path walks some
/// stretches upwards; `Rooted.fold_path` reverses those with `rev`. A type that does not
/// implement it is its own reverse, which is right for every commutative operation.
pub trait Reversible:
    fn rev(self) -> Self = self

/// `xs` folded left to right with `m`.
pub fn fold<T, M>(m: M, xs: List<T>) -> T:
    let k = Monoid<T>.of(m)
    var acc = k.id
    for x in xs:
        acc = k.op(acc, x)
    acc

/// Maximum non-empty subarray summary (Kadane monoid).
/// Concatenation (`+` or `MaxSub.join`) is associative with identity `MaxSub.empty` in O(1);
/// `s.scale(k)` scales every element by non-negative `k` in O(1).
/// Implements `Reversible` (swapping `pre` and `suf`) for tree-path folds.
pub struct MaxSub(has: Bool, best: Int, total: Int, pre: Int, suf: Int):
    /// Identity element representing an empty subarray. O(1).
    pub fn empty() -> MaxSub = MaxSub(false, 0, 0, 0, 0)

    /// Single-element summary for `v`. O(1).
    pub fn of(v: Int) -> MaxSub = MaxSub(true, v, v, v, v)

    /// Concatenate two adjacent summaries left to right. O(1).
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

    /// Scale every underlying value by `k >= 0`. O(1).
    pub fn scale(self, k: Int) -> MaxSub:
        assert k >= 0
        if not has: self
        else: MaxSub(true, best * k, total * k, pre * k, suf * k)

    /// Best non-empty subarray sum, or `none` when empty. O(1).
    pub @get fn try_best(self) -> ?Int = if has: best else: none

    /// The `Monoid<MaxSub>` value for `SegTree` and `fold`. O(1).
    pub fn monoid() -> Monoid<MaxSub> = Monoid(MaxSub.empty(), (a, b) => MaxSub.join(a, b))

impl Add for MaxSub:
    fn add(a, b) = MaxSub.join(a, b)

impl Reversible for MaxSub:
    fn rev(self) -> MaxSub = MaxSub(has, best, total, suf, pre)

@test
fn bare_operators_bring_their_identity():
    assert Monoid<Int>.of(+).id == 0 and Monoid<Int>.of(*).id == 1
    assert Monoid<Int>.of(^).id == 0 and Monoid<Int>.of(|).id == 0 and Monoid<Int>.of(gcd).id == 0
    assert Monoid<Int>.of(&).id == -1
    assert Monoid<Int>.of(max).id == Int.MIN and Monoid<Int>.of(min).id == Int.MAX
    assert Monoid<?Int>.of(max).id == none and Monoid<?Int>.of(min).id == none
    let xs = [-7, -1, 0, 2, 5, 9]
    assert laws_hold(Monoid<Int>.of(+), xs)
    assert laws_hold(Monoid<Int>.of(*), [-2, -1, 0, 1, 3])
    assert laws_hold(Monoid<Int>.of(^), xs)
    assert laws_hold(Monoid<Int>.of(&), xs)
    assert laws_hold(Monoid<Int>.of(|), xs)
    assert laws_hold(Monoid<Int>.of(max), xs)
    assert laws_hold(Monoid<Int>.of(min), xs)
    assert laws_hold(Monoid<Int>.of(gcd), [0, 4, 6, 9, 12])
    assert laws_hold(Monoid<?Int>.of(max), [none, -3, 0, 8])

@test
fn a_monoid_of_your_own_is_a_pair():
    let cat = Monoid<Str>("", (a, b) => a + b)
    assert laws_hold(cat, ["", "ab", "c"])
    assert fold::<Str>(cat, ["ab", "c", "de"]) == "abcde"
    assert fold::<Int>(+, [1, 2, 3]) == 6
    assert fold::<Int>(max, [1, 5, 3]) == 5

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
    sort(mut xs, by: p => (p.0, desc(p.1)))
    assert xs == [(1, 2), (2, 2), (3, 1), (3, 0)]
    sort(mut xs, by: p => desc(p))
    assert xs == [(3, 1), (3, 0), (2, 2), (1, 2)]
    assert desc(2) < desc(1) and desc(1) == desc(1)

struct Word(s: Str)

impl Reversible for Word:
    fn rev(self) -> Word = Word(Str([s[s.len - 1 - i] for i in 0..s.len]))

@test
fn plain_values_are_their_own_reverse():
    assert 5.rev() == 5 and (3, 4).rev() == (3, 4)
    assert Word("abc").rev() == Word("cba")

@test
fn max_sub_monoid_laws_and_kadane():
    let samples = [
        MaxSub.empty(),
        MaxSub.of(-5),
        MaxSub.of(3),
        MaxSub.of(-2) + MaxSub.of(4),
        MaxSub.of(5) + MaxSub.of(-1) + MaxSub.of(2),
    ]
    assert laws_hold(MaxSub.monoid(), samples)
    assert MaxSub.empty().try_best == none
    let xs = [-3, 4, -1, 2, -5, 3, 2, -1]
    var got = MaxSub.empty()
    for x in xs: got = got + MaxSub.of(x)
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
    assert a.rev() == b
