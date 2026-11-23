// Source: Codeforces 2245A (Who Watches the Watchpig?),
//   https://codeforces.com/contest/2245/problem/A
// Problem: n <= 100 pigs in a row face L or R; pigs x < y with x facing R and y facing L form a
//   pair. Given 1 <= k < n, flip the fewest pigs so every pig is in >= k pairs; print that count,
//   or -1. Multitest.
// Solution: a pig facing L needs k R's before it and one facing R needs k L's after it, so the
//   first k must face R and the last k face L. If n >= 2 k that suffices (every middle pig pairs
//   with one whole end), else print -1. The answer is the mismatches in s[0..k] and s[n - k..n],
//   O(n).

@!multitest
let (n, k): (1..=200_000, 1..=200_000) = read()
let s: [0..n]Char = read()
if n < 2 * k:
    print(-1)
    return
print(2 * k - s[0..k].count('R') - s[n - k..n].count('L'))
