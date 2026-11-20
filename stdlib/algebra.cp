//! Operation values: the behaviour a type can have many versions of.
//!
//! A structure never asks "what is the sum of two `T`s"; it is handed a
//! value that says so. `SegTree<V, Int>(Max<Int>)` and
//! `SegTree<V, Int>(Sum<Int>)` are the same tree with two monoids. Each
//! struct states its laws, and `laws_hold(m, samples)` checks them.

/// An associative operation. Enough for a sparse table when it is also idempotent.
pub struct Semigroup<T>(op: (T, T) -> T):
    law assoc(a: T, b: T, c: T) = op(op(a, b), c) == op(a, op(b, c))

/// An associative operation with an identity.
pub struct Monoid<T>(id: T, op: (T, T) -> T):
    law assoc(a: T, b: T, c: T) = op(op(a, b), c) == op(a, op(b, c))
    law ident(a: T) = op(id, a) == a and op(a, id) == a

/// A monoid where every value can be undone. What a Fenwick tree needs for range queries.
pub struct Group<T>(id: T, op: (T, T) -> T, inv: T -> T):
    law assoc(a: T, b: T, c: T) = op(op(a, b), c) == op(a, op(b, c))
    law ident(a: T) = op(id, a) == a and op(a, id) == a
    law inverse(a: T) = op(a, inv(a)) == id and op(inv(a), a) == id

/// Updates `F` acting on values `T`: `apply(f, x)` is `x` after `f`,
/// `compose(f, g)` is "g first, then f".
pub struct Action<T, F>(id: F, compose: (F, F) -> F, apply: (F, T) -> T):
    law ident(a: T) = apply(id, a) == a
    law composes(f: F, g: F, a: T) = apply(compose(f, g), a) == apply(f, apply(g, a))

/// A strict ordering passed as a value: `sort(mut xs, by: Order.by(e => e.w))`.
pub struct Order<T>(lt: (T, T) -> Bool):
    law irreflexive(a: T) = not lt(a, a)
    law transitive(a: T, b: T, c: T) = not (lt(a, b) and lt(b, c)) or lt(a, c)

    /// Order by a key.
    pub fn by(key: T -> Int) -> Order<T> = Order((a, b) => key(a) < key(b))

    /// The same order, reversed.
    pub @get fn rev(self) -> Order<T> = Order((a, b) => self.lt(b, a))

pub const Sum<T> = Group<T>(id: T.zero, op: (a, b) => a + b, inv: a => -a) where T: Add
pub const Prod<T> = Monoid<T>(id: T.one, op: (a, b) => a * b) where T: Mul
pub const Max<T> = Monoid<T>(id: T.MIN, op: (a, b) => max(a, b)) where T: Ord + Bounded
pub const Min<T> = Monoid<T>(id: T.MAX, op: (a, b) => min(a, b)) where T: Ord + Bounded
pub const Xor<T> = Group<T>(id: T.zero, op: (a, b) => a ^ b, inv: a => a) where T: Bits
pub const Less<T> = Order<T>(lt: (a, b) => a < b) where T: Ord
pub const Greater<T> = Order<T>(lt: (a, b) => b < a) where T: Ord

/// `m` with its arguments swapped: folds right to left.
pub fn flip<T>(m: Monoid<T>) -> Monoid<T> = Monoid(m.id, (a, b) => m.op(b, a))

/// Two monoids side by side, e.g. `both(Sum<Int>, Max<Int>)` over pairs.
pub fn both<A, B>(x: Monoid<A>, y: Monoid<B>) -> Monoid<(A, B)>:
    Monoid((x.id, y.id), (a, b) => (x.op(a.0, b.0), y.op(a.1, b.1)))

/// A semigroup made into a monoid by adding `none` as the identity.
pub fn opt<T>(s: Semigroup<T>) -> Monoid<?T>:
    Monoid(none, (a, b) => if a == none: b else: if b == none: a else: s.op(a!, b!))

/// The monoid part of a group; structures that take a `Monoid` accept a `Group` directly.
pub fn fold<T>(m: Monoid<T>, xs: List<T>) -> T:
    var acc = m.id
    for x in xs:
        acc = m.op(acc, x)
    acc

@test
fn laws_of_the_stock_values():
    let xs = [-7, -1, 0, 2, 5, 9]
    assert laws_hold(Sum<Int>, xs)
    assert laws_hold(Max<Int>, xs)
    assert laws_hold(Min<Int>, xs)
    assert laws_hold(Xor<Int>, xs)
    assert laws_hold(Prod<Int>, xs)
    assert laws_hold(Less<Int>, xs)
    assert laws_hold(Less<Int>.rev, xs)
    assert laws_hold(flip(Sum<Int>), xs)
    assert laws_hold(both(Sum<Int>, Max<Int>), [(1, 2), (0, -3), (-4, 4)])
    assert laws_hold(opt(Semigroup<Int>((a, b) => min(a, b))), [none, 1, 5])

@test
fn fold_respects_order():
    let cat = Monoid<Str>("", (a, b) => a + b)
    assert fold(cat, ["ab", "c", "de"]) == "abcde"
    assert fold(flip(cat), ["ab", "c", "de"]) == "decab"
    assert fold(Sum<Int>, [1, 2, 3]) == 6

@test
fn affine_action_laws():
    // f = (mul, add) acting on x as mul * x + add
    let act = Action<Int, (Int, Int)>((1, 0), (f, g) => (f.0 * g.0, f.0 * g.1 + f.1), (f, x) => f.0 * x + f.1)
    assert laws_hold(act, T: [-3, 0, 4], F: [(1, 0), (2, 3), (-1, 5), (0, 7)])

@test
fn order_by_key():
    var xs = [(3, 1), (1, 2), (2, 0)]
    sort(mut xs, by: Order<(Int, Int)>.by(p => p.1))
    assert xs == [(2, 0), (3, 1), (1, 2)]
    sort(mut xs, by: Greater<(Int, Int)>)
    assert xs == [(3, 1), (2, 0), (1, 2)]
