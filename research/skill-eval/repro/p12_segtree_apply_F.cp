// Repro 2: SegTree.apply fails E0204 (`F` is not a type `cpl run` knows)
domain I = 1..=3
let a = [I] i => Int(i)
let act = Action<Int, (mul: Int, add: Int)>(
    id: (mul: 1, add: 0),
    compose: (f, g) => (mul: f.mul * g.mul, add: f.mul * g.add + f.add),
    apply: (f, s, len) => f.mul * s + f.add * len,
    over: Monoid<Int>.of(+),
)
var st = SegTree<I, Int>(Monoid<Int>.of(+), a, act: act)
st.apply(I(1)..=I(3), (mul: 1, add: 5))
print(st.fold(I(1)..=I(3)))
