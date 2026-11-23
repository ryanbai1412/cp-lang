// Source: NOI 2009 P5, Pipe Marbles, https://dmoj.ca/problem/noi09p5
// Problem: two pipes of n and m A/B marbles (n, m <= 500); a move shoots the rightmost marble of
//   either into the output. If output sequence i arises from a_i move orders, print sum a_i^2
//   mod 1024523.
// Solution: sum a_i^2 counts pairs of games with the same output, so run two games in lockstep:
//   dp[i, j] after k marbles has game one i taken from pipe one and game two j, and each step
//   both take an equal marble (4 choices). Reading the pipes from the left instead pairs the
//   reversed outputs, the same count. O((n + m) n^2), one layer kept.

type Z = Mod<1024523>

let (n, m): (Int, Int) = read()
let s1: Str = read()
let s2: Str = read()

let stride = n + 1
var dp = List.filled(stride * stride, Z.zero)
dp[0] = 1

for k in 0..(n + m):
    var nxt = List.filled(stride * stride, Z.zero)
    let lo = max(0, k - m)
    let hi = min(n, k)
    for i in lo..=hi:
        let i2 = k - i
        let row = i * stride
        let row1 = (i + 1) * stride
        for j in lo..=hi:
            let cur = dp[row + j]
            if cur == 0: continue
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
