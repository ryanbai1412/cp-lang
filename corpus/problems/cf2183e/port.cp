// Source: Codeforces 2183E (LCM is Legendary Counting Master),
//   https://codeforces.com/contest/2183/problem/E
// Problem: multitest; a_1..a_n with each a_i in 0..m (2 <= n <= m <= 3000, sum of m <= 3000).
//   Count ways to replace every 0 by a value in 1..m so that a is strictly increasing and
//   1/lcm(a_1, a_2) + ... + 1/lcm(a_{n-1}, a_n) + 1/lcm(a_n, a_1) >= 1, modulo 998244353.
// Solution: for x < y, 1/lcm(x, y) = gcd/(xy) <= (y - x)/(xy) = 1/x - 1/y, equal iff y - x
//   divides x, and the wrap term is at most 1/a_n, so the sum telescopes to at most 1/a_1:
//   a good sequence starts at 1 and every step goes x -> x + d with d | x. ways[x] counts
//   prefixes ending at x; a step pushes ways[j * d] to (j + 1) * d for every d and j
//   (O(m log m)), then zeroes all but a_i if it is given. O(n m log m) per test.
// Bounds: n, m: 2..=3000; a: 0..=m
type Z = Mod<998_244_353>

@!multitest
let (n, m): (2..=3000, 2..=3000) = read()
let a: [0..n]0..=m = read()
var ways: [1..=m]Z = 0
var next: [1..=m]Z = 0
if a[0] in 0..=1: ways[1] = 1
for i in 1..n:
    next.fill(0)
    for d in 1..=m:
        for j in 1..m / d: next[(j + 1) * d] += ways[j * d]
    if a[i] != 0:
        for x in 1..=m if x != a[i]: next[x] = 0
    swap(mut ways, mut next)
print(ways.sum())
