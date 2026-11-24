// Source: Codeforces 2135A (Against the Difference), https://codeforces.com/contest/2135/problem/A
// limits: 2000 ms, 256 MB
// Problem: a block is x copies of x, and an array is neat if it is blocks concatenated. Given
//   n <= 2e5 values a_i in 1..=n, print the length of the longest neat subsequence. Multitest,
//   sum n <= 2e5.
// Solution: dp[j] = the longest neat subsequence of a[j..]. Going right to left, inds[x] holds
//   the positions of x up to i; a block of x ending at i best starts at the x-th last of them,
//   so dp[start] max= dp[i + 1] + x, and dp[i] max= dp[i + 1]. O(n).

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]1..=n = read()
var inds: [1..=n]List<Int> = []
for (i, x) in a.items: inds[x].push(i)

var dp: [0..=n]Int = 0
for i in (0..n).rev:
    dp[i] max= dp[i + 1]
    let x = a[i]
    if inds[x].len >= x:
        let start = inds[x][inds[x].len - x]
        dp[start] max= dp[i + 1] + x
    inds[x].pop()
print(dp[0])
