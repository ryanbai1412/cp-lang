// expect: ok, output "-10 10 0 1 -10"
// rule: 02.2.1, R08.2.2
// A generic `T: Bounded + Zero + One` given the range alias `M` has `M`'s ends as `T.MIN` and `T.MAX`, not `Int`'s; a bound-free `T` has no members (R08.2.2).
type M = -10..=10
fn ends<T: Bounded + Zero + One>() -> (T, T, T, T) = (T.MIN, T.MAX, T.zero, T.one)
let (lo, hi, z, o) = ends<M>()
let best = Monoid<M>.of(max)
print(lo, hi, z, o, best.id)
