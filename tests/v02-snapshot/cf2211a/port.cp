// Source: Codeforces 2211A (Antimedian Deletion), https://codeforces.com/contest/2211/problem/A
// Problem: a permutation p of n <= 100. An operation takes 3 adjacent elements and deletes their
//   minimum or maximum. For each i, print the shortest reachable length of an array that still
//   holds p[i]. Multitest.
// Solution: an operation needs 3 elements, so lengths stop at 2, and any window has a minimum and
//   a maximum, one of them not p[i], so p[i] survives down to length 2. Print 2 for every i, or 1
//   when n = 1; O(n).

@!multitest
let n: 1..=100 = read()
let _: [0..n]1..=n = read()
print(List.filled(n, if n == 1: 1 else: 2))
