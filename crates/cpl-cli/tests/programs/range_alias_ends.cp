use std.segtree.{SegTree}

// A generic `T` given the range alias `M` has `M`'s ends as `T.MIN` and `T.MAX`, not `Int`'s.
type M = -10..=10
fn ends<T>() -> (T, T, T, T) = (T.MIN, T.MAX, T.zero, T.one)
let (lo, hi, z, o) = ends<M>()
let best = Monoid<M>.of(max)
print(lo, hi, z, o, best.id)
var t = SegTree<0..4, M>(max)
print(t.fold(0..2))
