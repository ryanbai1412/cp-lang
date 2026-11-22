// algebra.md documents `affine` as "x -> a * x + b" next to `struct Affine<T>(a: T, b: T)`,
// but `SegTree(+, a, act: affine)` takes a plain `(Int, Int)` tag.
// expected (from the docs): compiles. actual: E0302 "`st.apply` takes an (Int, Int) for `f`".
use std.algebra.{Affine}
domain V = 1..=3
var st = SegTree(+, [V] v => 1, act: affine)
st.apply(V(1)..=V(2), Affine(2, 1))
print(st.fold(V(1)..=V(3)))
// output: 7
