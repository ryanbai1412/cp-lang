// Source: Codeforces 2222D (Permutation Construction),
//   https://codeforces.com/contest/2222/problem/D
// Problem: given n <= 2e5 values |a_i| <= 1e9, the value of an inversion (i < j, p_i > p_j) of a
//   permutation p is a_i + ... + a_{j-1}, and its beauty is the sum over its inversions.
//   Print any permutation of maximum beauty. Multitest, sum n <= 2e5.
// Solution: with prefix sums P, the value of (i, j) is P_{j-1} - P_{i-1}, positive exactly when
//   P_{i-1} < P_{j-1}. Ranking the positions by descending P_{i-1} makes every positive pair an
//   inversion and no negative one, so it is optimal. Sort indices by (prefix before i, i),
//   invert that order to get ranks, print n - rank; O(n log n).

@!multitest
let n: 1..=200_000 = read()
domain I = 0..n
let a: [I]-10 ** 9..=10 ** 9 = read()
let pref = prefix(a)
var ord: [I]I = [I] i => i
ord.sort(by: i => (pref.prefix(i.ord), i))
let rank = ord.inverse()
print([n - rank[i].ord for i in I])
