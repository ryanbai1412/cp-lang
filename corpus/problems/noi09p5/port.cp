// Source: NOI 2009 P5, Pipe Marbles, https://dmoj.ca/problem/noi09p5
// Problem: two pipes of n and m A/B marbles (n, m <= 500); a move shoots the rightmost marble of
//   either into the output. If output sequence i arises from a_i move orders, print sum a_i^2
//   mod 1024523.
// Solution: sum a_i^2 counts pairs of games with the same output, so run two games in lockstep:
//   dp[i, j] after k marbles has game one i taken from pipe one and game two j, and each step
//   both take an equal marble (4 choices). Reading the pipes from the left instead pairs the
//   reversed outputs, the same count. O((n + m) n^2), one layer kept.
// original: corpus/submissions/5108100.cpp
type Z = Mod<1024523>

let (n, m): (1..=500, 1..=500) = read()
let s1: [0..n]Char = read()
let s2: [0..m]Char = read()

// dp[i, j]: pairs of games, k marbles out, game one has taken i from pipe one, game two j
var dp: [0..=n, 0..=n]Z = 0
dp[0, 0] = 1
for k in 0..n + m:
    var next: [0..=n, 0..=n]Z = 0
    let taken = max(0, k - m)..=min(n, k)
    for i in taken:
        for j in taken:
            let cur = dp[i, j]
            let (i2, j2) = (k - i, k - j)
            if i < n and j < n and s1[i] == s1[j]: next[i + 1, j + 1] += cur
            if i < n and j2 < m and s1[i] == s2[j2]: next[i + 1, j] += cur
            if i2 < m and j < n and s2[i2] == s1[j]: next[i, j + 1] += cur
            if i2 < m and j2 < m and s2[i2] == s2[j2]: next[i, j] += cur
    dp = next
print(dp[n, n])
