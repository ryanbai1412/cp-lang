// Source: Codeforces 2187A (Restricted Sorting), https://codeforces.com/contest/2187/problem/A
// Problem: k is piggy if a (n <= 2e5, a_i <= 1e9) can be sorted by swaps of a_i, a_j with
//   |a_i - a_j| >= k. Print the largest piggy k, or -1 if there is none (a already sorted, so
//   every k works). Multitest, sum n <= 2e5.
// Solution: the farthest partner of x is the minimum lo or the maximum hi, so x can move at all
//   iff k <= max(x - lo, hi - x); then it can reach any slot by swaps through lo or hi. So the
//   answer is the minimum of max(x - lo, hi - x) over the elements whose slot in the stable
//   sort of (value, index) is not their own index, or -1 if none; O(n log n).

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]1..=10 ** 9 = read()
let s = sorted([(a[i], i) for i in 0..n])
let (lo, hi) = (s[0].0, s.last!.0)
print(s.items.filter((i, (x, j)) => j != i).map((i, (x, j)) => max(x - lo, hi - x)).min() ?? -1)
