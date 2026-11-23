let (n, k): (1..=200000, 1..=200000) = read()
let a: [0..n]Int = read()
var cur = 0
for i in 0..k:  // expect: loop_reduction
    cur += a[i]
var best = cur
for i in k..n:
    cur += a[i] - a[i - k]  // expect: sliding_sum_by_hand
    best = max(best, cur)  // expect: chmin_chmax_if
print(best)
