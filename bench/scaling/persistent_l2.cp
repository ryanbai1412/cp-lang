// n: 4096
// These structures hold n log n or 8n nodes. n stays small so they still fit in
// L2 at 4n: past it each step pays a cache miss the cost model does not count,
// and the growth would measure the memory hierarchy, not the fn.
use std.persistent.{PersistentSegTree, RangeKth}
let n: Int = read()
domain V = 0..n
let reps = 1 << 17
var s = 0
let a: [V]Int = [V] v => (v.ord * 7919) % n + 1

// a fresh tree holds about 2n nodes; reserving 8n moves them
for _ in 0..800:
    var t = PersistentSegTree(+, a: a)
    t.reserve(8 * n) // timed: persistent.PersistentSegTree.reserve [let: len = 2 n]

for _ in 0..16:
    let k = RangeKth(a) // timed: persistent.RangeKth.new
    s += k.kth(V(0)..V(n - 1), 0)
let rk = RangeKth(a)
for i in 0..reps:
    let l = i * 7919 % (n - 1)
    let r = l + (n - 1 - l) / 2 + 1
    s += rk.kth(V(l)..V(r), (r - l) / 2) // timed: persistent.RangeKth.kth
    s += rk.count_le(V(l)..V(r), i % n) // timed: persistent.RangeKth.count_le
    s += rk.count_lt(V(l)..V(r), i % n) // timed: persistent.RangeKth.count_lt
    s += rk.count_in(V(l)..V(r), i % n, i % n + n / 3) // timed: persistent.RangeKth.count_in

print(s)
