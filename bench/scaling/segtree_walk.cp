use std.segtree.{SegTree}
let n: Int = read()
domain V = 0..n
let reps = 1 << 17
var s = 0
let a: [V]Int = [V] v => (v.ord * 7919) % n + 1

var t = SegTree(+, a)
for i in 0..reps:
    let l = i * 7919 % n
    let r = l + (n - l) / 2
    s += t.walk(V(l)..V(r)).collect().len // timed: segtree.SegTree.walk
print(s % 1000000007)
