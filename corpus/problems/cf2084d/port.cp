// Source: Codeforces 2084D (Arcology On Permafrost), https://codeforces.com/contest/2084/problem/D
// Problem: given n <= 2e5, m, k with m k < n, f(b) is the least mex of b after at most m
//   removals of k consecutive elements (the rest concatenated). Print any a of length n,
//   0 <= a_i <= 1e9, maximising f(a). Multitest, sum of n <= 2e5.
// Solution: every value below the mex must survive m removals, so needs m + 1 copies kept apart.
//   The code walks the positions residue class by residue class mod k (stride k, starting from
//   the class of n - 1) and gives the cells in that order cur / (m + 1): 0, 0, ..., 1, 1, ...,
//   each value m + 1 times. O(n).

@!multitest
let (n, m, k): (2..=200_000, 1..=200_000, 1..=200_000) = read()
var a: [0..n]Int = 0
var cur = 0
for i in 0..k:
    for v in (n - 1 - i) % k..n by k:
        a[v] = cur / (m + 1)
        cur += 1
print(a)
