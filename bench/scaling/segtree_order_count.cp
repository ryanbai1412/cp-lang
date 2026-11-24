use std.segtree.{DynamicOrderTree}
let n: Int = read()
domain V = 0..n
let reps = 1 << 17
var s = 0
let a: [V]Int = [V] v => (v.ord * 7919) % n + 1

var dt = DynamicOrderTree(a)
for i in 0..reps / 8:
    let l = i * 7919 % n
    let rg = V(l / 2)..V(l / 2 + n / 2)
    s += dt.count_lt(rg, l) // timed: segtree.DynamicOrderTree.count_lt
    s += dt.count_in(rg, l / 2, l) // timed: segtree.DynamicOrderTree.count_in
    s += dt.kth(rg, l % (n / 2)) ?? 0 // timed: segtree.DynamicOrderTree.kth
print(s % 1000000007)
