// Source: Codeforces 2187E (Doors and Keys), https://codeforces.com/contest/2187/problem/E
// limits: 3000 ms, 1024 MB
// Problem: rooms 1..n+1 in a line; door i (rooms i, i+1) opens by itself at second a_i <= 2e5
//   or when a key is spent on it. Room i holds a key if s_i = 1; you carry at most one key at
//   a time, and a move takes a second. For each i print the least time to reach room i+1
//   (each i on its own). Multitest, sum n <= 2e5.
// Solution: dp[k] = least time to be past door i with k keys brought along; ferrying k >= 1
//   keys one room costs 2k - 1 seconds. A key room adds a key (k -> k + 1). At door i, wait:
//   max(a_i, dp[k]) + max(1, 2k - 1), or spend one of k + 1 keys: dp[k+1] + 2k + 1. States
//   worse than n + 1 + max a (waiting everywhere) are popped, so O(sqrt(n + max a)) remain:
//   O(n sqrt(n + max a)). The answer for i is dp[0].
// Bounds: n: 1..=200_000; a: 0..=10 ** 9

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]0..=10 ** 9 = read()
let s: Str = read()

let t_inf = n + 1 + a.max()!
var dp: List<Int> = [0]
var ans: [0..n]Int = 0

for i in 0..n:
    if s[i] == '1': dp.insert(0, dp[0])
    for k in 0..dp.len:
        var best = max(a[i], dp[k]) + 1 + 2 * max(k - 1, 0)
        if k + 1 < dp.len:
            best min= dp[k + 1] + 1 + 2 * k
        dp[k] = best
    while dp.last! > t_inf: dp.pop()
    ans[i] = dp[0]

print(ans)
