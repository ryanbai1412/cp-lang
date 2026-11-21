@!multitest
let (n, m, p100): (Int, Int, Int) = read()
let p = F64(p100) / 100.0
let h: List<Int> = [read::<Int>() for _ in 0..n]
let min_h = min(x for x in h)
let tot_ones = sum(x - min_h for x in h)
let tot_all = min_h - 1
if tot_ones + tot_all > m:
    print(0.0)
    return
var endgames: [0..(tot_all + 1)]F64 = 0.0
var dp: [0..(tot_all + 1), 0..n]F64 = 0.0
var ndp: [0..(tot_all + 1), 0..n]F64 = 0.0
dp[0, 0] = 1.0
for step in 0..=m:
    let rem = m - step
    if rem >= tot_ones and rem <= tot_ones + tot_all:
        let idx = tot_all + tot_ones - rem
        endgames[idx] = dp[idx, 0]
    for i in 0..=tot_all:
        for j in 0..n:
            let fail = if j > 0: dp[i, j - 1] elif i > 0: max(dp[i - 1, n - 1], dp[i, j]) else: 1.0
            ndp[i, j] = p * dp[max(i - 1, 0), j] + (1.0 - p) * fail
    swap(mut dp, mut ndp)
var endgames2: [0..(tot_ones + 1)]F64 = 0.0
var dp2: [0..(tot_ones + 1)]F64 = 0.0
var ndp2: [0..(tot_ones + 1)]F64 = 0.0
dp2[0] = 1.0
for step in 0..=m:
    let rem = m - step
    if rem >= tot_all and rem <= tot_ones + tot_all:
        let idx = tot_all + tot_ones - rem
        endgames2[idx] = dp2[idx]
    for i in 0..=tot_ones:
        ndp2[i] = p * dp2[i] + (1.0 - p) * dp2[max(i - 1, 0)]
    swap(mut dp2, mut ndp2)
var win: [0..(tot_ones + 1), 0..(tot_all + 1)]F64 = 0.0
for j in 0..=tot_all: win[0, j] = endgames[j]
for i in 0..=tot_ones: win[i, 0] = endgames2[i]
for i in 1..=tot_ones:
    for j in 1..=tot_all:
        win[i, j] = p * win[i, j - 1] + (1.0 - p) * win[i - 1, j]
print(win[tot_ones, tot_all])
