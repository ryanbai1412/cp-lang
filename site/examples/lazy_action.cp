let n: 1..=100 = read()
let q: 1..=100 = read()
domain V = 1..=n
let a: [V]Int = read()

let sum_m = Monoid<Int>(0, (x, y) => x + y)
let add_act = Action<Int, Int>(
    0,
    (new_f, old_f) => new_f + old_f,
    (f, val, span) => val + f * span,
    sum_m,
)
var lz = LazySegTree<V, Int, Int>(add_act, a)
for _ in 0..q:
    let t: Int = read()
    let l: V = read()
    let r: V = read()
    if t == 1:
        let d: Int = read()
        lz.apply(l..=r, d)
    else:
        print(lz.fold(l..=r))
