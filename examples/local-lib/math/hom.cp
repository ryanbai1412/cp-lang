//! Affine maps `x -> a * x + b` as a monoid under composition.
//!
//! The classic non-commutative monoid: a `SegTree` of them answers "apply the maps on
//! this range, in order", and a path of them on a tree needs the order kept, which is
//! what `Reversible` is for.

/// The map `x -> a * x + b` over any type with `+` and `*` (`Int`, `Mod<P>`).
use std.algebra.{Monoid, Reversible}
pub struct Hom<T>(a: T, b: T):
    /// The value of the map at `x`.
    pub fn at(self, x: T) -> T = a * x + b

    /// The map "first `self`, then `next`".
    pub fn then(self, next: Hom<T>) -> Hom<T> = Hom(next.a * a, next.a * b + next.b)

    /// Composition in application order, with `one` and `zero` of `T` for the identity map:
    /// `fold` of a range is the map that applies its elements left to right.
    pub fn compose(one: T, zero: T) -> Monoid<Hom<T>> = Monoid(Hom(one, zero), (f, g) => f.then(g))

/// A pair of an affine map and the same stretch composed right to left: the value to
/// store when a path may be walked in either direction.
pub struct BothWays<T>(fwd: Hom<T>, back: Hom<T>):
    pub fn of(f: Hom<T>) -> BothWays<T> = BothWays(f, f)

    pub fn compose(one: T, zero: T) -> Monoid<BothWays<T>>:
        let id = Hom(one, zero)
        Monoid(BothWays(id, id), (x, y) => BothWays(x.fwd.then(y.fwd), y.back.then(x.back)))

impl Reversible for BothWays<T>:
    fn rev(self) -> BothWays<T> = BothWays(back, fwd)

@test
fn composition_applies_left_to_right():
    let m = Hom<Int>.compose(1, 0)
    for _ in 0..50:
        let fs = [Hom(rand(-3, 3), rand(-5, 5)) for _ in 0..rand(0, 6)]
        var all = m.id
        var x = rand(-4, 4)
        let start = x
        for f in fs:
            all = m.op(all, f)
            x = f.at(x)
        assert all.at(start) == x
    assert laws_hold(m, [Hom(2, 1), Hom(0, 3), Hom(-1, 0), Hom(1, 0)])

@test
fn a_segment_tree_of_maps():
    domain I = 0..5
    let fs = [I] i => Hom(i.to_idx + 1, i.to_idx)
    var st = SegTree<I, Hom<Int>>(Hom<Int>.compose(1, 0), fs)
    var x = 1
    for i in I(1)..=I(3): x = fs[i].at(x)
    assert st.fold(I(1)..=I(3)).at(1) == x
    st.set(I(2), Hom(0, 7))
    assert st.fold(I(1)..=I(3)).at(1) == 4 * 7 + 3

@test
fn reversing_a_stretch_swaps_the_directions():
    let m = BothWays<Int>.compose(1, 0)
    let fs = [Hom(2, 1), Hom(3, -1), Hom(-1, 4)]
    var acc = m.id
    for f in fs: acc = m.op(acc, BothWays<Int>.of(f))
    assert acc.fwd.at(5) == fs[2].at(fs[1].at(fs[0].at(5)))
    assert acc.rev().fwd.at(5) == fs[0].at(fs[1].at(fs[2].at(5)))
