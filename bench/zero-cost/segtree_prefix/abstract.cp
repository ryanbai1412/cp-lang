use std.segtree.{SegTree}
use std.prefix.{Prefix}

fn kernel(n: Int, q: Int, a: List<Int>, qs: List<(Int, Int, Int)>) -> Int:
    domain V = 0..n
    var init: [V]Int = 0
    for i in V: init[i] = a[i.to_idx]
    let pf = Prefix.new(init)
    var st = SegTree<V, Int>(+, init)
    var acc = 0
    for (t, x, y) in qs:
        if t == 1:
            st.set(V.nth(x), y)
        else:
            let r = V.nth(x)..=V.nth(y)
            acc = (acc + st.fold(r) + pf.fold(r)) ^ (acc >> 4)
    acc

let n: Int = read()
let q: Int = read()
let a: [0..n]Int = read()
var qs: List<(Int, Int, Int)> = []
for _ in 0..q:
    let (t, x, y): (Int, Int, Int) = read()
    qs.push((t, x, y))
print(kernel(n, q, a, qs))
