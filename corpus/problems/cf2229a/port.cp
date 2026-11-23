// Source: Codeforces 2229A (Slimes on a Line), https://codeforces.com/contest/2229/problem/A
// Problem: n <= 1000 slimes at positions a[i] <= 1000. An operation picks x and moves every
//   slime one step toward x (those at x stay). Print the fewest operations that gather all
//   slimes at one position. Multitest, sum of n <= 1000.
// Solution: an operation moves max down and min up by at most 1 each, so max - min shrinks by
//   at most 2 and ceil((max - min) / 2) operations are needed; picking x strictly between the
//   extremes (x = min once they are 1 apart) reaches it. O(n) per test.

@!multitest
let n: 1..=200_000 = read()
let a: [0..n]-10 ** 9..=10 ** 9 = read()
print(ceil_div(a.max()! - a.min()!, 2))
