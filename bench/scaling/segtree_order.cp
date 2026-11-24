use std.segtree.{DynamicOrderTree}
let n: Int = read()
domain V = 0..n
let reps = 1 << 17
var s = 0
let a: [V]Int = [V] v => (v.ord * 7919) % n + 1

for _ in 0..3:
    s += DynamicOrderTree(a).get(V(0)) // timed: segtree.DynamicOrderTree.new
var dt = DynamicOrderTree(a)
for i in 0..reps / 8:
    let l = i * 7919 % n
    dt.set(V(i * 13 % n), l) // timed: segtree.DynamicOrderTree.set
for i in 0..reps:
    s += dt.get(V(i * 17 % n)) // timed: segtree.DynamicOrderTree.get
print(s % 1000000007)
