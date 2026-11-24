// Source: Codeforces 2207 E2 (N-MEX, counting version),
//   https://codeforces.com/contest/2207/problem/E2
// limits: 2000 ms, 256 MB
// Problem: given a_1..a_n (n <= 2e5, a_i <= 1e9), count arrays b with 0 <= b_i <= n where the
//   (n - i + 1)-th smallest missing value of b_1..b_i is a_i for every i, modulo 1e9 + 7.
//   Multitest, sum of n <= 2e5.
// Solution: a must satisfy n - i <= a_i <= a_{i-1} (a_0 = n), else 0. Undo b_n, .., b_1 in
//   turn, keeping pool, the free values below the current a. Going from a_i to a_{i-1}: if a
//   rises to cur, the values strictly between join the pool and b_i is a pool value or any of
//   cur..n (pool + n - cur + 1 ways); if it stays, b_i takes one pool value. O(n).

type Z = Mod<1_000_000_007>

@!multitest
let n: 1..=200_000 = read()
var a: [0..=n]0..=10 ** 9 = n
for i in 1..=n: a[i] = read()

if not (1..=n).all(i => a[i] in (n - i)..=a[i - 1]):
    print(0)
    return

var ans: Z = 1
var prv = a[n]
var pool = a[n]
for i in (0..n).rev:
    let cur = a[i]
    if cur > prv:
        pool += cur - prv - 1
        prv = cur
        ans *= pool + (n - cur + 1)
    else:
        ans *= pool
        pool -= 1
print(ans)
