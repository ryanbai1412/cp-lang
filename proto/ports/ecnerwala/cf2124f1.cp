@!multitest
type Z = Mod<998_244_353>
let (n, m): (Int, Int) = read()
var bads: [0..(2 * n - 1)]List<Int> = []
for _ in 0..m:
    let (i, x): (Int, Int) = read()
    bads[(i - 1) - (x - 1) + (n - 1)].push(i - 1)
for off in 0..(2 * n - 1):
    bads[off].sort()
    bads[off].reverse()
var dp: [0..(n + 1)]Z = 0
dp[0] = 1
for i in 0..n:
    let off0 = i + (n - 1)
    let j_max0 = if let last = bads[off0].last(): last else: n
    for j in (i + 2)..=j_max0:
        var k_max = min(n, 2 * j - i - 1)
        if j < n:
            if let last = bads[j + (n - 1)].last():
                k_max min= last
        for k in (j + 1)..=k_max:
            dp[k] -= dp[i]
    for off in i..=(i + (n - 1)):
        let x0 = i + (n - 1) - off
        while let last = bads[off].last():
            if last >= i: break
            bads[off].pop()
        var j_max = n - x0
        if let last = bads[off].last():
            j_max min= last
        for j in (i + 1)..=j_max:
            var ok = j == n
            if not ok:
                if let last = bads[j + (n - 1)].last():
                    ok = last >= j + x0
                else:
                    ok = true
            if ok:
                dp[j + x0] += dp[i]
print(dp[n])
