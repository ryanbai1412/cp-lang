// Source: Codeforces 2183B (Yet Another MEX Problem),
//   https://codeforces.com/contest/2183/problem/B
// limits: 1500 ms, 256 MB
// Problem: n <= 2e5 values in 0..=n and 2 <= k <= n. n - k + 1 times, pick a length-k window of
//   maximum mex and delete one element inside it. Print the largest possible mex of the k - 1
//   elements left. Multitest, sum n <= 2e5.
// Solution: k - 1 elements have mex <= k - 1 and deleting never supplies a missing value, so the
//   answer is at most min(mex(a), k - 1), and that bound is reached. Mark the values below k - 1
//   and print the first unmarked one, else k - 1; O(n).
// Bounds: n, k: 2..=200_000; a: 0..=n

@!multitest
let (n, k): (2..=200_000, 2..=200_000) = read()
let a: [0..n]0..=n = read()
var has: [0..k - 1]Bool = false
for x in a if x < k - 1: has[x] = true
print((0..k - 1).find(x => not has[x]) ?? (k - 1))
