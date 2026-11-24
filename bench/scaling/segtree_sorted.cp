use std.segtree.{MergeSortTree}
let n: Int = read()
domain V = 0..n
let reps = 1 << 17
var s = 0
let a: [V]Int = [V] v => (v.ord * 7919) % n + 1

for _ in 0..10:
    s += MergeSortTree(a).count_lt(V(0)..V(n - 1), n / 2) // timed: segtree.MergeSortTree.new
let mt = MergeSortTree(a)
for i in 0..reps / 4:
    let l = i * 7919 % n
    let rg = V(l / 2)..V(l / 2 + n / 2)
    s += mt.count_lt(rg, l) // timed: segtree.MergeSortTree.count_lt
    s += mt.count_le(rg, l) // timed: segtree.MergeSortTree.count_le
    s += mt.count_gt(rg, l) // timed: segtree.MergeSortTree.count_gt
    s += mt.count_ge(rg, l) // timed: segtree.MergeSortTree.count_ge
    s += mt.count_in(rg, l / 2, l) // timed: segtree.MergeSortTree.count_in
    s += mt.first_ge(rg, l) ?? 0 // timed: segtree.MergeSortTree.first_ge
    s += mt.last_le(rg, l) ?? 0 // timed: segtree.MergeSortTree.last_le
print(s % 1000000007)
