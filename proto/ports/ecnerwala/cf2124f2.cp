const MOD = 998244353

@! multitest
let n: Int = read()
let m: Int = read()
var bads: [0..2 * n - 1]List<Int> = []
for _ in 0..m:
    let (i0, x0): (Int, Int) = read()
    let i = i0 - 1
    let x = x0 - 1
    bads[i - x + (n - 1)].push(i)
for k in 0..2 * n - 1:
    bads[k].sort()
    bads[k].reverse()
var dp: [0..=n]Mod<MOD> = 0
dp[0] = 1
var dp_sub: Mod<MOD> = 0
var dp_sub_lazy: [0..n + 2]Mod<MOD> = 0
var dp_x: [0..n]Mod<MOD> = 0
var dp_x_lazy: [0..n][0..n + 2]Mod<MOD> = 0
for i in 0..=n:
    dp_sub += dp_sub_lazy[i]
    dp[i] -= dp_sub
    for x0 in 0..n:
        dp_x[x0] += dp_x_lazy[x0][i]
        if i + x0 <= n:
            var ok = true
            if i < n:
                if let last = bads[i + (n - 1)].last():
                    ok = last >= i + x0
            if ok:
                dp[i + x0] += dp_x[x0]
    if i == n:
        break
    var j_max = n
    if let last = bads[i + (n - 1)].last():
        j_max = last
    if i + 2 <= j_max:
        for j in i + 2..=j_max:
            var k_max = min(n, 2 * j - i - 1)
            if j < n:
                if let last = bads[j + (n - 1)].last():
                    k_max min= last
            let k_min = j + 1
            if k_min <= k_max:
                dp_sub_lazy[k_min] += dp[i]
                dp_sub_lazy[k_max + 1] -= dp[i]
    for off in i..=i + (n - 1):
        let x0 = i + (n - 1) - off
        while let last = bads[off].last(), last < i:
            let _ = bads[off].pop()
        var j_lim = n - x0
        if let last = bads[off].last():
            j_lim min= last
        let j_min = i + 1
        if j_min <= j_lim:
            dp_x_lazy[x0][j_min] += dp[i]
            dp_x_lazy[x0][j_lim + 1] -= dp[i]
print(dp[n])
