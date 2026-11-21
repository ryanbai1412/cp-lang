type Z = Mod<998_244_353>

@! multitest
let (n, m): (Int, Int) = read()
let bad_pos: [0..m](Int, Int) = read()
var bads: [0..2 * n - 1]List<Int> = []
for (i, x) in bad_pos:
    bads[i - x + n - 1].push(i - 1)
for k in 0..2 * n - 1:
    bads[k].sort(by: x => -x)

var dp: [0..=n]Z = 0
dp[0] = 1
var dp_sub: Z = 0
var dp_sub_lazy: [0..n + 2]Z = 0
var dp_x: [0..n]Z = 0
var dp_x_lazy: [0..n, 0..n + 2]Z = 0

for i in 0..=n:
    dp_sub += dp_sub_lazy[i]
    dp[i] -= dp_sub
    let bound = if i < n: bads[i + n - 1].last ?? n else: n
    for x0 in 0..n:
        dp_x[x0] += dp_x_lazy[x0, i]
        if i + x0 <= bound:
            dp[i + x0] += dp_x[x0]
    if i == n: break

    for j in i + 2..=bound:
        let k_max = min(2 * j - i - 1, if j < n: bads[j + n - 1].last ?? n else: n)
        if j + 1 <= k_max:
            dp_sub_lazy[j + 1] += dp[i]
            dp_sub_lazy[k_max + 1] -= dp[i]

    for off in i..=i + n - 1:
        let x0 = i + n - 1 - off
        while let last = bads[off].last, last < i:
            let _ = bads[off].pop()
        let j_lim = min(n - x0, bads[off].last ?? n)
        if i + 1 <= j_lim:
            dp_x_lazy[x0, i + 1] += dp[i]
            dp_x_lazy[x0, j_lim + 1] -= dp[i]

print(dp[n])
