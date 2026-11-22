// expect: ok, output "1 3 -1 2 3 3\n1 -1 4 -1 5 0\nYES"
use std.segtree.{WaveletMatrix}
use std.rand.{Rng}
domain P = 0..5
let xs: [P]Int = [3, 1, 4, 1, 5]
let w = WaveletMatrix(xs)
let all = P(0)..=P(4)
print(w.kth(all, 0) ?? -1, w.kth(all, 2) ?? -1, w.kth(all, 5) ?? -1, w.rank(all, 1), w.count_lt(all, 4), w.count_in(all, 1, 3))
print(w.prev_val(all, 3) ?? -1, w.prev_val(all, 1) ?? -1, w.next_val(all, 3) ?? -1, w.next_val(all, 5) ?? -1, w.quantile_sum(all, 3), w.quantile_sum(P(1)..P(3), 0))
domain Q = 0..40
var rng = Rng(9)
var a: [Q]Int = 0
for i in Q: a[i] = rng.int(0, 20)
let v = WaveletMatrix(a)
var ok = true
for _ in 0..300:
    let lo = rng.int(0, 39)
    let hi = rng.int(lo, 39)
    let x = rng.int(-2, 23)
    var ys = [a[Q(i)] for i in lo..=hi]
    ys.sort()
    let r = Q(lo)..=Q(hi)
    let k = rng.int(0, ys.len - 1)
    var lt = 0
    var eq = 0
    var pre = 0
    for i in 0..ys.len:
        if ys[i] < x: lt += 1
        if ys[i] == x: eq += 1
        if i < k: pre += ys[i]
    if v.kth(r, k) != Some(ys[k]) or v.count_lt(r, x) != lt or v.rank(r, x) != eq or v.quantile_sum(r, k) != pre: ok = false
    let pv: ?Int = if lt == 0: None else: Some(ys[lt - 1])
    if v.prev_val(r, x) != pv: ok = false
print(yes(ok))
