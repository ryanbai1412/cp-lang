// Source: Codeforces 2180B (Ashmal), https://codeforces.com/contest/2180/problem/B
// limits: 1000 ms, 256 MB
// Problem: strings a_1..a_n (n <= 1000, total length L <= 4000, multitest) are added in order,
//   each to the front or the back of an initially empty s. Print the lexicographically smallest
//   final s.
// Solution: all candidates after a step have the same length, and s1 < s2 gives both
//   s1 + a < s2 + a and a + s1 < a + s2, so keeping only s = min(s + a, a + s) is optimal.
//   O(n L) character copies.

@!multitest
let n: 1..=1_000 = read()
var s = ""
for _ in 0..n:
    let a: Str = read()
    s = min(s + a, a + s)
print(s)
