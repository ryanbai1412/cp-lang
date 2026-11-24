// Source: Codeforces 2261B (What's the Capital of France?),
//   https://codeforces.com/contest/2261/problem/B
// limits: 1500 ms, 256 MB
// Problem: n <= 2e5 ratings 0 <= a_i <= 1e9 in a line; an operation on two adjacent unequal
//   ratings raises the lower by 1. Print the largest bitwise OR of all ratings reachable.
//   Multitest, sum n <= 2e5.
// Solution: ratings stay within [min, max], so they keep the bits above d, the top bit where
//   min and max differ, and the OR is at most those bits with every bit <= d set. Some adjacent
//   pair differs at bit d, and raising its lower one to the shared high bits, 0 at d and all
//   ones below reaches that bound. So take, over unequal adjacent (x, y), the max of
//   x | (2^bit_width(x ^ y) - 1), or a[0] when all are equal; O(n).

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]0..=2 ** 30 - 1 = read()
print(
    a.adjacent.filter((x, y) => x != y).map(
        (x, y) => x | ((1 << bit_width(x ^ y)) - 1)
    ).max() ?? a[0]
)
