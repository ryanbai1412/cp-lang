// fixed: C
use std.math.{gcd}
use std.sparse.{DisjointSparseTable, LinearRmq, SparseTable2D, idempotent}
let n: Int = read()
domain V = 0..n
domain C = 0..4
domain R = 0..n / 16
let a = [V] v => (v.ord * 7919) % 1000003
let g = [R, C] (v, c) => (v.ord * 31 + c.ord * 7) % 1009
let xs = [(i * 7919) % 1000003 for i in 0..n]
var s = 0
for _ in 0..16:
    s += SparseTable<V, Int>(min, a).get(V(0)) // timed: sparse.SparseTable.new
    s += DisjointSparseTable<V, Int>(+, a).get(V(0)) // timed: sparse.DisjointSparseTable.new
for _ in 0..64:
    s += LinearRmq<V, Int>(a).best(V(0)..V(1)) // timed: sparse.LinearRmq.new
for _ in 0..32:
    s += SparseTable2D<R, C, Int>(max, g).get(R(0), C(0)) // timed: sparse.SparseTable2D.new
for _ in 0..512:
    if idempotent<Int>(gcd, xs): s += 1 // timed: sparse.idempotent
let st = SparseTable<V, Int>(min, a)
let ds = DisjointSparseTable<V, Int>(+, a)
let lr = LinearRmq<V, Int>(a)
let s2 = SparseTable2D<R, C, Int>(max, g)
for i in 0..8 * n:
    let l = V((i * 31) % n)
    let r = V((i * 31 + i % 977) % n)
    let (lo, hi) = if l <= r: (l, r) else: (r, l)
    let c = C(i % 4)
    let (r0, r1) = (R(lo.ord / 16), R(hi.ord / 16))
    s += st.fold(lo..=hi) // timed: sparse.SparseTable.fold
    s += st.try_fold(lo..=hi) ?? 0 // timed: sparse.SparseTable.try_fold
    s += st.get(lo) // timed: sparse.SparseTable.get
    s += st.all() // timed: sparse.SparseTable.all
    s += ds.fold(lo..=hi) // timed: sparse.DisjointSparseTable.fold
    s += ds.try_fold(lo..=hi) ?? 0 // timed: sparse.DisjointSparseTable.try_fold
    s += ds.get(lo) // timed: sparse.DisjointSparseTable.get
    s += ds.all() // timed: sparse.DisjointSparseTable.all
    s += lr.best(lo..=hi) // timed: sparse.LinearRmq.best
    s += lr.try_best(lo..=hi) ?? 0 // timed: sparse.LinearRmq.try_best
    s += lr.argbest(lo..=hi).ord // timed: sparse.LinearRmq.argbest
    s += s2.fold(r0..=r1, C(0)..=c) // timed: sparse.SparseTable2D.fold
    s += s2.try_fold(r0..=r1, c..=c) ?? 0 // timed: sparse.SparseTable2D.try_fold
    s += s2.get(r1, c) // timed: sparse.SparseTable2D.get
    s %= 1000000007
print(s)
