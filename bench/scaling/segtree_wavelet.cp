use std.segtree.{WaveletMatrix}
let n: Int = read()
domain V = 0..n
let reps = 1 << 17
var s = 0
let a: [V]Int = [V] v => (v.ord * 7919) % n + 1

for _ in 0..20:
    s += WaveletMatrix(a).rank(V(0)..V(n - 1), 3) // timed: segtree.WaveletMatrix.new
let wm = WaveletMatrix(a)
for i in 0..reps:
    let l = i * 7919 % n
    let rg = V(l / 2)..V(l / 2 + n / 2)
    s += wm.kth(rg, l % (n / 2)) ?? 0 // timed: segtree.WaveletMatrix.kth
    s += wm.rank(rg, l) // timed: segtree.WaveletMatrix.rank
    s += wm.count_lt(rg, l) // timed: segtree.WaveletMatrix.count_lt
    s += wm.count_in(rg, l / 2, l) // timed: segtree.WaveletMatrix.count_in
    s += wm.prev_val(rg, l) ?? 0 // timed: segtree.WaveletMatrix.prev_val
    s += wm.next_val(rg, l) ?? 0 // timed: segtree.WaveletMatrix.next_val
    s += wm.quantile_sum(rg, l % (n / 2)) // timed: segtree.WaveletMatrix.quantile_sum
print(s % 1000000007)
