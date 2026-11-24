// Source: Codeforces 2165B (Marble Council), https://codeforces.com/contest/2165/problem/B
// limits: 2000 ms, 512 MB
// Problem: a multiset a of n <= 5000 values in 1..=n. Split a into any non-empty groups and
//   put one mode of each group into s. Count the distinct multisets s, mod 998244353.
//   Multitest, sum of n <= 5000.
// Solution: s is fixed by the set S of values in it and a count 1..=freq[v] for each v in S,
//   and it can be made iff the frequencies in S sum to at least the largest frequency. So
//   knapsack over the distinct values on total frequency, each value weighted by its
//   freq[v] choices (dp[i + f] += dp[i] * f), and sum dp[i] for i >= max freq: O(n^2).
// Bounds: n: 1..=5_000; a: 1..=n

const MOD = 998_244_353
type Z = Mod<MOD>

@!multitest
let n: 1..=5_000 = read()
let a: [0..n]1..=n = read()
var freq: [1..=n]Int = 0
for x in a: freq[x] += 1

var dp: [0..=n]Z = 0
dp[0] = 1
var sz = 0
for f in freq if f > 0:
    for i in (0..=sz).rev: dp[i + f] += dp[i] * f
    sz += f

print(sum(dp[i] for i in freq.max()!..=n))
