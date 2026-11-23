// Source: Codeforces 2084A (Max and Mod), https://codeforces.com/contest/2084/problem/A
// Problem: multitest; n (2 <= n <= 100). Print a permutation p of 1..n with
//   max(p_{i-1}, p_i) mod i = i - 1 for all 2 <= i <= n, or -1 if none exists.
// Solution: [1, ..., pos - 1, n, pos, ..., n - 1] has max(p_{j-1}, p_j) = j - 1 except at
//   pos and pos + 1, where the max n needs pos | n + 1 and pos + 1 | n + 1. Odd n takes
//   pos = 1 ([n, 1, ..., n - 1]); for even n, n + 1 is odd so no such pos exists, and -1
//   is the answer. Search pos, then print: O(n) per test.

@!multitest
let n: 2..=100 = read()
if let pos = first(i for i in 1..=n if (n + 1) % i == 0 and (n + 1) % (i + 1) == 0):
    print([if j == pos: n elif j < pos: j else: j - 1 for j in 1..=n])
else:
    print(-1)
