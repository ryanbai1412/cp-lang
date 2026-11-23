// Source: Codeforces 2084B (MIN = GCD), https://codeforces.com/contest/2084/problem/B
// Problem: given n <= 1e5 positive a_i <= 1e18, can a be rearranged so some split 1 <= i < n has
//   min(a_1..a_i) = gcd(a_{i+1}..a_n)? Print Yes or No. Multitest, sum n <= 1e5.
// Solution: the global minimum mn must be in the prefix (it is <= the suffix gcd), so the
//   suffix must have gcd mn: all its elements are multiples of mn, and adding more multiples
//   only pulls the gcd down toward mn. So the answer is Yes iff the gcd of every multiple of mn
//   other than one copy of mn equals mn. One pass, O(n log a).

@!multitest
let n: 2..=100_000 = read()
let a: [0..n]1..=10 ** 18 = read()
let p = argmin(a)!
let mn = a[p]
var g = 0
for i in 0..n if i != p and a[i] % mn == 0: g = gcd(g, a[i])
print(if g == mn: "Yes" else: "No")
