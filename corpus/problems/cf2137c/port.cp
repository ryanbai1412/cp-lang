// Source: Codeforces 2137C (Maximum Even Sum), https://codeforces.com/contest/2137/problem/C
// limits: 2000 ms, 256 MB
// Problem: integers a, b with 1 <= a, b <= a b <= 1e18. Pick k dividing b, replace (a, b) by
//   (a k, b / k); print the largest even a + b reachable, or -1. Multitest.
// Solution: the product P = a b is fixed and x + P / x grows toward the extremes, so take the
//   largest k with matching parities. Both odd: k = b gives a b + 1. b even and 4 | a b: k = b / 2
//   gives a b / 2 + 2. Otherwise one side is odd for every k: -1. O(1).

@!multitest
let (a, b): (1..=10 ** 18, 1..=10 ** 18) = read()
let ans = if a % 2 == 1 and b % 2 == 1: a * b + 1
elif b % 2 == 0 and (a * b) % 4 == 0: a * b / 2 + 2
else: -1
print(ans)
