// CSES 1735 Range Updates and Sums: range add, range assign, range sum.
use std.algebra.{Monoid, Action}
use std.lazy.LazySegTree
let (n, q): (1..=2e5, 1..=2e5) = read()
domain I = 1..=n
let t: [I]1..=1e6 = read()
// a tag (mul, add) maps x to mul * x + add: adding x is (1, x), assigning x is (0, x)
let affine = Action<Int, (Int, Int)>(
    id: (1, 0),
    compose: (new_f, old_f) => (new_f.0 * old_f.0, new_f.0 * old_f.1 + new_f.1),
    apply: (f, val, span) => f.0 * val + f.1 * span,
    over: Monoid<Int>(id: 0, op: (x, y) => x + y),
)
var st = LazySegTree<I, Int, (Int, Int)>(affine, [I] i => t[i])
repeat q:
    let kind: 1..=3 = read()
    let (a, b): (I, I) = read()
    if kind == 3:
        print(st.fold(a..=b))
        continue
    let x: 1..=1e6 = read()
    st.apply(a..=b, if kind == 1: (1, x) else: (0, x))
