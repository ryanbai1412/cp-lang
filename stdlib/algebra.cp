//! Operations as values: what a range structure folds with.
//!
//! A structure never asks "what is the sum of two `T`s"; it is handed the
//! operation. Most of the time that is a bare operator: `SegTree<V, Int>(max, a)`
//! and `SegTree<V, Int>(+, a)` are the same tree with two operations. An operation
//! of your own is the pair `Monoid(id, op)`; a lazy update is an `Action`.
//! `desc(x)` reverses an order inside a sort key or a heap entry, and a
//! non-commutative value says how it reads backwards with `Reversible`.
use std.math.{gcd}

/// An associative operation with an identity: the pair `(id, op)`.
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

/// A lazy update `F` on values `T` folded with `over`: `apply(f, x, len)` is the fold `x` of
/// `len` positions after every one of them took `f`, `compose(f, g)` is "g first, then f",
/// and `id` changes nothing. `Action(id, compose, apply, over: op)`.
pub struct Action<T, F>(id: F, compose: (F, F) -> F, apply: (F, T, Int) -> T, over: Monoid<T>):
    law ident(a: T) = apply(id, a, 1) == a
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
    // f = (mul, add) acting on a sum of `len` values as mul * x + add * len
    let act = Action<Int, (Int, Int)>((1, 0), (f, g) => (f.0 * g.0, f.0 * g.1 + f.1), (f, x, len) => f.0 * x + f.1 * len, over: +)
    assert laws_hold(act, T: [-3, 0, 4], F: [(1, 0), (2, 3), (-1, 5), (0, 7)])
    assert act.over.id == 0

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
