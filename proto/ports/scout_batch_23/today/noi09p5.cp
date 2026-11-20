// NOI '09 P5 - Pipe Marbles (runnable on proto/cp.py lib-run today)
use std.mod.{Mod}

type Z = Mod<1024523>

let (n, m): (Int, Int) = read()
let s1: Str = read()
let s2: Str = read()

let stride = n + 1
var dp = List.filled(stride * stride, Z(0))
dp[0] = Z(1)

for k in 0..(n + m):
    var nxt = List.filled(stride * stride, Z(0))
    let lo = max(0, k - m)
    let hi = min(n, k)
    for i in lo..=hi:
        let i2 = k - i
        let row = i * stride
        let row1 = (i + 1) * stride
        for j in lo..=hi:
            let cur = dp[row + j]
            if cur == Z(0): continue
            let j2 = k - j
            if i < n and j < n and s1[i] == s1[j]:
                nxt[row1 + j + 1] += cur
            if i < n and j2 < m and s1[i] == s2[j2]:
                nxt[row1 + j] += cur
            if i2 < m and j < n and s2[i2] == s1[j]:
                nxt[row + j + 1] += cur
            if i2 < m and j2 < m and s2[i2] == s2[j2]:
                nxt[row + j] += cur
    dp = nxt

print(dp[n * stride + n])
