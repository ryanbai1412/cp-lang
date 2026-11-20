use std.algebra.{Monoid}
use std.segtree.{SegTree}

let n: 1..=100 = read()
let q: 1..=100 = read()
domain V = 1..=n
let a: [V]Int = read()

let sum_m = Monoid<Int>(0, (x, y) => x + y)
var st = SegTree<V, Int>(sum_m, a)
for _ in 0..q:
    let t: Int = read()
    if t == 1:
        let pos: V = read()
        let val: Int = read()
        st.set(pos, val)
    else:
        let l: V = read()
        let r: V = read()
        print(st.fold(l..=r))
