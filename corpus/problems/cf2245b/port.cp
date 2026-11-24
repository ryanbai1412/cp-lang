// Source: Codeforces 2245B (Delete and Concatenate), https://codeforces.com/contest/2245/problem/B
// limits: 1500 ms, 256 MB
// Problem: n <= 2e5 integers |a_i| <= 1e9 and a cost c <= 1e9. Until the array is empty, either
//   remove one element (score += it) or two adjacent ones (score += their max); each operation
//   also costs c. Print the maximum final score. Multitest, sum n <= 2e5.
// Solution: every operation scores one element minus c, and at most n / 2 elements can be eaten
//   silently by a larger partner. So in sorted order the top n - n / 2 always score x - c,
//   and each of the n / 2 smallest scores max(x - c, 0) (eaten when x <= c). O(n log n).

@!multitest
let (n, c): (1..=200_000, -10 ** 9..=10 ** 9) = read()
let a: [0..n]-10 ** 9..=10 ** 9 = read()
let b = a.sorted()
let mid = n / 2
print(sum(max(x - c, 0) for x in b[0..mid]) + sum(x - c for x in b[mid..n]))
