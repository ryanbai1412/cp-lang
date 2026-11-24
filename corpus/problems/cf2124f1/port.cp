// Source: Codeforces 2124F1 (Appending Permutations, Easy Version),
//   https://codeforces.com/contest/2124/problem/F1
// limits: 3000 ms, 1024 MB
// Problem: build an array of length n by appending cyclic shifts [r..s, 1..r-1] of [1..s], with
//   m bans a[i] != x. Count the distinct arrays mod 998244353. n <= 100, m <= 5000, sums over
//   tests too; multitest.
// Solution: dp[i] counts arrays of length i; a block from i is a run x0+1, x0+2, ... up to j,
//   then 1..x0. Bans are grouped by diagonal pos - val, so a run is ban-free when the next ban
//   on its diagonal is past it (bans sorted, popped as i passes). [1..L] then [1..M] with M < L
//   is also [1..M] then the shift [M+1..L, 1..M], so dp subtracts each such pair once. O(n^3).
// Bounds: n: 1..=100; m: 0..=10_000

@!multitest
type Z = Mod<998_244_353>
let (n, m): (1..=100, 0..=10_000) = read()
fn diag(pos: Int, val: Int) -> Int: pos - val + n - 1

var bads: [0..2 * n - 1]List<Int> = []
for _ in 0..m:
    let (i, x): (1..=n, 1..=n) = read()
    bads[diag(i, x)].push(i - 1)
for off in 0..2 * n - 1: bads[off].sort(by: x => desc(x))
fn next_bad(off: Int) -> Int: bads[off].last ?? n

var dp: [0..=n]Z = 0
dp[0] = 1
for i in 0..n:
    let j_max0 = next_bad(diag(i, 0))
    for j in i + 2..=j_max0:
        var end_max = min(n, 2 * j - i - 1)
        if j < n: end_max min= next_bad(diag(j, 0))
        for end in j + 1..=end_max: dp[end] -= dp[i]

    for off in i..=diag(i, 0):
        let x0 = diag(i, 0) - off
        while let last = bads[off].last and last < i: bads[off].pop()
        let j_max = min(n - x0, next_bad(off))
        for j in i + 1..=j_max if j == n or next_bad(diag(j, 0)) >= j + x0:
            dp[j + x0] += dp[i]

print(dp[n])
