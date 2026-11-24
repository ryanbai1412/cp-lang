// Source: Codeforces 2137A (Collatz Conjecture), https://codeforces.com/contest/2137/problem/A
// limits: 1000 ms, 256 MB
// Problem: k Collatz steps (x even: x / 2, else 3x + 1) turned some start value into x. Given
//   1 <= k, x <= 20, print any possible start. Multitest.
// Solution: start at x * 2^k: every value on the way is even, so each step halves it. Print
//   `x << k` (at most 20 * 2^20, no overflow), O(1).
// Bounds: k, x: 1..=20

@!multitest
let (k, x): (1..=20, 1..=20) = read()
print(x << k)
