@!multitest
type Z = Mod<998_244_353>
let (n, m): (Int, Int) = read()
var bads: [0..2 * n - 1]List<Int> = []
for _ in 0..m:
    let (i, x): (Int, Int) = read()
    bads[i - x + n - 1].push(i - 1)
for off in 0..2 * n - 1: bads[off].sort(desc: true)

var dp: [0..=n]Z = 0
dp[0] = 1
for i in 0..n:
    let j_max0 = bads[i + n - 1].last ?? n
    for j in i + 2..=j_max0:
        var k_max = min(n, 2 * j - i - 1)
        if j < n: k_max min= bads[j + n - 1].last ?? n
        for k in j + 1..=k_max: dp[k] -= dp[i]

    for off in i..=i + n - 1:
        let x0 = i + n - 1 - off
        while let last = bads[off].last, last < i: bads[off].pop()
        let j_max = min(n - x0, bads[off].last ?? n)
        for j in i + 1..=j_max if j == n or (bads[j + n - 1].last ?? n) >= j + x0:
            dp[j + x0] += dp[i]

print(dp[n])
