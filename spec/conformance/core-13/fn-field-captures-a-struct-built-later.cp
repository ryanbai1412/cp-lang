// expect: ok, output "4\n5\n5"
// rule: 13.3.2
// A fn field whose closures all copy captures of the same types holds those captures beside a
// plain function: `flip_action`'s `over` op copies the `Monoid<Int>` it folds with, and that
// struct is complete before the `Monoid<Flip<Int>>` holding the op.
use std.lazy.{flip_action}
domain I = 0..4
fn join(a: Flip<Int>, b: Flip<Int>) -> Flip<Int> = Flip(a.now + b.now, a.flipped + b.flipped)
var t = SegTree<I, Flip<Int>, Bool>(
    Monoid(Flip(0, 0), join),
    [I] i => Flip.of(Int(i) % 2 == 0, Int(i) + 1, 0),
    act: flip_action<Int>(+),
)
print(t.all.now)
t.apply(I(0)..=I(1), true)
print(t.all.now)
t.apply(I(0)..=I(3), true)
print(t.all.now)
