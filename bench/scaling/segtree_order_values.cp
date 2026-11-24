use std.segtree.{DynamicOrderTree}
let n: Int = read()
domain V = 0..n
let reps = 1 << 17
var s = 0
let a: [V]Int = [V] v => (v.ord * 7919) % n + 1
let vs: List<Int> = [x for x in 0..=n]

var dt = DynamicOrderTree(a, values: vs)
for i in 0..reps / 8:
    let l = i * 7919 % n
    dt.set(V(i * 13 % n), l)  // timed: segtree.DynamicOrderTree.set [let: D = n]
    let rg = V(l / 2)..V(l / 2 + n / 2)
    s += dt.kth(rg, l % (n / 2)) ?? 0  // timed: segtree.DynamicOrderTree.kth
print(s % 1000000007)
